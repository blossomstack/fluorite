use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub types: Vec<CustomType>,

    pub configs: Option<DefinitionConfig>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DefinitionConfig {
    pub rust_package: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SimpleType {
    String,
    Bool,
    DateTime,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float32,
    Float64,
}

impl SimpleType {
    pub fn all_values() -> Vec<SimpleType> {
        vec![
            SimpleType::String,
            SimpleType::Bool,
            SimpleType::DateTime,
            SimpleType::UInt32,
            SimpleType::UInt64,
            SimpleType::Int32,
            SimpleType::Int64,
            SimpleType::Float32,
            SimpleType::Float64,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        fields: Vec<Field>,
    },
    Enum {
        name: String,
        values: Vec<String>,
    },
    ObjectEnum {
        name: String,
        type_tag: String,
        values: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub config: Option<FieldConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConfig {
    rename: String,
}
