from abc import ABC, abstractmethod
from typing import Dict, List, Tuple, Union

from torch import Tensor

from ofnil.infra.kv.feature_view import FeatureView


class HopCollector(ABC):
    @abstractmethod
    def reset_seeds(self, seeds: Tuple[str, Union[Tensor, List[int]]]):
        """Add the indices of nodes for which neighbors are sampled to create mini-batches.
        `seed` is similar to `indices` DGL DataLoader and `input_nodes` in PyG NeighborLoader.
        However, `seeds` is exactly the root nodes of the sampled subgraph, while `indices` and
        `input_nodes` are the candidate root nodes which needs to sample the root nodes for each minibatch first.
        """

    @abstractmethod
    def add_hop(
        self,
        row_idx: list,
        col_idx: list,
        seed_type: str | list = None,
        nbr_type: str | list = None,
        edge_type: str | list = None,
    ) -> list:
        """Add a sampled neighbor-to-seed (sub)graph in COO format as a hop

        Parameters
        ----------
        row_idx : list
            The row indices of edges
        col_idx : list
            The column indicies of edges
        seed_type : str, optional
            The type label of seed vertices. None if sampling on a homogeneous graph.
        nbr_type : str, optional
            The type label of neighbor vertices. None if sampling on a homogeneous graph.
        edge_type : str, optional
            The type label of the edge. None if sampling on a homogeneous graph.

        Returns
        -------
        list
            The seed vertices for the next hop to be sampled
        """

    # TODO(tatiana): doc
    @abstractmethod
    def to_local(self, id_fn):
        pass

    @abstractmethod
    def add_features(
        self,
        vertex_feature_views: Dict[str, FeatureView],
        edge_feature_views: Dict[str, FeatureView] = None,
    ):
        """Joins the sampled vertices with their corresponding features,
        returning the result to be used downstream.

        Parameters
        ----------
        vertex_feature_views : Dict[str, FeatureView]
            A map from vertex type label to feature view
        edge_feature_views : Dict[str, FeatureView], optional
            A map from edge type label to feature view.
        """
