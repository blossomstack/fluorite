#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DefinitionConfig {
    pub rust_package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeConfig {
    pub object_enum_style: Option<crate::definitions::ObjectEnumStyle>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub configs: Option<crate::definitions::FieldConfig>,
}

pub type EnumValueList = Vec<String>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        configs: Option<crate::definitions::TypeConfig>,
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
pub type CustomTypeList = Vec<crate::definitions::CustomType>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ObjectEnumStyle {
    Inline,
    Extern,
}

pub type FieldList = Vec<crate::definitions::Field>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RustTypeWrapper {
    Box,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub types: crate::definitions::CustomTypeList,
    pub configs: crate::definitions::DefinitionConfig,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
