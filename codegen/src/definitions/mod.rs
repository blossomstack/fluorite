#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub configs: Option<crate::definitions::FieldConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub types: crate::definitions::CustomTypeList,
    pub configs: crate::definitions::DefinitionConfig,
}

pub type CustomTypeList = Vec<crate::definitions::CustomType>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RustTypeWrapper {
    Box,
}

pub type FieldList = Vec<crate::definitions::Field>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        fields: crate::definitions::FieldList,
    },
    Enum {
        name: String,
        values: crate::definitions::EnumValueList,
    },
    ObjectEnum {
        name: String,
        type_tag: String,
        values: crate::definitions::EnumValueList,
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
pub type EnumValueList = Vec<String>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefinitionConfig {
    pub rust_package: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
}
