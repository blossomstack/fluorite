#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DefinitionConfig {
    pub rust_package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeConfig {
    pub union_style: Option<crate::definitions::UnionStyle>,
    /// Rename all fields according to the given case convention (camelCase, snake_case, etc.)
    pub rename_all: Option<String>,
    /// Rust-specific type configuration
    pub rust: Option<RustTypeConfig>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
    /// Alternative names for this field when deserializing
    pub alias: Option<Vec<String>>,
    /// Default value for this field
    pub default: Option<String>,
    /// Rust-specific field configuration
    pub rust: Option<RustFieldConfig>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub configs: Option<crate::definitions::FieldConfig>,
    /// Documentation for this field
    pub description: Option<String>,
    /// Whether this field is deprecated
    pub deprecated: Option<bool>,
}

pub type EnumValueList = Vec<String>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        fields: crate::definitions::FieldList,
        configs: Option<crate::definitions::TypeConfig>,
        /// Documentation for this type
        description: Option<String>,
    },
    Enum {
        name: String,
        values: crate::definitions::EnumValueList,
        /// Documentation for this type
        description: Option<String>,
    },
    Union {
        name: String,
        type_tag: String,
        values: crate::definitions::EnumValueList,
        configs: Option<crate::definitions::TypeConfig>,
        /// Documentation for this type
        description: Option<String>,
    },
    List {
        name: String,
        item_type: String,
        /// Documentation for this type
        description: Option<String>,
    },
    Map {
        name: String,
        key_type: String,
        value_type: String,
        /// Documentation for this type
        description: Option<String>,
    },
}
pub type CustomTypeList = Vec<crate::definitions::CustomType>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UnionStyle {
    Inline,
    Extern,
}

pub type FieldList = Vec<crate::definitions::Field>;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RustTypeWrapper {
    Box,
}

/// Rust-specific type-level configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RustTypeConfig {
    /// Deny unknown fields during deserialization
    pub deny_unknown_fields: Option<bool>,
}

/// Rust-specific field-level configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RustFieldConfig {
    /// Skip serialization if the field is None
    pub skip_if_none: Option<bool>,
    /// Skip serialization if the field equals its default value
    pub skip_if_default: Option<bool>,
    /// Flatten the field during serialization
    pub flatten: Option<bool>,
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
