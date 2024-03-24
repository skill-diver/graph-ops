from abc import ABC, abstractmethod
from typing import TypeVar

from ofnil.infra.graph.hop_collector import HopCollector
from ofnil.ofnil import TopologyFeatureViewInfo

from .hop_collectors.simple_hop_collector import SimpleHopCollector


class NeighborSampler(ABC):
    """NeighborSampler for sampling k-hop neighbors of seed vertices."""

    subclasses = {}  # init from the graph module __init__ file

    def __new__(cls, topo_info: TopologyFeatureViewInfo, *args, **kwargs):
        """Factory method for NeighborSampler.

        Parameters
        ----------
        topo_view : TopologyFeatureViewInfo

        Returns
        -------
        A subclass of NeighborSampler
            Returns a sampled subgraph given a batch of seed vertices through a call to `sample_neighbors`.

        Raises
        ------
        ValueError
            Currently the supported infra is neo4j. The error is raised if topo_info.infra_info["infra_type"]
            has other values.
        """
        if cls is NeighborSampler:
            infra_type = topo_info.infra_info["infra_type"]
            if infra_type in NeighborSampler.subclasses:
                return super().__new__(NeighborSampler.subclasses[infra_type])
            else:
                raise ValueError(
                    f"Topology view resides in infra {infra_type} which does not support neighbor sampling. Support list {NeighborSampler.subclasses.keys()}"
                )
        else:
            return super().__new__(cls)

    def __init__(
        self,
        topo_info: TopologyFeatureViewInfo,
        hop_collector: TypeVar("T", bound="HopCollector"),
        replace=False,
        need_edge=False,
        **kwargs,
    ) -> None:
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

        Raises
        ------
        ValueError
        """
        # TODO(kaili): validate the fanout or num_neighbors.

        self.topo_info = topo_info
        self.sample_with_replacement = replace

        if hop_collector is None:
            self.hop_collector = SimpleHopCollector()
        else:
            self.hop_collector = hop_collector

        self.need_edge = need_edge

    @staticmethod
    def _get_edge_types(edge_types, topology_view, k):
        if edge_types is None:
            return [topology_view.get_all_edge_types()] * k
        else:
            all_edge_types = topology_view.get_all_edge_types()
            for type in edge_types:
                assert type in all_edge_types, f"Edge type {type} not available in topology view."
            return edge_types

    # TODO(tatiana): doc
    @abstractmethod
    def sample_neighbors(self, seeds) -> TypeVar("T", bound="HopCollector"):
        pass
