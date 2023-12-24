use crate::definitions::{Field, FieldConfig};

use super::FieldType;

pub enum TypeInfo {
    Object(ObjectTypeInfo),
    Enum(EnumTypeInfo),
    ObjectEnum(ObjectEnumTypeInfo),
}

pub struct ObjectEnumTypeInfo {
    pub package: String,
    pub name: String,
    pub type_tag: String,
    pub values: Vec<ObjectEnumValue>,
}

pub enum ObjectEnumValue {
    Simple(String),
    ObjectType(String),
}

pub struct EnumTypeInfo {
    pub package: String,
    pub name: String,
    pub values: Vec<String>,
}

pub struct ObjectTypeInfo {
    pub package: String,
    pub name: String,
    pub fields: Vec<ObjectField>,
    pub is_object_enum_value: bool,
}

pub struct ObjectField {
    pub name: String,
    pub field_type: FieldType,
    pub config: Option<FieldConfig>,
}

impl From<&Field> for ObjectField {
    fn from(f: &Field) -> Self {
        ObjectField {
            name: f.name.clone(),
            field_type: FieldType::get_field_type(f.field_type.as_str()),
            config: f.config.clone(),
        }
    }
}

impl TypeInfo {
    pub fn is_object_enum_value(&self) -> bool {
        match self {
            TypeInfo::Object(o) => o.is_object_enum_value,
            _ => false,
        }
    }
    pub fn get_referrenced_types(&self) -> Vec<String> {
        match &self {
            TypeInfo::Object(object) => object
                .fields
                .iter()
                .map(|f| match &f.field_type {
                    FieldType::Custom { name } => Some(name.clone()),
                    _ => None,
                })
                .filter(|t| t.is_some())
                .map(|t| t.unwrap())
                .collect(),
            TypeInfo::Enum(_) => vec![],
            TypeInfo::ObjectEnum(e) => e
                .values
                .iter()
                .map(|v| match v {
                    ObjectEnumValue::Simple(_) => None,
                    ObjectEnumValue::ObjectType(t) => Some(t.clone()),
                })
                .filter(|o| o.is_some())
                .map(|o| o.unwrap())
                .collect(),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeInfo::Object(o) => &o.name,
            TypeInfo::Enum(e) => &e.name,
            TypeInfo::ObjectEnum(o) => &o.name,
        }
    }

    pub(crate) fn has_unknown_fields(&self) -> bool {
        matches!(&self, TypeInfo::Object(o) if o.fields.iter().find(|f| FieldType::is_unknown_field(&f.field_type)).is_some())
    }

    pub(crate) fn package<'a>(&'a self) -> &'a str {
        match self {
            TypeInfo::Object(o) => o.package.as_str(),
            TypeInfo::Enum(e) => e.package.as_str(),
            TypeInfo::ObjectEnum(o) => o.package.as_str(),
        }
    }
}
