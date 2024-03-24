use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use super::ResourceId;
use super::ResourceOp;
use super::Variant;

#[macro_export]
macro_rules! entity {
    ($name: literal, $variant: expr, $tlabel: literal, $primary_key: literal) => {
        $crate::Entity::Vertex($crate::feature::VertexEntity {
            name: $name.to_owned(),
            variant: $variant,
            tlabel: $tlabel.to_owned(),
            primary_key: $primary_key.to_owned(),
        })
    };

    ($name: literal, $variant: expr, $tlabel: literal, $src_entity: expr, $dst_entity: expr) => {{
        use $crate::feature::ResourceOp;
        $crate::Entity::Edge($crate::feature::EdgeEntity {
            name: $name.to_string(),
            variant: $variant,
            tlabel: $tlabel.to_string(),
            src_tlabel: $src_entity.tlabel().to_string(),
            dst_tlabel: $dst_entity.tlabel().to_string(),
            src_entity_id: $src_entity.resource_id(),
            dst_entity_id: $dst_entity.resource_id(),
            directed: false,
            primary_key: None,
        })
    }};
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Entity {
    Vertex(VertexEntity),
    Edge(EdgeEntity),
}

#[pyclass(get_all)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VertexEntity {
    pub name: String,
    pub tlabel: String,
    pub primary_key: String,
    pub variant: Variant,
}

#[pyclass(get_all)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EdgeEntity {
    pub name: String,
    pub tlabel: String,
    pub src_tlabel: String,
    pub dst_tlabel: String,
    pub src_entity_id: String,
    pub dst_entity_id: String,
    pub directed: bool,
    pub primary_key: Option<String>,
    pub variant: Variant,
}

impl Entity {
    pub fn tlabel(&self) -> &str {
        match self {
            Entity::Vertex(entity) => &entity.tlabel,
            Entity::Edge(entity) => &entity.tlabel,
        }
    }

    pub fn primary_key(&self) -> Option<&String> {
        match self {
            Entity::Vertex(entity) => Some(&entity.primary_key),
            Entity::Edge(entity) => entity.primary_key.as_ref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entity::Vertex(entity) => &entity.name,
            Entity::Edge(entity) => &entity.name,
        }
    }
}

#[pymethods]
impl VertexEntity {
    fn resource_id(&self) -> ResourceId {
        format!("{}/Entity/{}", &self.variant, &self.name)
    }
}

#[pymethods]
impl EdgeEntity {
    fn resource_id(&self) -> ResourceId {
        format!(
            "{}/Entity/{}/{}/{}",
            &self.variant,
            &self.name,
            Entity::id_to_name(&self.src_entity_id),
            Entity::id_to_name(&self.dst_entity_id)
        )
    }
}

impl ResourceOp for Entity {
    fn resource_id(&self) -> ResourceId {
        match self {
            Entity::Vertex(entity) => entity.resource_id(),
            Entity::Edge(entity) => entity.resource_id(),
        }
    }

    fn id_to_name(id: &str) -> &str {
        id.split('/').nth(2).unwrap()
    }
}

impl IntoPy<PyObject> for Entity {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Entity::Vertex(entity) => entity.into_py(py),
            Entity::Edge(entity) => entity.into_py(py),
        }
    }
}

impl FromPyObject<'_> for Entity {
    fn extract(ob: &'_ PyAny) -> PyResult<Self> {
        let res: Result<VertexEntity, _> = ob.extract();
        if let Ok(vertex) = res {
            Ok(Entity::Vertex(vertex))
        } else {
            Ok(Entity::Edge(ob.extract()?))
        }
    }
}

/// Defines a vertex entity
#[pyfunction]
#[pyo3(signature=(name, tlabel,primary_key, variant=None),text_signature = "(name:str, tlabel:str, primary_key:str, variant:str = None)")]
pub fn vertex_entity(
    name: String,
    tlabel: String,
    primary_key: String,
    variant: Option<String>,
) -> Entity {
    Entity::Vertex(VertexEntity {
        name,
        variant: match variant {
            Some(str) => Variant::UserDefined(str),
            None => Variant::Default(),
        },
        tlabel,
        primary_key,
    })
}

#[pyfunction]
pub fn edge_entity(
    name: String,
    tlabel: String,
    src_entity: PyRef<VertexEntity>,
    dst_entity: PyRef<VertexEntity>,
    primary_key: Option<String>,
    variant: Option<String>,
    directed: Option<bool>,
) -> Entity {
    Entity::Edge(EdgeEntity {
        name,
        tlabel,
        primary_key,
        variant: match variant {
            Some(str) => Variant::UserDefined(str),
            None => Variant::Default(),
        },
        src_tlabel: src_entity.tlabel.to_owned(),
        dst_tlabel: dst_entity.tlabel.to_owned(),
        src_entity_id: src_entity.resource_id(),
        dst_entity_id: dst_entity.resource_id(),
        directed: directed.unwrap_or(false),
    })
}
