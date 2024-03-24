from argparse import ArgumentParser, Namespace
from os import path as osp

from ofnil import *

home = osp.dirname(osp.abspath(__file__))


def register_source_graph(client: Client, graph_name: str):
    # vertices
    reviewer = vertex_entity(f"{graph_name}_reviewer", "Reviewer", "reviewerID")
    product = vertex_entity(f"{graph_name}_product", "Product", "asin")
    # edges
    also_view = edge_entity(f"{graph_name}_also_view", "alsoView", product, product, directed=False)
    also_buy = edge_entity(f"{graph_name}_also_buy", "alsoBuy", product, product, directed=False)
    is_similar_to = edge_entity(f"{graph_name}_is_similar_to", "isSimilarTo", product, product, directed=False)
    same_rates = edge_entity(f"{graph_name}_same_rates", "sameRates", reviewer, reviewer, directed=False)
    rates = edge_entity(f"{graph_name}_rates", "rates", reviewer, product, directed=True)
    # properties
    product_fields = fields(
        [("asin", "String"), ("price", "Float"), ("rank1", "Int"), ("rank2", "Int")],
        product,
    )
    reviewer_fields = fields(
        [("reviewerId", "String")],
        reviewer,
    )
    # graph
    g = client.register_source_graph(
        graph_name,
        entities=[
            reviewer,
            product,
            also_view,
            also_buy,
            is_similar_to,
            same_rates,
            rates,
        ],
        fields=product_fields + reviewer_fields,
        infra={"Neo4j": "neo4j_1"},
    )

    return g


class DemoPipeline(procedures.Procedure):
    """A demo feature transformation pipeline

    This is an example pipeline that can be part of users' feature engineering library. This pipeline
    defines certain feature logic (e.g. PageRank, triangle counting on an induced homogeneous graph)
    with configurable parameters (e.g. damping factor in PageRank). In an ideal case, a user responsible
    for data engineering creates such pipeline, and several users as data scientists can reuse the
    pipeline definition with different parameters for different tasks.
    """

    def __init__(self, context, project) -> None:
        super().__init__(context)
        self.add_procedure(
            "user_page_rank",
            procedures.BuiltIn(
                procedure="page_rank",
                name="user_page_rank",
                args={
                    "entities": [
                        f"default/Entity/{project}_reviewer",
                        f"default/Entity/{project}_same_rates/{project}_reviewer/{project}_reviewer",
                    ],
                    "target_node_entity": f"default/Entity/{project}_reviewer",
                },
            ),
        )
        self.add_procedure(
            "average_price",
            procedures.BuiltIn(
                "aggregate_neighbors",
                name="average_price",
                args={
                    "edge_entity": f"default/Entity/{project}_rates/{project}_reviewer/{project}_product",
                    "target_node_entity": f"default/Entity/{project}_reviewer",
                    "properties": ["price"],
                    "aggregator": "mean",
                },
            ),
        )
        self.add_procedure(
            "user_tc",
            procedures.BuiltIn(
                "triangle_count",
                name="user_triangle_count",
                args={
                    "entities": [
                        f"default/Entity/{project}_reviewer",
                        f"default/Entity/{project}_same_rates/{project}_reviewer/{project}_reviewer",
                    ],
                    "target_node_entity": f"default/Entity/{project}_reviewer",
                },
            ),
        )

    def construct(self, graph, configs):
        for builtin in self.children():
            builtin.update_procedure_args(configs)
        return [proc(graph) for proc in self.children()]


def get_args(argv=None):
    parser = ArgumentParser("Quickstart Feature Engineering")
    parser.add_argument("--project", type=str, default="pydemo", help="The graph name")
    parser.add_argument("--damping_factor", type=int, default=0.85, help="The damping factor for user page rank")
    parser.add_argument(
        "--max_iteration", type=int, default=20, help="The number of maximum iterations of user page rank"
    )
    parser.add_argument(
        "--tolerance", type=float, default=1e-7, help="The minimum change in scores between iterations."
    )

    return parser.parse_args(argv)


if __name__ == "__main__":
    client = Client(home)
    args = get_args()
    g = register_source_graph(client, args.project)
    print(f"registered source graph {g.name}")

    pipeline = DemoPipeline(client.new_pipeline_context(), args.project)
    dataframes = pipeline(
        g,
        {
            "user_page_rank": {
                # algorithm configs
                "damping_factor": args.damping_factor,
                "max_iteration": args.max_iteration,
                "tolerance": args.tolerance,
                # common procedure configs
                "infra": {"Neo4j": "neo4j_1"},
            },
            "average_price": {"infra": {"Neo4j": "neo4j_1"}},
            "user_triangle_count": {"infra": {"Neo4j": "neo4j_1"}},
        },
    )
    transformation_id = pipeline.finalize(fields=[(dataframes, {"Redis": "redis"})])
    print(f"registered transformation {transformation_id}")
