from typing import Dict, List, Tuple, TypeVar, Union

from neo4j import GraphDatabase
from torch import Tensor

from ofnil.infra.graph.hop_collectors.dgl_hop_collector import DGLHopCollector
from ofnil.infra.graph.hop_collectors.pyg_hop_collector import PyGHopCollector
from ofnil.infra.graph.neighbor_sampler import NeighborSampler
from ofnil.ofnil import TopologyFeatureViewInfo, neighbor_sample


class Neo4jNeighborSampler(NeighborSampler):
    """Sample neighboring edges of the given seed nodes from Neo4j graph database and return the induced subgraph.
    In specific, the return format is similar to NeighborSampler for **heterogenuous graph** in DGL/PyG.
    """

    def __init__(
        self,
        topo_info: TopologyFeatureViewInfo,
        hop_collector: TypeVar("T", bound="HopCollector"),
        replace=False,
        need_edge=False,
        **kwargs,
    ):
        """Constructor

        Parameters
        ----------
        topo_info : TopologyFeatureViewInfo
        hop_collector : HopCollector
            We support two types of hop collectors: DGLHopCollector and HopCollector.
            The output format of DGLHopCollector is compatible with DGL's NeighborSampler,
            and the output format of PyGHopCollector is compatible with PyG's NeighborSampler.
        replace : bool, optional
            Whether the sampling is with replacement.
            By default False
        need_edge : bool, optional
            Whether the sampled subgraph needs edge information.
            By default False
        kwargs : Dict
            The parameters for ofnil.infra.graph.neighbor_sampler.NeighborSampler.
            The is for using the same interface to support the different interfaces in DGL and PyTorch Geometric when specifying the neighbor sampler.
            In DGL, the required parameters include `fanout`, `indices`.
            In PyG, the required parameters include `num_neighbors`, `input_nodes`.

        Raises
        ------
        ValueError
        """
        super().__init__(
            topo_info,
            hop_collector=hop_collector,
            replace=replace,
            need_edge=need_edge,
        )

        if isinstance(hop_collector, PyGHopCollector):
            """required parameters for PyGHopCollector:
            ----------
            num_neighbors: Union[
                List[int], Dict[Tuple[str, str, str], List[int]]
            ] = None,
            input_nodes: Union[Tensor, None, str, Tuple[str, Optional[Tensor]]] = None.
            """
            assert "num_neighbors" in kwargs, "num_neighbors should be provided for PyGHopCollector."
            assert "input_nodes" in kwargs, "input_nodes should be provided for PyGHopCollector."
            num_neighbors = kwargs["num_neighbors"]
            seed_type = kwargs["input_nodes"]
        elif isinstance(hop_collector, DGLHopCollector):
            """required parameters for DGLHopCollector:
            ----------
            fanouts: Union[List[int], List[Dict[etype, int]]),
                    List of neighbors to sample per edge type for each GNN layer, with the i-th element being the fanout for the i-th GNN layer.
                    If only a single integer is provided, DGL assumes that every edge type will have the same fanout.
                    If -1 is provided for one edge type on one layer, then all inbound edges of that edge type will be included.
            indices: Union[Tensor, Dict[ntype, Tensor]],
            """
            assert "fanouts" in kwargs, "fanouts should be provided for DGLHopCollector."
            assert "indices" in kwargs, "indices should be provided for DGLHopCollector."
            # We use the same name as PyG's NeighborSampler.
            num_neighbors = kwargs["fanouts"]
            seed_type = kwargs["indices"]
        else:
            raise ValueError("Unsupported hop_collector.")

        if type(num_neighbors) == list:
            self.fan_outs = num_neighbors
            all_edge_types = topo_info.get_all_edge_types_triplet()
        elif type(num_neighbors) == dict:
            self.fan_outs = num_neighbors.values()
            all_edge_types = num_neighbors.keys()
        else:
            raise ValueError("num_neighbors should be list or dict")

        self.edge_types, self.dst_types = [], []
        # We assume the seed_type is a str for now.
        # frontier is the destination nodes.
        frontier_type = [seed_type]
        for _ in range(len(self.fan_outs)):
            src_types, dst_types, edge_types = [], [], []
            for edge_type in all_edge_types:
                if edge_type[1] in frontier_type:
                    edge_types.append(edge_type[2])
                    dst_types.append(edge_type[1])
                    src_types.append(edge_type[0])
            frontier_type = src_types
            self.edge_types.append(edge_types)
            self.dst_types.append(dst_types)

        self.driver = GraphDatabase.driver(
            uri=topo_info.infra_info["uri"],
            auth=(topo_info.infra_info["username"], topo_info.infra_info["password"]),
        )

        if type(num_neighbors) == list:
            self.fan_outs = num_neighbors
            all_edge_types = topo_info.get_all_edge_types_triplet()
        elif type(num_neighbors) == dict:
            self.fan_outs = num_neighbors.values()
            all_edge_types = num_neighbors.keys()
        else:
            raise ValueError("num_neighbors should be list or dict")

        self.edge_types, self.dst_types = [], []
        # We assume the seed_type is a str for now.
        # frontier is the destination nodes.
        frontier_type = [seed_type]
        for _ in range(len(self.fan_outs)):
            src_types, dst_types, edge_types = [], [], []
            for edge_type in all_edge_types:
                if edge_type[1] in frontier_type:
                    edge_types.append(edge_type[2])
                    dst_types.append(edge_type[1])
                    src_types.append(edge_type[0])
            frontier_type = src_types
            self.edge_types.append(edge_types)
            self.dst_types.append(dst_types)

        # deduplicate multi-edges of the same type for now
        if need_edge:
            # TODO(tatiana): support edge prop handling
            # TODO(tatiana): use APOC agg.first?
            self.neighbor_query = """MATCH (seed)-[e]-(nbr) WHERE ID(seed) in $seed_ids and type(e)=$edge_type
                                    WITH seed, nbr, collect(e)[0] as edge ORDER BY seed
                                    RETURN ID(seed) as seed, ID(nbr) as nbr, labels(nbr)[0] as nbr_label, ID(edge) as edge"""
        else:
            self.neighbor_query = """MATCH (seed)-[e]-(nbr) WHERE ID(seed) in $seed_ids and type(e)=$edge_type
                                    WITH DISTINCT seed, nbr ORDER BY seed
                                    RETURN ID(seed) as seed, ID(nbr) as nbr, labels(nbr)[0] as nbr_label"""

        self.vertex_primary_keys = topo_info.get_vertex_primary_keys()

        # TODO(tatiana): support rendering opt in topo_info

    def __del__(self):
        self.driver.close()

    def _get_vertex_primary_keys(self, tlabel, internal_ids: list):
        if tlabel not in self.vertex_primary_keys:
            return internal_ids
        assert tlabel in self.vertex_primary_keys
        assert internal_ids is not None
        assert len(internal_ids) > 0
        primary_key = self.vertex_primary_keys[tlabel]
        id_query = (
            f"MATCH(v) WHERE ID(v) in $ids RETURN ID(v) as internal, v.{primary_key} as id, LABELS(v)[0] as label"
        )
        with self.driver.session() as id_session:
            id_stream = id_session.run(id_query, parameters={"ids": internal_ids})
            # format: {internal_id: "{type}/{id}"}
            return {record["internal"]: "{}/{}".format(record["label"], record["id"]) for record in id_stream}

    def sample_neighbors(self, seeds: Tuple[str, Union[Tensor, List[int]]]):
        frontier = seeds
        self.hop_collector.reset_seeds(seeds)

        for fanout, dst_types_hop, edge_types in zip(self.fan_outs, self.dst_types, self.edge_types):
            row_idx_hop, col_idx_hop, nbr_types_hop = [], [], []
            for idx, edge_type in enumerate(edge_types):
                frontier_nodes = frontier[dst_types_hop[idx]]
                # Get neighbors by edge_type in CSC format, i.e. nbr (src row) to frontier (dst col) adj matrix
                frontier_nodes, col_offsets, row, nbr_type = self._construct_csc(frontier_nodes, edge_type)
                # Get sampling result in COO format. The i-th edge is from row_idx[i] to col_idx[i], whose index in the input CSC is edge_index[i]
                # now the row_idx and col_idx are the internal neo4j IDs
                row_idx, col_idx, _edge_indices = neighbor_sample(
                    frontier_nodes,
                    col_offsets,
                    row,
                    fanout,
                    self.sample_with_replacement,
                )
                row_idx_hop.append(row_idx)
                col_idx_hop.append(col_idx)
                nbr_types_hop.append(nbr_type)

            if nbr_type is not None:
                # handle the case that all the vertexs all isolated.
                frontier = self.hop_collector.add_hop(
                    row_idx_hop,
                    col_idx_hop,
                    seed_type=dst_types_hop,
                    nbr_type=nbr_types_hop,
                    edge_type=edge_types,
                )
            else:
                break

        return self.hop_collector.to_local(self._get_vertex_primary_keys)

    def _construct_csc(self, frontier, edge_type):
        assert not self.need_edge, "Unsupported yet"
        with self.driver.session() as neighbor_session:
            edge_stream = neighbor_session.run(
                self.neighbor_query,
                parameters={"seed_ids": frontier, "edge_type": edge_type},
            )

            col_offsets = [0]
            row = []
            nbr_type = None  # TODO(tatiana): get nbr type and edge direction info if edge_type is specific
            current_seed = None
            seeds = []
            for record in edge_stream:
                seed, nbr, nbr_label = record
                # check
                if nbr_type is not None:
                    assert nbr_type == nbr_label, "All sampled vertices in the same hop should be of the same type"
                else:
                    nbr_type = nbr_label
                # construct CSC
                if seed == current_seed:
                    row.append(nbr)
                else:
                    if current_seed != None:
                        col_offsets.append(len(row))
                    current_seed = seed
                    seeds.append(seed)
                    row.append(nbr)
            if current_seed != None:
                col_offsets.append(len(row))
                current_seed = seed
            # handle vertices in frontier that are without nbrs
            if len(seeds) < len(frontier):
                seed_set = set(seeds)
                for v in frontier:
                    if v not in seed_set:
                        seeds.append(v)
                        col_offsets.append(col_offsets[-1])
            assert (
                len(col_offsets) == len(frontier) + 1
            ), f"len(col_offsets) != len(frontier): {len(col_offsets)} vs {len(frontier)}, seeds={seeds}, frontier={frontier}"
            return (seeds, col_offsets, row, nbr_type)
