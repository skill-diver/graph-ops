from typing import List, Union

from ofnil.infra.graph import HopCollector
from ofnil.infra.kv.feature_view import FeatureView
from ofnil.ofnil import (
    ClientInner,
    PipelineContext,
    TableFeatureViewInfo,
    TopologyFeatureViewInfo,
)


class Client:
    def __init__(self, ofnil_home):
        self.client = ClientInner(ofnil_home)

    def new_pipeline_context(self) -> PipelineContext:
        return PipelineContext(self.client)

    def register_source_graph(self, graph_name: str, entities, fields, infra: dict, variant=None):
        return self.client.register_graph(graph_name, entities, fields, infra, variant)

    def get_feature_view(self, view_id: str):
        return FeatureView(self.client.get_feature_view(view_id))

    def get_topology_view(self, view_id: str):
        # TODO(tatiana): wrap the result with a topo retrieval interface?
        return self.client.get_topology_view(view_id)

    def neighbor_sampled_dataloader(self, graph_dataset: str, **kwargs):
        (topos, tables) = self.client.get_graph_dataset(graph_dataset)
        assert len(topos) == 1
        return self.get_neighbor_sampled_dataloader(topo=topos[0], features=tables, **kwargs)

    # TODO(tatiana): integrate with DGL/PyG dataloader API
    def get_neighbor_sampled_dataloader(
        self,
        topo: str | TopologyFeatureViewInfo,
        features: List[str | TableFeatureViewInfo],
        batch_size: int,
        backend: str = "torch",
        hop_collector: Union[str, HopCollector] = None,
        shuffle: bool = False,
        shuffle_buf_size: int = None,
        **kwargs,
    ):
        """Creates a NeighborSampledDataLoader

        Examples
        ----------
        .. code-block:: python

            import ofnil

            client = ofnil.Client("/path/to/ofnil/home")
            loader = client.get_neighbor_sampled_dataloader(
                topo="default/TopologyFeatureView/test_py_topo_view",
                features=[],
                batch_size=10,
                fan_outs=[10, 5],
                input_nodes="Reviewer",
                edge_types=["sameRates", "rates"],
            )
            batch = next(iter(loader))
            print(len(batch[1]))
            # >>> 2

        Parameters
        ----------
        topo : str
            The resource id of the topology feature view to be used in the data loader.
        features : List[str]
            A list of resource ids of the table feature views to be used in the data loader.
        batch_size : int
            The batch size of the data loader.
        backend : str, optional
            The backend of the NeighborSampledDataLoader to be constructed, by default "torch"
        hop_collector : TypeVar("T", bound="HopCollector")
            The hop collector used to produce the sampled subgraph for specific output format.
        kwargs : dict
            Additional arguments to be passed to the NeighborSampledDataLoader.

        Returns
        -------
        NeighborSampledDataLoader
        """
        topo_info = self.client.get_topology_view(topo) if type(topo) is str else topo
        feature_views = [
            self.client.get_feature_view(feature) if type(feature) is str else feature for feature in features
        ]

        if hop_collector == "pyg":
            raise RuntimeError("To be implemented")
        elif hop_collector == "dgl":
            raise RuntimeError("To be implemented")
        elif hop_collector is not None and isinstance(hop_collector, str):
            raise ValueError(f"Unsupported hop_collector {hop_collector}")

        if backend == "torch":
            try:
                import torch as _
            except ModuleNotFoundError:
                raise ValueError(f"Backend {backend} is chosen, but torch is not installed")

            from ofnil.torch import NeighborSampledDataLoader

            return NeighborSampledDataLoader(
                data=(topo_info, feature_views),
                batch_size=batch_size,
                hop_collector=hop_collector,
                shuffle=shuffle,
                shuffle_buffer_size=shuffle_buf_size,
                **kwargs,
            )
        else:
            raise ValueError(f"Unsupported backend {backend}")
