# Interface comparison between Ofnil's Neighbor Sampled DataLoader with DGL and PyG

## Overview

Ofnil currently only supports the widely-utilized `NeighborSampling` method, with plans to include other sampling methods soon. Both DGL and PyG also implement the `NeighborSampling` method.

We will first examine the differences in the interfaces of the three projects, then compare their parameters, and finally provide a code snippet to easily scale up graph learning by switching from the NeighborSampling method in DGL and PyG to that of Ofnil.

## Interface comparison

The interface of the neighbor sampling class is a Dataloader which prepares data objects from the data source (stored data with their metadata in Ofnil, Dataset in PyG/DGL) to a mini-batch. Different sampling procedures are implemented in Sampler, which is used by Dataloader. The comparison of the interface name is listed:
|  | Sampler | Dataloader |
|:---|:--- |:---|
| DGL | `NeighborSampler` (Using `dgl.dataloading.as_edge_prediction_sampler(sampler)` for edge classification and link prediction)  | `dgl.dataloading.DataLoader` (Need to specify the sampler manually.) |
| PyG | `NeighborSampler` | `torch_geometric.loader.NeighborLoader` (No need to specify the sampler, which is binding to `NeighborSampler`. Using `torch_geometric.loader.NeighborLoader`, which is derived from `torch_geometric.loader.NeighborLoader`, for edge classification and link prediction.) |
| Ofnil | `NeighborSampler` | `Ofnil.torch.NeighborSampledDataLoader` (`Ofnil.torch.NeighborSampledDataLoader` is called in `Ofnil.Client.get_neighbor_sampled_dataloader()` or `Ofnil.Client.neighbor_sampled_dataloader()`, and it binds to `NeighborSampler`. Similarly, `Ofnil.torch.LinkNeighborSampledDataLoader` is called in `Ofnil.Client.get_link_neighbor_sampled_dataloader()`. we use `HopCollector` to support the output format of both DGL and PyG. We will discuss `HopCollector` later.) |

## Parameters comparison

Parameters difference between Ofnil `NeighborSampledDataLoader` and DGL `NeighborSampler` and `DataLoader` suite:

| Ofnil NeighborSampledDataLoader | DGL  NeighborSampler | DGL  DataLoader | Description|
|:---|:--- |:---|:---|
| `data(Tuple[TopologyFeatureViewInfo, List[TableFeatureViewInfo]])`      |   | `graph(DGL graph)` |Input of Ofnil `NeighborSampledDataLoader` contains topology information and graph entity information which can be used after process. <br> The input of DGL DataLoader is DGL graph that can be used directly.|
| `batch_size`| | `batch_size` |The number of seed nodes in each batch.|
| `replace`     | `replace`    |  | Whether to sample with replacement |
| `hop_collector`     |   | | The hop collector used to produce the sampled subgraph for specific output format. |
| `need_edge` |||Whether to include edge features, by default False|
| `kwargs`      |   | `kwargs`| For Ofnil, specify `fanouts` and `indices`.
| | `prefetch_labels`   |
| | `output_device`   |
| | `prefetch_node_feats`  |
| | `prefetch_edge_feats`  |
| | `fanouts`||Ofnil specify this parameter `fanouts` via `kwarg`|
| | `edge_dir`  |
| | `prob`   |
| | `mask`   |
| | `indices`||Ofnil specify this parameter `indices` via `kwarg`|
| | | `graph_sampler`|
| | | `device` |
| | | `use_ddp` |
| | | `ddp_seed` |
| | | `drop_last` |
| | | `shuffle` |
| | | `use_prefetch_thread`|
| | | `use_alternate_streams`|
| | | `pin_prefetcher` |
| | | `use_uva` |
| | | `use_cpu_worker_affinity` |
| | | `cpu_worker_affinity_cores` |

Parameters difference between Ofnil `NeighborSampledDataLoader` and PyG `NeighborLoader`

| Ofnil NeighborSampledDataLoader| PyG  NeighborLoader | Description|
|:---|:--- |:---|
|  `data(Tuple[TopologyFeatureViewInfo, List[TableFeatureViewInfo]])` |  `data(Union[Data, HeteroData, Tuple[FeatureStore, GraphStore]])` | The data structure used as input data is different for Ofnil and PyG. Input of Ofnil `NeighborSampledDataLoader` contains topology information and graph entity information which can be used after process. Data input in PyG can be used directly. |
| `batch_size`| | The number of seed nodes in each batch. In PyG, `batch_size` can pass to `torch.utils.data.DataLoader` by `kwargs`. In Ofnil, `batch_size` is required.|
| `need_edge`|   |  Whether to include edge features, by default False|
| `replace`  |  `replace`  | Whether to sample with replacement.  |
| `hop_collector`  |   | The hop collector used to produce the sampled subgraph for specific output format. |
| `kwargs`  |  `kwarg` | Pass key value pair to parent PyTorch class `torch.utils.data.DataLoader`. Usually we can pass arguments: <br>  <ul><li> `batch_size` (int): The number of indices in each batch. </li><li> `drop_last` (bool): Whether to drop the last incomplete batch.</li><li> `shuffle` (bool): Whether to randomly shuffle the indices at each epoch.</li></ul> Detail: <https://pytorch.org/docs/stable/data.html>  |
|   |  `num_neighbors` |  Ofnil can pass same parameter through `kwarg`  |
|   |  `input_nodes` |  Ofnil can pass same parameter through `kwarg` |
|   |  `input_time` |   |
|   |  `directed` |   |
|   |  `disjoint` |   |
|   |  `temporal_strategy` |   |
|   |  `time_attr` |   |
|   |  `transform` |   |
|   |  `transform_sampler_output` |   |
|   |  `is_sorted` |   |
|   |  `filter_per_worker` |   |
|   |  `neighbor_sampler` |   `neighbor_sampler` in PyG is optional. `NeighborLoader` can construct a neighbor sampler if there is no user input |

## Code Snippet for DataLoader Initialization in Ofnil, DGL and PyG

### Ofnil

* Node sampler

    ```python
        client = ofnil.Client(OFNIL_HOME)
        loader = client.get_neighbor_sampled_dataloader(
            topo_view_id,
            [table_view_id],
            sample_with_replacement=True,
            num_neighbors=[10, 5],
            batch_size=batch_size,
            hop_collector=DGLHopCollector(sparse=True), # or PygHopCollector(disjoint=disjoint, sparse=sparse),
            input_nodes="Product",
        )
        sampled_hetero_data = next(iter(loader))
    ```

* Edge sampler

    ```Python
    client = ofnil.Client(OFNIL_HOME)
    loader = client.get_link_neighbor_sampled_dataloader(
        topo_view_id,
        [table_view_id],
        sample_with_replacement=True,
        num_neighbors=[10, 5],
        batch_size=batch_size,
        hop_collector=PyGHopCollector(disjoint=disjoint, sparse=sparse),
        edge_label_index=seed_edge_types,
        edge_label=torch.ones(num_edges)
    )
    sampled_hetero_data = next(iter(loader))
    ```

### DGL

* Node Sampler

    ```python
    sampler = dgl.dataloading.NeighborSampler([5, 10, 15])
    dataloader = dgl.dataloading.DataLoader(
        g, train_nid, sampler,
        batch_size=1024, shuffle=True, drop_last=False, num_workers=4)
    for input_nodes, output_nodes, blocks in dataloader:
        train_on(blocks)
    ```

* Edge Sampler

    ```python
    sampler = dgl.dataloading.NeighborSampler([5, 10, 15])
    sampler = dgl.dataloading.as_edge_prediction_sampler(sampler)
    dataloader = dgl.dataloading.DataLoader(
        g, train_eid, sampler,
        batch_size=1024, shuffle=True, drop_last=False, num_workers=4)
    for input_nodes, output_nodes, blocks in dataloader:
        train_on(blocks)
    ```

### PyG

* Node sampler

    ```python
    sampler = dgl.dataloading.NeighborSampler([5, 10, 15])
    dataloader = dgl.dataloading.DataLoader(
        g, train_nid, sampler,
        batch_size=1024, shuffle=True, drop_last=False, num_workers=4)
    for input_nodes, output_nodes, blocks in dataloader:
        train_on(blocks)
    ```

* Edge sampler

    ```python
    from torch_geometric.loader import LinkNeighborLoader
    loader = LinkNeighborLoader(
        hetero_data,
        # Sample 30 neighbors for each node for 2 iterations
        num_neighbors=[30] * 2,
        # Use a batch size of 128 for sampling training nodes
        batch_size=128,
        edge_label_index=data.edge_index,
        # provide edge labels for sampled edges
        edge_label=torch.ones(data.edge_index.size(1))
    )
    sampled_data = next(iter(loader))
    ```

## Reference

* <https://docs.dgl.ai/api/python/dgl.dataloading.html>
* <https://pytorch-geometric.readthedocs.io/en/latest/modules/sampler.html>
