use core::fmt;
use std::fmt::Display;

use crate::definitions::{CustomType, SimpleType};

impl Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self)
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
