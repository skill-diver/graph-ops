import random
from typing import List

import torch

from ofnil.infra.graph import GraphProvider


class IterableDatasetWrapper(torch.utils.data.IterableDataset):
    def __init__(self, graph_provider: GraphProvider, shuffle: bool = False, shuffle_buffer_size=None):
        self.graph_provider = graph_provider
        self.shuffle = shuffle
        self.shuffle_buffer_size = shuffle_buffer_size

    def __iter__(self):
        if self.shuffle:
            seed_node_buf = []
            for seed_node in iter(self.graph_provider):
                if len(seed_node_buf) == self.shuffle_buffer_size:
                    idx = random.randint(0, self.shuffle_buffer_size - 1)
                    yield seed_node_buf[idx]
                    seed_node_buf[idx] = seed_node
                else:
                    seed_node_buf.append(seed_node)
            random.shuffle(seed_node_buf)
            while seed_node_buf:
                yield seed_node_buf.pop()
        else:
            for seed_node in iter(self.graph_provider):
                yield seed_node
