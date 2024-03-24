""" Tests for :class:`ofnil.infra.graph.neo4j_neighbor_sampler` """

import os
from collections import defaultdict

import pytest

from ofnil.infra.graph.hop_collectors.pyg_hop_collector import PyGHopCollector
from ofnil.infra.graph.neo4j_graph_provider import Neo4jGraphProvider
from ofnil.infra.graph.neo4j_neighbor_sampler import Neo4jNeighborSampler
from ofnil.ofnil import ClientInner

OFNIL_HOME = os.environ.get("OFNIL_HOME")


@pytest.mark.parametrize("topo_view_id", ["default/TopologyFeatureView/test_py_topo_view"])
@pytest.mark.parametrize("batch_size", [1, 10])
@pytest.mark.parametrize("disjoint", [False])
@pytest.mark.parametrize("sparse", [False, True])
def test_neo4j_pyg_sampler_hetero(topo_view_id, batch_size, disjoint, sparse):
    client = ClientInner(OFNIL_HOME)
    topo_view = client.get_topology_view(topo_view_id)
    g = Neo4jGraphProvider(topo_view, seed_type="Product")

    sampler = Neo4jNeighborSampler(
        topo_view,
        num_neighbors=[10, 5],
        replace=True,
        hop_collector=PyGHopCollector(disjoint=disjoint, sparse=sparse),
        input_nodes="Product",
    )

    seed_iter = iter(g)
    for _ in range(0, 5):
        try:
            seeds = [next(seed_iter) for _ in range(batch_size)]
            seeds_dict = defaultdict(list)
            for item in seeds:
                for key, value in item.items():
                    seeds_dict[key].append(value)
            seeds = dict(seeds_dict)
        except StopIteration:
            break
        hop_collector = sampler.sample_neighbors(seeds)

        print(hop_collector.heterodata)


if __name__ == "__main__":
    pytest.main([__file__])
