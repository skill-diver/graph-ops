""" Tests for :class:`ofnil.torch.neighbor_sampled_dataloader.NeighborSampledDataLoader` """

import os
import unittest

import pytest

from ofnil import Client
from ofnil.infra.graph.hop_collectors.dgl_hop_collector import DGLHopCollector
from ofnil.infra.graph.hop_collectors.pyg_hop_collector import PyGHopCollector

OFNIL_HOME = os.environ.get("OFNIL_HOME")


@unittest.skipIf(OFNIL_HOME is None, reason="Need to set OFNIL_HOME")
@pytest.mark.parametrize("topo_view_id", ["default/TopologyFeatureView/test_py_topo_view"])
@pytest.mark.parametrize("table_view_id", ["default/TableFeatureView/test_py_table_view"])
@pytest.mark.parametrize("batch_size", [1, 10])
@pytest.mark.parametrize("sparse", [False, True])
def test_neo4j_dgl_sampler_hetero(topo_view_id, table_view_id, batch_size, sparse):
    client = Client(OFNIL_HOME)
    dataloader = client.get_neighbor_sampled_dataloader(
        topo_view_id,
        [table_view_id],
        batch_size=batch_size,
        hop_collector=DGLHopCollector(sparse=sparse),
        replace=True,
        indices="Product",
        fanouts=[10, 5],
    )
    try:
        heterograph = next(iter(dataloader))
    except StopIteration:
        return
    print(heterograph)


@unittest.skipIf(OFNIL_HOME is None, reason="Need to set OFNIL_HOME")
@pytest.mark.parametrize("topo_view_id", ["default/TopologyFeatureView/test_py_topo_view"])
@pytest.mark.parametrize("table_view_id", ["default/TableFeatureView/test_py_table_view"])
@pytest.mark.parametrize("batch_size", [1, 10])
@pytest.mark.parametrize("disjoint", [False])
@pytest.mark.parametrize("sparse", [False, True])
def test_neo4j_pyg_sampler_hetero(topo_view_id, table_view_id, batch_size, disjoint, sparse):
    client = Client(OFNIL_HOME)
    dataloader = client.get_neighbor_sampled_dataloader(
        topo_view_id,
        [table_view_id],
        batch_size=batch_size,
        hop_collector=PyGHopCollector(disjoint=disjoint, sparse=sparse),
        shuffle=True,
        shuffle_buf_size=8,
        replace=True,
        num_neighbors=[10, 5],
        input_nodes="Product",
    )

    # create a dataloader without shuffle for result compare
    not_shuffle_dataloader = client.get_neighbor_sampled_dataloader(
        topo_view_id,
        [table_view_id],
        batch_size=batch_size,
        hop_collector=PyGHopCollector(disjoint=disjoint, sparse=sparse),
        shuffle=False,
        shuffle_buf_size=None,
        replace=False,
        num_neighbors=[10, 5],
        input_nodes="Product",
    )
    # get result for shuffle and not_shuffled dataloader
    not_shuffle_result_for_wapper = list((not_shuffle_dataloader.g))
    shuffle_result_for_wapper = list((dataloader.g))

    # size of these two dataloader should be the same
    assert len(not_shuffle_result_for_wapper) == len(shuffle_result_for_wapper)

    # the output turn for these two dataloader should be different
    if len(not_shuffle_result_for_wapper) > 0:
        for i in range(len(not_shuffle_result_for_wapper)):
            if not_shuffle_result_for_wapper[i] != shuffle_result_for_wapper[i]:
                break
        assert i != len(not_shuffle_result_for_wapper)

    try:
        heterograph = next(iter(dataloader))
    except StopIteration:
        return
    print(heterograph)


if __name__ == "__main__":
    pytest.main([__file__])
