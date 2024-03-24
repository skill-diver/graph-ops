from abc import ABC, abstractmethod

from ofnil.ofnil import TopologyFeatureViewInfo


# TODO(tatiana): a better name
class GraphProvider(ABC):
    """
    NeighborSampledDataLoader will call IterableDatasetWrapper, usage for NeighborSampledDataLoader can be found in docs/dataloader.md
    IterableDatasetWrapper will call GraphProvider to provide needed info about a graph. The method for a subclass
    the plugin should provide the number of nodes, type of nodes, number of edges, and neighbors for each node.

    Parameters
    ----------
        topo_info: TopologyFeatureViewInfo
            Record topology information of a graph, including src label, dst label, edge label,
            and other topology information.
        seed_type: str
            The label for nodes within the graph.

    Attributes
    ----------
        self.driver:
            The corresponding driver from different databases
        self.seed_type:
            The label for nodes within the graph.
        self.seed_query:
            The query to get seed_type from a graph.
        self.seed_session:
            The session within the chosen driver.
        self.seed_stream:
            This variable used to store the excuting result from seed_session.

    Methods
    ----------
        get_num_seeds(self):
            get num of nodes and return it.
        get_num_edges(self):
            get num of edges and return it.
        get_node_neighbors(self):
            return the neighbors nodes for each nodes.

    """

    subclasses = {}  # init from the graph module __init__ file

    def __new__(cls, topo_info: TopologyFeatureViewInfo, *args, **kwargs):
        """Factory method for GraphProvider.

        Parameters
        ----------
        topo_view : TopologyFeatureViewInfo

        Returns
        -------
        A subclass of GraphProvider
            An iterator that returns a sampled subgraph at each iteration.

        Raises
        ------
        ValueError
            Currently the supported infra is neo4j. The error is raised if topo_view.infra_info["infra_type"]
            has other values.
        """
        if cls is GraphProvider:
            infra_type = topo_info.infra_info["infra_type"]
            if infra_type in GraphProvider.subclasses:
                return super().__new__(GraphProvider.subclasses[infra_type])
            else:
                raise ValueError(
                    f"Topology view resides in infra {infra_type} which does not support neighbor sampling. Support list {GraphProvider.subclasses.keys()}"
                )
        else:
            return super().__new__(cls)

    # TODO(tatiana): support filtering by property (e.g. training mask)
    @abstractmethod
    def get_num_vertices(self, vertex_type: str):
        pass

    def __len__(self):
        return self.get_num_vertices(self.seed_type)

    def __init__(self, seed_type: str = None):
        self.seed_type = seed_type
