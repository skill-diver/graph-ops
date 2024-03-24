from collections import defaultdict
from typing import List, Tuple, TypeVar

import torch

from ofnil.infra.graph.graph_provider import GraphProvider
from ofnil.infra.graph.hop_collectors.dgl_hop_collector import DGLHopCollector
from ofnil.infra.graph.hop_collectors.pyg_hop_collector import PyGHopCollector
from ofnil.infra.graph.neighbor_sampler import NeighborSampler
from ofnil.infra.kv.feature_view import FeatureView
from ofnil.ofnil import TableFeatureViewInfo, TopologyFeatureViewInfo
from ofnil.torch.dataset_wrapper import IterableDatasetWrapper


class NeighborSampledDataLoader(torch.utils.data.DataLoader):
    # TODO(tatiana): doc

    def __init__(
        self,
        data: Tuple[TopologyFeatureViewInfo, List[TableFeatureViewInfo]],
        batch_size: int,
        hop_collector: TypeVar("T", bound="HopCollector"),
        replace: bool = False,
        need_edge: bool = False,
        shuffle: bool = False,
        shuffle_buffer_size: int = None,
        **kwargs,
    ) -> None:
        """
        Parameters
        ----------
        data : Tuple[TopologyFeatureViewInfo, List[TableFeatureViewInfo]]
            The stored data and metadata in Ofnil.
            TopologyFeatureViewInfo:
                    Record topology information of a graph, including src label, dst label, edge label,
                    and other topology information
            List[TableFeatureViewInfo]:
                    Record graph entity information, field of entities,etc
        batch_size : int
            The number of seed nodes in each batch.
        hop_collector : TypeVar("T", bound="HopCollector")
            The hop collector used to produce the sampled subgraph for specific output format.
        replace : bool, optional
            Whether to sample with replacement, by default False
        need_edge : bool, optional
            Whether to include edge features, by default False
        kwargs : dict
            Additional parameters for the NeighborSampler.
        """
        topo_info, feature_views = data
        # print(f"len(feature_views) = {len(feature_views)}")

        self.vertex_feature_views = {
            view.entity_label: FeatureView(view) for view in feature_views if view.entity_type == "vertex"
        }
        self.edge_feature_views = {
            view.entity_label: FeatureView(view) for view in feature_views if view.entity_type == "edge"
        }

        # TODO(tatiana): check whether edge feature views are included
        need_edge = False

        self.neighbor_sampler = NeighborSampler(
            topo_info,
            replace=replace,
            hop_collector=hop_collector,
            need_edge=need_edge,
            **kwargs,
        )

        # TODO(kaili): It works only for now, since we only give str for the seed type.
        # TODO(kaili): handle the cases when the seed type include the list of ids, link prediction, and homogeneous sampling.
        if isinstance(hop_collector, DGLHopCollector):
            assert "indices" in kwargs, "indices must be specified for DGLHopCollector"
            seed_type = kwargs["indices"]
        elif isinstance(hop_collector, PyGHopCollector):
            assert "input_nodes" in kwargs, "input_nodes must be specified for PyGHopCollector"
            seed_type = kwargs["input_nodes"]
        else:
            raise NotImplementedError(f"hop_collector {hop_collector} is not supported!")

        self.g = IterableDatasetWrapper(GraphProvider(topo_info, seed_type=seed_type), shuffle, shuffle_buffer_size)

        super().__init__(
            self.g,
            collate_fn=self.collate_fn,
            batch_size=batch_size,
        )

    def collate_fn(self, seeds):
        """Samples a subgraph from a batch of input nodes."""
        seeds_dict = defaultdict(list)
        for item in seeds:
            for key, value in item.items():
                seeds_dict[key].append(value)
        seeds = dict(seeds_dict)
        hops = self.neighbor_sampler.sample_neighbors(seeds)
        return hops.add_features(self.vertex_feature_views, self.edge_feature_views)
