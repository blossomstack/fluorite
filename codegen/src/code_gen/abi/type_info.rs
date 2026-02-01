use crate::definitions::{Field, FieldConfig, SimpleType, TypeConfig};

pub enum TypeInfo {
    Object(ObjectTypeInfo),
    Enum(EnumTypeInfo),
    Union(UnionTypeInfo),
    List(ListTypeInfo),
    Map(MapTypeInfo),
}

pub enum TypeName {
    Any,
    Simple(SimpleType),
    CustomType(String),
}

impl TypeName {
    pub fn is_custom_type(&self) -> bool {
        match self {
            TypeName::Simple(_) => false,
            TypeName::CustomType(_) => true,
            TypeName::Any => false,
        }
    }

    pub fn parse(field_type: &str) -> TypeName {
        let opt_simple_type = SimpleType::all_values()
            .into_iter()
            .find(|t| t.to_string() == field_type);
        match opt_simple_type {
            Some(t) => TypeName::Simple(t),
            None if field_type == "Any" => TypeName::Any,
            None => TypeName::CustomType(field_type.to_owned()),
        }
    }
}

pub struct ListTypeInfo {
    pub package: String,
    pub name: String,
    pub item_type: TypeName,
}

pub struct MapTypeInfo {
    pub package: String,
    pub name: String,
    pub key_type: TypeName,
    pub value_type: TypeName,
}

pub struct UnionTypeInfo {
    pub package: String,
    pub name: String,
    pub type_tag: String,
    pub values: Vec<UnionValue>,
    pub configs: Option<TypeConfig>,
}

pub enum UnionValue {
    Simple(String),
    CustomType(String),
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
    pub is_union_value: bool,
}

pub struct ObjectField {
    pub name: String,
    pub field_type: TypeName,
    pub config: Option<FieldConfig>,
    pub optional: Option<bool>,
}

impl From<&Field> for ObjectField {
    fn from(f: &Field) -> Self {
        ObjectField {
            name: f.name.clone(),
            field_type: TypeName::parse(f.field_type.as_str()),
            config: f.configs.clone(),
            optional: f.optional,
        }
    }
}

impl ObjectField {
    pub fn is_optional(&self) -> bool {
        matches!(&self.optional, Some(true))
    }
}

impl TypeInfo {
    pub fn is_union_value(&self) -> bool {
        match self {
            TypeInfo::Object(o) => o.is_union_value,
            TypeInfo::Enum(_) | TypeInfo::Union(_) | TypeInfo::List(_) | TypeInfo::Map(_) => false,
        }
    }

    pub fn get_referrenced_types(&self) -> Vec<String> {
        match &self {
            TypeInfo::Object(object) => {
                let field_types = object.fields.iter().map(|f| &f.field_type).collect();
                Self::get_custom_types(field_types)
            }
            TypeInfo::Enum(_) => vec![],
            TypeInfo::Union(e) => e
                .values
                .iter()
                .filter_map(|v| match v {
                    UnionValue::Simple(_) => None,
                    UnionValue::CustomType(t) => Some(t.clone()),
                })
                .collect(),
            TypeInfo::List(l) => match &l.item_type {
                TypeName::Simple(_) => vec![],
                TypeName::CustomType(name) => vec![name.to_owned()],
                TypeName::Any => vec![],
            },
            TypeInfo::Map(m) => Self::get_custom_types(vec![&m.key_type, &m.value_type]),
        }
    }

    fn get_custom_types(types: Vec<&TypeName>) -> Vec<String> {
        types
            .into_iter()
            .filter_map(|t| match t {
                TypeName::CustomType(name) => Some(name.clone()),
                TypeName::Any | TypeName::Simple(_) => None,
            })
            .collect()
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeInfo::Object(o) => o.name.as_str(),
            TypeInfo::Enum(e) => e.name.as_str(),
            TypeInfo::Union(o) => o.name.as_str(),
            TypeInfo::List(l) => l.name.as_str(),
            TypeInfo::Map(m) => m.name.as_str(),
        }
    }

    pub(crate) fn package(&self) -> &str {
        match self {
            TypeInfo::Object(o) => o.package.as_str(),
            TypeInfo::Enum(e) => e.package.as_str(),
            TypeInfo::Union(o) => o.package.as_str(),
            TypeInfo::List(l) => l.package.as_str(),
            TypeInfo::Map(m) => m.package.as_str(),
        }
    }
}
