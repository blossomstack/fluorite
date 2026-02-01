use core::fmt;
use std::fmt::Display;

use crate::definitions::{CustomType, SimpleType};

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

    pub fn from(s: &str) -> Option<SimpleType> {
        SimpleType::all_values()
            .into_iter()
            .find(|t| t.to_string() == s)
    }
}

impl CustomType {
    pub(crate) fn type_name(&self) -> &str {
        match self {
            CustomType::Object {
                name,
                fields: _,
                configs: _,
                description: _,
            } => name.as_str(),
            CustomType::Enum {
                name,
                values: _,
                description: _,
            } => name.as_str(),
            CustomType::Union {
                name,
                type_tag: _,
                values: _,
                configs: _,
                description: _,
            } => name.as_str(),
            CustomType::List {
                name,
                item_type: _,
                description: _,
            } => name.as_str(),
            CustomType::Map {
                name,
                key_type: _,
                value_type: _,
                description: _,
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

pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
