use core::fmt;
use std::{collections::HashMap, fmt::Display};

use crate::definitions::{CustomType, Definition, SimpleType};

use super::abi::{
    CodeGenContext, EnumTypeInfo, ListTypeInfo, MapTypeInfo, ObjectEnumTypeInfo, ObjectEnumValue,
    ObjectField, ObjectTypeInfo, PreProcessor, TypeInfo, TypeName,
};

impl Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
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
impl CustomType {
    pub(crate) fn type_name(&self) -> &str {
        match self {
            CustomType::Object { name, fields: _ } => name.as_str(),
            CustomType::Enum { name, values: _ } => name.as_str(),
            CustomType::ObjectEnum {
                name,
                type_tag: _,
                values: _,
            } => name.as_str(),
            CustomType::List { name, item_type: _ } => name.as_str(),
            CustomType::Map {
                name,
                key_type: _,
                value_type: _,
            } => name.as_str(),
        }
    }
}

pub fn to_snake_case(s: &str) -> String {
    let mut snake_case = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() && !snake_case.is_empty() && chars.peek().is_some() {
            snake_case.push('_');
        }
        snake_case.extend(c.to_lowercase());
    }

    snake_case
}

pub(crate) fn build_type_dict<C: CodeGenContext>(
    definitions: &Vec<Definition>,
    pre_processor: &Box<dyn PreProcessor<C>>,
) -> anyhow::Result<HashMap<String, TypeInfo>> {
    let mut object_enum_value_type_names = Vec::new();
    let mut all_type_names = Vec::new();
    for d in definitions {
        for t in &d.types {
            all_type_names.push(t.type_name().to_owned());
            if let CustomType::ObjectEnum { values, .. } = t {
                for v in values {
                    object_enum_value_type_names.push(v.clone());
                }
            }
        }
    }

    // identify all types
    let mut all_types: HashMap<String, TypeInfo> = HashMap::new();
    for d in definitions {
        let package = pre_processor.get_package_name(d)?;
        for t in &d.types {
            match t {
                CustomType::Object { name, fields } => {
                    let fields = fields.iter().map(ObjectField::from).collect();
                    let is_object_enum_value = object_enum_value_type_names.contains(name);
                    let type_info = ObjectTypeInfo {
                        package: package.clone(),
                        name: name.clone(),
                        fields,
                        is_object_enum_value,
                    };
                    all_types.insert(name.clone(), TypeInfo::Object(type_info));
                }
                CustomType::Enum { name, values } => {
                    let type_info = EnumTypeInfo {
                        package: package.clone(),
                        name: name.clone(),
                        values: values.clone(),
                    };
                    all_types.insert(name.clone(), TypeInfo::Enum(type_info));
                }
                CustomType::ObjectEnum {
                    name,
                    type_tag,
                    values,
                } => {
                    let values = values
                        .iter()
                        .map(|v| match all_type_names.contains(v) {
                            true => ObjectEnumValue::CustomType(v.clone()),
                            false => ObjectEnumValue::Simple(v.clone()),
                        })
                        .collect();
                    let type_info = ObjectEnumTypeInfo {
                        package: package.clone(),
                        name: name.clone(),
                        type_tag: type_tag.clone(),
                        values,
                    };
                    all_types.insert(name.clone(), TypeInfo::ObjectEnum(type_info));
                }
                CustomType::List { name, item_type } => {
                    let type_info = ListTypeInfo {
                        package: package.clone(),
                        name: name.clone(),
                        item_type: TypeName::from_str(item_type),
                    };
                    all_types.insert(name.clone(), TypeInfo::List(type_info));
                }
                CustomType::Map {
                    name,
                    key_type,
                    value_type,
                } => {
                    let type_info = MapTypeInfo {
                        package: package.clone(),
                        name: name.clone(),
                        key_type: TypeName::from_str(key_type),
                        value_type: TypeName::from_str(value_type),
                    };
                    all_types.insert(name.clone(), TypeInfo::Map(type_info));
                }
            }
        }
    }
    Ok(all_types)
}
