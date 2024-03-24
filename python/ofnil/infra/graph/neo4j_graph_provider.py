from neo4j import GraphDatabase

from ofnil.infra.graph.graph_provider import GraphProvider
from ofnil.ofnil import TopologyFeatureViewInfo


# TODO(tatiana): a better name
class Neo4jGraphProvider(GraphProvider):
    def __init__(self, topo_info: TopologyFeatureViewInfo, seed_type: str = None):
        super().__init__(seed_type=seed_type)

        self.driver = GraphDatabase.driver(
            uri=topo_info.infra_info["uri"],
            auth=(topo_info.infra_info["username"], topo_info.infra_info["password"]),
        )

        self.seed_query = (
            f"MATCH (n:{seed_type}) RETURN ID(n) AS seed, LABELS(n)[0] as label"
            if seed_type is not None
            else "MATCH (n) RETURN ID(n) AS seed, LABELS(n)[0] as label"
        )

        self.seed_session = self.driver.session()
        self.seed_stream = None

    def __del__(self):
        self.seed_session.close()
        self.driver.close()

    def __iter__(self):
        """Implement an iterable-style dataset

        Yields
        ------
        int
            The internal id of a seed vertex
        """
        self.seed_stream = self.seed_session.run(self.seed_query)
        for record in self.seed_stream:
            yield {record["label"]: record["seed"]}

    def get_num_seeds(self):
        return self.get_num_vertices(self.seed_type)

    def get_num_vertices(self, vertex_type: str = None):
        query = (
            f"MATCH (n:{vertex_type}) RETURN count(n) as cnt"
            if vertex_type is not None
            else "MATCH (n) RETURN count(n) AS cnt"
        )
        with self.driver.session() as session:
            cnt = [record["cnt"] for record in session.run(query)][0]
        return cnt
