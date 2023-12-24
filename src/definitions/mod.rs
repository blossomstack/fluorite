use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_yaml::Value;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Definition {
    #[serde(rename = "types")]
    pub custom_types: Vec<CustomType>,

    #[serde(flatten)]
    pub configs: HashMap<String, Value>,
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

impl Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object { name: String, fields: Vec<Field> },
    Enum { name: String, values: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Simple(SimpleType),
    Custom { name: String },
}

impl Serialize for FieldType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FieldType::Simple(simple) => serializer.serialize_str(format!("{:?}", simple).as_str()),
            FieldType::Custom { name } => serializer.serialize_str(name.as_str()),
        }
    }
}

struct FieldTypeEnumVisitor;

impl<'de> Visitor<'de> for FieldTypeEnumVisitor {
    type Value = FieldType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let all_simple_types = SimpleType::all_values()
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        formatter.write_fmt(format_args!(
            "Simple types: {}, or user defined custom types.",
            all_simple_types
        ))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let opt_simple_type = SimpleType::all_values()
            .into_iter()
            .find(|t| t.to_string() == value);
        let field_type = match opt_simple_type {
            Some(t) => FieldType::Simple(t),
            None => FieldType::Custom {
                name: value.to_owned(),
            },
        };
        Ok(field_type)
    }
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FieldTypeEnumVisitor)
    }
}
