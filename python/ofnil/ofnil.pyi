# TODO: GraphDatasetRenderingOptions
# TableFeatureViewInfo, TopologyFeatureViewInfo, neighbor_sample, FeatureServingOutputType, fields
import datetime
from enum import Enum
from typing import Any, Dict, List, Tuple

ResourceId = str
Variant = str

class VertexEntity:
    name: str
    tlabel: str
    primary_key: str
    variant: Variant
    def resource_id(self) -> ResourceId: ...

class EdgeEntity:
    name: str
    tlabel: str
    src_tlabel: str
    dst_tlabel: str
    src_entity_id: str
    dst_entity_id: str
    directed: bool
    primary_key: str | None
    variant: Variant
    def resource_id(self) -> ResourceId: ...

class Entity(Enum):
    Vertex: VertexEntity
    Edge: EdgeEntity

def vertex_entity(
    name: str,
    tlabel: str,
    primary_key: str,
    variant: str | None,
) -> Entity: ...
def edge_entity(
    name: str,
    tlabel: str,
    src_entity: VertexEntity,
    dst_entity: VertexEntity,
    primary_key: str | None,
    variant: str | None,
    directed: bool | None,
) -> Entity: ...

InfraIdentifier = Dict[str, str]

class Graph:
    name: str
    variant: Variant
    description: str | None
    entity_ids: Dict[str, ResourceId]
    tags: Dict[str, str]
    owners: List[str]
    sink_infra_id: InfraIdentifier | None

FeatureValueType = str

class Field:
    name: str
    variant: Variant
    value_type: FeatureValueType
    entity_id: str | None
    transformation_id: ResourceId | None
    description: str | None
    tags: Dict[str, str]
    owners: List[str]
    sink_infra_id: InfraIdentifier | None

class ClientInner:
    def __init__(self, ofnil_home: str | None) -> None:
        """A constructor of the ClientInner object."""
        ...
    def register_graph(
        self,
        graph_name: str,
        entities: List[Entity],
        fields: List[Field],
        infra: InfraIdentifier | None,
        variant: str | None,
    ) -> Graph:
        """Register a new graph into the feature registry.

        Args:
            graph_name: A name of the new graph.

            entities: A list of the graph's entity (include Vertex and/or Edge).

            fields: A list of fields associated with the given entities.

        Returns:
            A graph's object.
        """
        ...
    def get_graph_dataset(
        self,
        graph_id: ResourceId,
    ) -> Tuple[TopologyFeatureViewInfo, TableFeatureViewInfo]:
        """Get the information of the feature views contained in the graph dataset specified by the resource ID.

        Args:
            graph_id: a target's ResourceId.

        Returns:
            Topology and table feature views of the graph.
        """
        ...
    def get_feature_view(
        self,
        view_id: ResourceId,
    ) -> TableFeatureViewInfo:
        """Get a table feature of a graph from a specific ResourceId.

        Args:
            view_id: a target's ResourceId.

        Returns:
            A table freature of the graph.
        """
        ...
    def get_topology_view(
        self,
        view_id: ResourceId,
    ) -> TopologyFeatureViewInfo:
        """Get a topology freature of a graph from a specific ResourceId.

        Args:
            view_id: a target's ResourceId.

        Returns:
            A topology freature of the graph.
        """
        ...

class TableFeatureViewInfo:
    entity_label: str
    primary_key: str
    field_names: List[str]
    entity_type: str
    rendering_opt: FeatureRenderingOptions
    infra_info: Dict[str, str]

class TopologyFeatureViewInfo:
    topologies: List[Tuple[Entity, Entity, Entity, Topology]]
    rendering_opt: TopologyRenderingOptions
    infra_info: Dict[str, str]

def neighbor_sample(
    input_vertices: List[int],
    col_offsets: List[int],
    row: List[int],
    num_samples: int,
    replace: bool,
) -> Tuple[List[int], List[int], List[int]]: ...
def fields(
    name_types: List[Tuple[str, FeatureValueType]],
    entity: Entity,
    variant: str | None,
) -> List[Field]: ...

class FeatureServingOutputType(Enum):
    NdArray: str

# added on

class TopologyServingLayout(Enum):
    CompressedSparseRow: str

class ServingMode(Enum):
    PythonBinding: str
    ProtoBuf: str
    File: str

class FeatureRenderingOptions:
    output_type: FeatureServingOutputType
    mode: ServingMode
    def __init__(output_type: FeatureServingOutputType) -> FeatureRenderingOptions: ...

class TopologyRenderingOptions:
    layout: TopologyServingLayout
    mode: ServingMode
    def __init__(output_type: TopologyServingLayout) -> TopologyRenderingOptions: ...

class Topology:
    name: str
    transformation_id: str | None
    topology_type: TopologyType | None
    edge_entity_id: ResourceId | None
    src_node_entity_id: ResourceId | None
    dst_node_entity_id: ResourceId | None
    variant: Variant
    description: str | None
    created_at: datetime.datetime | None
    tags: Dict[str, str]
    owners: List[str]
    sink_infra_id: InfraIdentifier | None

class TopologyType(Enum):
    AdjacencyList: str
    AdjacencyMatrix: str
    BipartiteGraphChain: str
