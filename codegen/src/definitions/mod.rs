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
    List {
        name: String,
        item_type: String,
    },
    Map {
        name: String,
        key_type: String,
        value_type: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub config: Option<FieldConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<RustTypeWrapper>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RustTypeWrapper {
    Box,
}
