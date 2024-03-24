import copy
from collections import defaultdict
from typing import Dict, List, Tuple, Union

from torch import Tensor
from torch_geometric.data import HeteroData

from ofnil.infra.graph.hop_collector import HopCollector
from ofnil.infra.kv.feature_view import FeatureView


class PyGHopCollector(HopCollector):
    def __init__(self, disjoint=False, sparse=True):
        """This class supports PyG format sampler output.
        In specific, it aligns with NeighborLoader in PyG [https://pytorch-geometric.readthedocs.io/en/latest/modules/loader.html?highlight=NeighborLoader#torch_geometric.loader.NeighborLoader].

        Parameters
        ----------
        disjoint : bool, optional
            If set to True, mini-batch outputs will have a batch vector holding the mapping of nodes to their respective subgraph.
            Will get automatically set to True in case of temporal sampling.
            By default False
        sparse : bool, optional
            If set to True, the output adjacency matrix will be a sparse matrix, by default True.

        """
        self.disjoint = disjoint
        self.sparse = sparse

    def reset_seeds(self, seeds: Tuple[str, Union[Tensor, List[int]]]):
        self.seeds = seeds
        self.hops = []

    def add_hop(
        self,
        row_idx: List[List[int]],
        col_idx: List[List[int]],
        seed_type: List[str] = None,
        nbr_type: List[str] = None,
        edge_type: List[str] = None,
    ) -> Dict[str, List[int]]:
        """Add the sampled data index (row_idx, col_idx) and
        the corresponding types (seed_type, nbr_type, edge_type) for each edge type in each hop.
        Each element represents a list of sampled data index (row_idx:List[int], col_idx:List[int]) or
        its type (seed_type:str, nbr_type:str, edge_type:str) for each edge type.
        """
        if len(self.hops) == 0:
            # self.seed_type is the node type of the root node of each instance graph.
            self.seed_type = seed_type
        hop_list = list(zip(row_idx, col_idx, seed_type, nbr_type, edge_type))
        self.hops.append(hop_list)
        frontier = defaultdict(dict)
        for col_idx_, row_idx_, nbr_type_ in zip(col_idx, row_idx, nbr_type):
            if self.disjoint:
                # dedup for instance graph
                # TODO(kaili)
                pass
            else:
                frontier[nbr_type_] = list(set(row_idx_))
        return frontier

    def _get_local_id_map(self, id_fn):
        vids_by_tlabel = defaultdict(set)
        for hop_with_etypes in self.hops:
            for hop in hop_with_etypes:
                row_idx, col_idx, seed_type, nbr_type, edge_type = hop
                vids_by_tlabel[seed_type].update(col_idx)
                vids_by_tlabel[nbr_type].update(row_idx)
        id_map = {tlabel: id_fn(tlabel, list(vids)) for tlabel, vids in vids_by_tlabel.items()}
        return id_map

    def to_local(self, id_fn):
        if len(self.hops) == 0:
            self.heterodata = HeteroData()
            return self

        if self.disjoint:
            # TODO(kaili)
            pass
        else:
            id_map = self._get_local_id_map(id_fn)
            to_local_vertex = {}
            hop_vertices = defaultdict(dict)  # local to original id for each node type
            rows, cols = defaultdict(dict), defaultdict(dict)
            original_ids = defaultdict(dict)
            for hop_with_etypes in self.hops:
                for hop in hop_with_etypes:
                    row_idx, col_idx, seed_type, nbr_type, edge_type = hop
                    for i, vid in enumerate(col_idx):
                        original_id = id_map[seed_type][vid]
                        assert original_id is not None, f"seed_type={seed_type}, vid={vid}, id_map={id_map}"
                        if original_id not in to_local_vertex:
                            local_id = to_local_vertex[original_id] = len(hop_vertices[seed_type])
                            # TODO(kaili) any elegant way to do this?
                            id_list = list(hop_vertices[seed_type])
                            id_list.append(original_id)
                            hop_vertices[seed_type] = copy.deepcopy(id_list)
                        else:
                            local_id = to_local_vertex[original_id]
                        col_idx[i] = local_id
                        # original_id format (type/id), original_id_ format (id).
                        original_id_ = original_id.split("/")[-1]
                        original_ids[seed_type].update({local_id: original_id_})
                    for i, vid in enumerate(row_idx):
                        if original_id not in to_local_vertex:
                            local_id = to_local_vertex[original_id] = len(hop_vertices[nbr_type])
                            id_list = list(hop_vertices[nbr_type])
                            id_list.append(original_id)
                            hop_vertices[nbr_type] = copy.deepcopy(id_list)
                        else:
                            local_id = to_local_vertex[original_id]
                        row_idx[i] = local_id
                        original_id_ = original_id.split("/")[-1]
                        original_ids[nbr_type].update({local_id: original_id_})
                    edge = (nbr_type, edge_type, seed_type)
                    rows[edge] = row_idx if len(rows[edge]) == 0 else rows[edge].extend(row_idx)
                    cols[edge] = col_idx if len(cols[edge]) == 0 else cols[edge].extend(col_idx)

        heterodata = HeteroData()
        for edge in rows.keys():
            edge_index = set(list(zip(rows[edge], cols[edge])))
            edge_index = Tensor(list(edge_index))
            if self.sparse:
                heterodata[edge].adj = Tensor.to_sparse(edge_index)
            else:
                heterodata[edge].edge_index = edge_index

        # original id is the original name in the gdb.
        for tlabel, original_ids_by_tlabel in original_ids.items():
            heterodata[tlabel].original_id = [original_ids_by_tlabel[i] for i in range(len(original_ids_by_tlabel))]

        if self.disjoint:
            # TODO(kaili)
            pass

        self.heterodata = heterodata
        return self

    def add_features(
        self,
        vertex_feature_views: Dict[str, FeatureView],
        _edge_feature_views: Dict[str, FeatureView] = None,
    ):
        """Add features for the vertices in the subgraph.
        Parameters
        ----------
        vertex_feature_views : Dict[str, FeatureView]
            A map from vertex type label to feature view
        edge_feature_views : Dict[str, FeatureView], optional
            A map from edge type label to feature view. Unused in current implementation.
        """

        node_types, edge_types = self.heterodata.metadata()
        for node_type in node_types:
            if node_type in vertex_feature_views:
                # if node_type not in vertex_feature_views, it means there is no feature for this node type
                vertex_features = vertex_feature_views[node_type].get(self.heterodata[node_type].original_id)
                self.heterodata[node_type].x = vertex_features

        return self.heterodata
