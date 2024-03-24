"""
This module provides interfaces and implementations of connectors for graph data storages and engines
"""
from .graph_provider import GraphProvider
from .hop_collector import HopCollector
from .hop_collectors import *
from .neighbor_sampler import NeighborSampler
from .neo4j_graph_provider import Neo4jGraphProvider
from .neo4j_neighbor_sampler import Neo4jNeighborSampler

interfaces = [
    "GraphProvider",
    "HopCollector",
    "NeighborSampler",
]

implementations = [
    "PyGHopCollector",
    "Neo4jNeighborSampler",
    "Neo4jGraphProvider",
]

__all__ = interfaces + implementations

NeighborSampler.subclasses = {"neo4j": Neo4jNeighborSampler}
GraphProvider.subclasses = {"neo4j": Neo4jGraphProvider}
