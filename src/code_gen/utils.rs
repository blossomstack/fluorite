use crate::definitions::{CustomType, FieldType};

pub fn get_ref_types(value: &CustomType) -> Vec<String> {
    match value {
        CustomType::Object { name: _, fields } => fields
            .iter()
            .filter(|f| is_custom_type(&f.field_type))
            .map(|f| get_field_type_name(&f.field_type))
            .collect(),
        CustomType::Enum { name: _, values: _ } => vec![],
    }
}

pub fn get_field_type_name(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Simple(t) => format!("{:?}", t),
        FieldType::Custom { name } => name.to_string(),
    }
}

pub fn is_custom_type(field_type: &FieldType) -> bool {
    match field_type {
        FieldType::Simple(_) => false,
        FieldType::Custom { name: _ } => true,
    }
}

pub fn get_type_name(t: &CustomType) -> String {
    match t {
        CustomType::Object { name, fields: _ } => name.to_string(),
        CustomType::Enum { name, values: _ } => name.to_string(),
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
