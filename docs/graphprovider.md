# Ofnil's GraphProvider

## Workflow of Ofnil GraphProvider

Input data of Ofnil NeighborSampledDataLoader is a tuple containing two structs, `TopologyFeatureViewInfo` and `List[TableFeatureViewInfo]].` Ofnil uses those two inputs to generate corresponding topology information and vertex/edge feature views.

After that, Ofnil calls `GraphProvider` to get information about the input graph. As Pytorch will use this information, we have designed a class called `IterableDatasetWrapper` to store data according to Pytorch format.
According to the definition in PyTorch docs, all subclasses should overwrite the  `__iter__` in python. That's why an iterator is needed for making a GraphProvider plugin. The graph provider should provide the number of nodes, the type of nodes, edges, and the relationship between the root node and its neighbors to `NeighborSampledDataLoader` for sample generation.

## How to create a GraphProvider plugin

### Example：Build a GraphProvider for Neo4j

-
  Step 1: Import the corresponding database module, GraphProvider abstract method and TopologyFeatureViewInfo.

    ```{python}
    from neo4j import GraphDatabase
    from ofnil.infra.graph.graph_provider import GraphProvider
    from ofnil.ofnil import TopologyFeatureViewInfo
    ```  

  Step 2: Create the constructor and destructor for GraphProvider class.

    ```{python}
    def __init__(self, topo_info, seed_type):
        super().__init__(seed_type) # The label for nodes within the graph.

        self.driver = GraphDatabase.driver(
            uri=,
            auth=,
        ) # The driver API from the corresponding database

        self.seed_query = (

        ) # The query to get seed_type from a graph.

        self.seed_session # The session within the chosen driver
        self.seed_stream  # This variable used to store the excuting result from seed_session.

    def __del__(self):
        self.seed_session.close()
        self.driver.close()
    ```

  Step 3: Create an iterator for GraphProvider class to get an iterable-style dataset.

    ```{python}
     def __iter__(self):
        self.seed_stream = self.seed_session.run(self.seed_query)
        for record in self.seed_stream:
        yield record["seed"]
    ```

  Step 4: Create methods that provide needed information about the graph.

    ```{python}
        def get_num_seeds(self):

        def get_num_vertices(self, vertex_type: str = None):

        def get_num_edges(self):
        
    ```
