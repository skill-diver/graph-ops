use std::collections::HashMap;

use crate::{
    feature::ResourceId,
    infra::pi::QueryParserError,
    transformation::{GraphSchemaEntity, InputSchema},
    FeatureValueType,
};

type IdentifierIndex = usize;

#[derive(Debug)]
pub(super) enum Identifier {
    Id(IdPropIdentifier),
    Prop(IdPropIdentifier),
    Vertex(VertexEdgeIdentifier),
    Edge(VertexEdgeIdentifier),
}

#[derive(Debug)]
pub(crate) struct IdPropIdentifier {
    index: IdentifierIndex,
    origin: IdentifierIndex,
    value_type: Option<FeatureValueType>,
}

impl IdPropIdentifier {
    pub(super) fn set_type(&mut self, value_type: FeatureValueType) {
        self.value_type = Some(value_type);
    }

    pub(super) fn get_type(&self) -> &Option<FeatureValueType> {
        &self.value_type
    }
}

#[derive(Debug)]
pub(super) struct VertexEdgeIdentifier {
    index: IdentifierIndex,
    label: Option<String>,
    entity: Option<ResourceId>,
    fields: HashMap<String, FeatureValueType>,
}

impl VertexEdgeIdentifier {
    fn new(index: IdentifierIndex) -> Self {
        Self {
            index,
            label: None,
            entity: None,
            fields: HashMap::new(),
        }
    }

    pub(super) fn set_label(&mut self, label: &str) {
        self.label = Some(label.to_string());
    }

    pub(super) fn check_needed_property(
        &self,
        identifier: &str,
        prop: &str,
    ) -> Result<&FeatureValueType, QueryParserError> {
        self.label.as_ref().unwrap_or_else(|| {
            panic!(
                "check_needed_property should be called only if label is known. {identifier}.{prop} of {self:?}"
            )
        });
        if let Some(res) = self.fields.get(prop) {
            Ok(res)
        } else {
            Err(QueryParserError::UnsupportedQuery(format!(
                "{}.{} is not an available field {:?}",
                identifier, prop, &self
            )))
        }
    }

    pub(super) fn get_schema_with_fields(&self, input_schema: &InputSchema) -> GraphSchemaEntity {
        if let Some(tlabel) = &self.label {
            let (entity_id, fields) = input_schema
                .get_vertex_schema(tlabel)
                .expect("Should have passed validate_query()");
            GraphSchemaEntity {
                tlabel: self.label.clone(),
                entity_id: Some(entity_id.clone()),
                fields: fields.clone().into_iter().collect(),
            }
        } else {
            GraphSchemaEntity {
                tlabel: None,
                entity_id: None,
                fields: Vec::new(),
            }
        }
    }

    pub(super) fn get_vertex_schema(&self, input_schema: &InputSchema) -> GraphSchemaEntity {
        GraphSchemaEntity {
            tlabel: self.label.clone(),
            entity_id: self.label.as_ref().map(|tlabel| {
                input_schema
                    .get_vertex_schema(tlabel)
                    .unwrap_or_else(|| {
                        panic!(
                            "Should have passed validate_query(). Cannot find {tlabel} in {input_schema:?}",
                        )
                    })
                    .0
                    .clone()
            }),
            fields: Vec::new(),
        }
    }

    pub(super) fn get_edge_schema(&self, input_schema: &InputSchema) -> GraphSchemaEntity {
        GraphSchemaEntity {
            tlabel: self.label.clone(),
            entity_id: self.label.as_ref().map(|tlabel| {
                input_schema
                    .get_edge_schema(tlabel)
                    .unwrap_or_else(|| {
                        panic!(
                            "Should have passed validate_query(). Cannot find {tlabel} in {input_schema:?}"
                        )
                    })
                    .0
                    .clone()
            }),
            fields: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct IdentifierMap {
    identifier_map: HashMap<String, IdentifierIndex>, // name, index
    seen_identifiers: Vec<Identifier>,
    edge_map: HashMap<(IdentifierIndex, IdentifierIndex), Vec<IdentifierIndex>>, // (src, dst), edge
}

impl IdentifierMap {
    pub(super) fn new() -> Self {
        IdentifierMap {
            seen_identifiers: Vec::new(),
            identifier_map: HashMap::new(),
            edge_map: HashMap::new(),
        }
    }

    pub(super) fn add_prop_identifier(
        &mut self,
        name: &str,
        origin: IdentifierIndex,
    ) -> &mut IdPropIdentifier {
        let index = self.seen_identifiers.len();
        self.identifier_map.insert(name.to_string(), index);
        self.seen_identifiers
            .push(Identifier::Prop(IdPropIdentifier {
                index,
                origin,
                value_type: None,
            }));
        self.seen_identifiers.last_mut().unwrap().prop_mut()
    }

    pub(super) fn add_id_identifier(
        &mut self,
        name: &str,
        origin: IdentifierIndex,
    ) -> &mut IdPropIdentifier {
        let index = self.seen_identifiers.len();
        self.identifier_map.insert(name.to_string(), index);
        self.seen_identifiers.push(Identifier::Id(IdPropIdentifier {
            index,
            origin,
            value_type: None,
        }));
        self.seen_identifiers.last_mut().unwrap().id()
    }

    pub(super) fn add_edge_identifier(
        &mut self,
        name: &str,
        src_identifier: &str,
        dst_identifier: &str,
    ) -> &mut Identifier {
        self.edge_map
            .entry((
                self.identifier_map
                    .get(src_identifier)
                    .expect("Src identifier should be in map")
                    .to_owned(),
                self.identifier_map
                    .get(dst_identifier)
                    .expect("Src identifier should be in map")
                    .to_owned(),
            ))
            .or_default()
            .push(self.seen_identifiers.len());
        let index = self.seen_identifiers.len();
        self.identifier_map.insert(name.to_string(), index);
        self.seen_identifiers
            .push(Identifier::Edge(VertexEdgeIdentifier::new(index)));
        self.seen_identifiers.last_mut().unwrap()
    }

    pub(super) fn add_vertex_identifier(&mut self, name: &str) -> &mut Identifier {
        let index = self.seen_identifiers.len();
        self.identifier_map.insert(name.to_string(), index);
        self.seen_identifiers
            .push(Identifier::Vertex(VertexEdgeIdentifier::new(index)));
        self.seen_identifiers.last_mut().unwrap()
    }

    pub(super) fn get_unique_edge(
        &self,
        src_index: IdentifierIndex,
        dst_index: IdentifierIndex,
    ) -> Option<&Identifier> {
        if let Some(edge_index) = self.edge_map.get(&(src_index, dst_index)) {
            if edge_index.len() == 1 {
                Some(&self.seen_identifiers[*edge_index.first().unwrap()])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(super) fn get_identifier(&self, identifier: &str) -> Option<&Identifier> {
        if let Some(index) = self.identifier_map.get(identifier) {
            Some(&self.seen_identifiers[*index])
        } else {
            None
        }
    }

    pub(super) fn get_seen_identifier(&self, index: usize) -> &Identifier {
        &self.seen_identifiers[index]
    }

    pub(super) fn get_identifier_mut(&mut self, identifier: &str) -> Option<&mut Identifier> {
        if let Some(index) = self.identifier_map.get(identifier) {
            Some(&mut self.seen_identifiers[*index])
        } else {
            None
        }
    }

    pub(super) fn rename_identifier(
        &mut self,
        old_name: &str,
        new_identifier_name: &str,
    ) -> Option<&Identifier> {
        if let Some(index) = self.identifier_map.remove(old_name) {
            self.identifier_map
                .insert(new_identifier_name.to_string(), index);
            Some(&self.seen_identifiers[index])
        } else {
            None
        }
    }

    pub(crate) fn get_graph_schema_entity(
        &self,
        identifier_name: &str,
        input_schema: &InputSchema,
    ) -> Option<GraphSchemaEntity> {
        self.get_identifier(identifier_name)
            .map(|identifier| identifier.get_graph_schema_entity(input_schema, self))
    }
}

impl Identifier {
    pub(super) fn prop_mut(&mut self) -> &mut IdPropIdentifier {
        if let Identifier::Prop(inner) = self {
            inner
        } else {
            panic!("Not Identifier::PropIdentifier.")
        }
    }

    pub(super) fn prop(&self) -> &IdPropIdentifier {
        if let Identifier::Prop(inner) = self {
            inner
        } else {
            panic!("Not Identifier::PropIdentifier.")
        }
    }

    pub(super) fn id(&mut self) -> &mut IdPropIdentifier {
        if let Identifier::Id(inner) = self {
            inner
        } else {
            panic!("Not Identifier::IdIdentifier.")
        }
    }

    pub(super) fn check_needed_property(
        &self,
        identifier: &str,
        prop: &str,
    ) -> Result<&FeatureValueType, QueryParserError> {
        match &self {
            Identifier::Vertex(inner) => inner.check_needed_property(identifier, prop),
            Identifier::Edge(inner) => inner.check_needed_property(identifier, prop),
            _ => Err(QueryParserError::UnsupportedQuery(format!(
                "Cannot get property {prop} on {identifier}"
            ))),
        }
    }

    pub(super) fn get_graph_schema_entity(
        &self,
        input_schema: &InputSchema,
        map: &IdentifierMap,
    ) -> GraphSchemaEntity {
        match &self {
            Identifier::Vertex(inner) => inner.get_schema_with_fields(input_schema),
            Identifier::Edge(inner) => inner.get_edge_schema(input_schema),
            Identifier::Id(inner) => match map.get_seen_identifier(inner.origin) {
                Identifier::Vertex(origin) => origin.get_vertex_schema(input_schema),
                Identifier::Edge(origin) => origin.get_edge_schema(input_schema),
                _ => panic!(),
            },
            Identifier::Prop(_inner) => GraphSchemaEntity {
                tlabel: None,
                entity_id: None,
                fields: Vec::new(),
            },
        }
    }

    pub(super) fn index(&self) -> IdentifierIndex {
        match &self {
            Identifier::Vertex(inner) => inner.index,
            Identifier::Edge(inner) => inner.index,
            Identifier::Id(inner) => inner.index,
            Identifier::Prop(inner) => inner.index,
        }
    }

    pub(super) fn set_label_schema(
        &mut self,
        label: &str,
        identifier: &str,
        input_schema: &InputSchema,
    ) -> Result<(), QueryParserError> {
        match self {
            Identifier::Vertex(inner) => {
                inner.set_label(label);
                let tlabel = inner.label.as_ref().unwrap();
                if let Some((entity, fields)) = input_schema.get_vertex_schema(tlabel) {
                    inner.entity = Some(entity.clone());
                    inner.fields = fields.clone();
                    Ok(())
                } else {
                    Err(QueryParserError::UnsupportedQuery(format!(
                        "{identifier}:{tlabel} is not an available vertex entity"
                    )))
                }
            }
            Identifier::Edge(inner) => {
                inner.set_label(label);
                let tlabel = inner.label.as_ref().unwrap();
                if let Some((entity, fields)) = input_schema.get_edge_schema(tlabel) {
                    inner.entity = Some(entity.clone());
                    inner.fields = fields.clone();
                    Ok(())
                } else {
                    Err(QueryParserError::UnsupportedQuery(format!(
                        "{identifier}:{tlabel} is not an available edge entity"
                    )))
                }
            }
            _ => {
                panic!("Cannot call set_label_schema on non edge/vertex identifier")
            }
        }
    }
}
