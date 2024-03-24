from collections import defaultdict
from typing import Dict

from ofnil.infra.graph.hop_collector import HopCollector
from ofnil.infra.kv.feature_view import FeatureView


class SimpleHopCollector(HopCollector):
    def __init__(self) -> None:
        super().__init__()

    def reset_seeds(self, seeds):
        self.seeds = seeds
        self.hops = []

    def add_hop(
        self,
        row_idx: list,
        col_idx: list,
        seed_type=None,
        nbr_type=None,
        edge_type=None,
    ) -> list:
        self.hops.append((row_idx, col_idx, seed_type, nbr_type, edge_type))
        return list(set(row_idx))

    def to_local(self, id_fn):
        if len(self.hops) == 0:
            return None

        vids_by_tlabel = defaultdict(set)
        for hop in self.hops:
            row_idx, col_idx, seed_type, nbr_type, edge_type = hop
            vids_by_tlabel[seed_type].update(col_idx)
            vids_by_tlabel[nbr_type].update(row_idx)
        id_map = {tlabel: id_fn(tlabel, list(vids)) for tlabel, vids in vids_by_tlabel.items()}

        self.hop_data = []
        for hop in self.hops:
            row_idx, col_idx, seed_type, nbr_type, edge_type = hop
            to_local_vertex = {}
            hop_vertices = []  # local to original id
            for i, vid in enumerate(col_idx):
                original_id = id_map[seed_type][vid]
                assert original_id is not None, f"seed_type={seed_type}, vid={vid}, id_map={id_map}"
                if original_id not in to_local_vertex:
                    local_id = to_local_vertex[original_id] = len(hop_vertices)
                    hop_vertices.append(original_id)
                else:
                    local_id = to_local_vertex[original_id]
                col_idx[i] = local_id
            nbr_idx = len(hop_vertices)
            for i, vid in enumerate(row_idx):
                original_id = id_map[nbr_type][vid]
                if original_id not in to_local_vertex:
                    local_id = to_local_vertex[original_id] = len(hop_vertices)
                    hop_vertices.append(original_id)
                else:
                    local_id = to_local_vertex[original_id]
                row_idx[i] = local_id
            # original ids of vertices in current hop, nbr (src) index, nbr type, seed (dst) index, seed type
            self.hop_data.append((hop_vertices, nbr_idx, row_idx, nbr_type, col_idx, seed_type))
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
        hop_vertices, nbr_idx, row_idx, nbr_type, col_idx, seed_type = self.hop_data[-1]
        if nbr_type in vertex_feature_views:
            vertex_features = vertex_feature_views[nbr_type].get(hop_vertices[nbr_idx:])
            # need to be reshaped to provide a desired input tensor
            # TODO(tatiana): where do we handle null feature values? and how?
        else:
            vertex_features = None
        return vertex_features, self.hop_data
