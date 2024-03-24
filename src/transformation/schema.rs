use std::collections::HashMap;

use crate::{feature::ResourceId, FeatureValueType, Field};

/// The schema of an input graph, including entity label, entity id, and property/field names and types.
#[derive(Debug, Default)]
pub struct InputSchema {
    pub(crate) vertex_entities: HashMap<String, (ResourceId, HashMap<String, FeatureValueType>)>,
    pub(crate) edge_entities: HashMap<String, (ResourceId, HashMap<String, FeatureValueType>)>,
}

impl InputSchema {
    pub fn from(
        vertex_fvs: &HashMap<String, (String, Vec<Field>)>,
        edge_fvs: &HashMap<String, (String, Vec<Field>)>,
        v_entities: &HashMap<String, ResourceId>,
        e_entities: &HashMap<String, ResourceId>,
    ) -> Self {
        let mut res = Self::default();
        for (tlabel, (_, fields)) in vertex_fvs.iter() {
            let field_map: HashMap<String, FeatureValueType> = fields
                .iter()
                .map(|f| (f.name.clone(), f.value_type.clone()))
                .collect();

            // tlabel, (ResourceID, HashMap(field_name, FeatureValueType))
            res.vertex_entities.insert(
                tlabel.to_string(),
                (v_entities.get(tlabel).unwrap().to_owned(), field_map),
            );
        }

        for (tlabel, (_, fields)) in edge_fvs.iter() {
            let field_map: HashMap<String, FeatureValueType> = fields
                .iter()
                .map(|f| (f.name.clone(), f.value_type.clone()))
                .collect();
            res.edge_entities.insert(
                tlabel.to_string(),
                (e_entities.get(tlabel).unwrap().to_owned(), field_map),
            );
        }
        res
    }

    /// @returns None if no vertex with the given tlabel is in the schema; otherwise the entity, field names, and types associated with the vertex
    pub(crate) fn get_vertex_schema(
        &self,
        tlabel: &str,
    ) -> Option<&(ResourceId, HashMap<String, FeatureValueType>)> {
        self.vertex_entities.get(tlabel)
    }

    /// @returns None if no edge with the given tlabel is in the schema; otherwise the field names, entity, and types associated with the edge
    pub(crate) fn get_edge_schema(
        &self,
        tlabel: &str,
    ) -> Option<&(ResourceId, HashMap<String, FeatureValueType>)> {
        self.edge_entities.get(tlabel)
    }
}

#[allow(dead_code)] // TODO(tatiana): to be finished
#[derive(Debug)]
pub(crate) struct GraphSchemaEntity {
    pub(crate) tlabel: Option<String>,        // None if new entity
    pub(crate) entity_id: Option<ResourceId>, // None if new entity
    pub(crate) fields: Vec<(String, FeatureValueType)>, // empty if RETURN id(node)
}

#[allow(dead_code)] // TODO(tatiana): to be finished
impl GraphSchemaEntity {
    pub fn new(
        tlabel: Option<String>,
        entity_id: Option<ResourceId>,
        fields: Vec<(String, FeatureValueType)>,
    ) -> Self {
        Self {
            tlabel,
            entity_id,
            fields,
        }
    }

    pub fn get_tlabel(&self) -> &Option<String> {
        &self.tlabel
    }

    pub fn get_entity_id(&self) -> &Option<ResourceId> {
        &self.entity_id
    }

    pub fn get_tlabel_id(&self) -> (&Option<String>, &Option<ResourceId>) {
        (&self.tlabel, &self.entity_id)
    }
}

#[derive(Debug)]
pub struct GraphSchema {
    pub(crate) src: GraphSchemaEntity,
    pub(crate) dst: GraphSchemaEntity,
    pub(crate) edge: GraphSchemaEntity,
}

impl GraphSchema {
    pub fn new() -> Self {
        Self {
            src: GraphSchemaEntity::new(None, None, vec![]),
            dst: GraphSchemaEntity::new(None, None, vec![]),
            edge: GraphSchemaEntity::new(None, None, vec![]),
        }
    }

    pub(crate) fn get_node_schema(&self) -> (&GraphSchemaEntity, &GraphSchemaEntity) {
        (&self.src, &self.dst)
    }

    pub(crate) fn get_edge_schema(&self) -> &GraphSchemaEntity {
        &self.edge
    }
}

impl Default for GraphSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // TODO(tatiana): to be finished
#[derive(Debug)]
pub enum OutputSchema {
    DataFrameSchema {
        schema: Vec<(Option<ResourceId>, String, FeatureValueType)>,
    }, // entity id, column name, column type
    GraphSchema(GraphSchema),
}
