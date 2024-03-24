import copy
from collections import defaultdict
from typing import Dict, List, Tuple, Union

import dgl
import torch

from ofnil.infra.graph.hop_collector import HopCollector
from ofnil.infra.kv.feature_view import FeatureView


class DGLHopCollector(HopCollector):
    def __init__(self, sparse=True):
        """This class supports DGL format sampler output.
        In specific, it aligns with NeighborSampler wrapped in dgl.dataloading.DataLoader [https://docs.dgl.ai/generated/dgl.dataloading.NeighborSampler.html?highlight=neighborsampler#dgl.dataloading.NeighborSampler].

        Parameters
        ----------
        sparse : bool, optional
            If set to True, the output adjacency matrix will be a sparse matrix, by default True.

        """
        self.sparse = sparse

    def reset_seeds(self, seeds):
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
        hop_list = list(zip(row_idx, col_idx, seed_type, nbr_type, edge_type))
        self.hops.append(hop_list)
        frontier = defaultdict(dict)
        for col_idx_, row_idx_, nbr_type_ in zip(col_idx, row_idx, nbr_type):
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
            self.mfgs, self.original_ids_hop = [], []
            return self

        id_map = self._get_local_id_map(id_fn)
        self.mfgs, self.original_ids_hop = [], []
        to_local_vertex = {}
        hop_vertices = defaultdict(dict)  # local to original id for each node type
        relations = defaultdict(dict)
        original_ids = defaultdict(dict)
        for hop_with_etypes in self.hops:
            for hop in hop_with_etypes:
                row_idx, col_idx, seed_type, nbr_type, edge_type = hop
                for i, vid in enumerate(col_idx):
                    original_id = id_map[seed_type][vid]
                    assert original_id is not None, f"seed_type={seed_type}, vid={vid}, id_map={id_map}"
                    if original_id not in to_local_vertex:
                        local_id = to_local_vertex[original_id] = len(hop_vertices[seed_type])
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
                    original_id = id_map[nbr_type][vid]
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
                relations[edge] = (row_idx, col_idx)
            block = dgl.create_block(relations)
            # original id is the original name in the gdb.
            original_ids_dict = defaultdict(dict)
            for tlabel, original_ids_by_tlabel in original_ids.items():
                original_ids_dict[tlabel] = [original_ids_by_tlabel[i] for i in range(len(original_ids_by_tlabel))]
            self.original_ids_hop.append(original_ids_dict)
            self.mfgs.append(block)

        return self

    def add_features(
        self,
        vertex_feature_views: Dict[str, FeatureView],
        _edge_feature_views: Dict[str, FeatureView] = None,
    ):
        """Add features for the vertices in the outermost hop

        Parameters
        ----------
        vertex_feature_views : Dict[str, FeatureView]
            A map from vertex type label to feature view
        edge_feature_views : Dict[str, FeatureView], optional
            A map from edge type label to feature view. Unused in current implementation.
        """
        for original_ids_dict, block in zip(self.original_ids_hop, self.mfgs):
            features = defaultdict(dict)
            for node_type, original_ids in original_ids_dict.items():
                if node_type in vertex_feature_views:
                    # if node_type not in vertex_feature_views, it means there is no feature for this node type
                    feature_type = vertex_feature_views[node_type].get(original_ids)
                    # TODO: remove this after we have a better way to handle missing features
                    feature_type[feature_type == None] = 0.0
                    features[node_type] = torch.Tensor(feature_type.astype(float))
            block.ndata["feature"] = features

        return self.mfgs
