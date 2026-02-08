//! Askama templates for Rust code generation

use askama::Template;

/// Template for rendering a struct
#[derive(Template)]
#[template(path = "rust/struct.rs.j2")]
pub struct StructTemplate {
    pub derives: String,
    pub name: String,
    pub fields: Vec<FieldTemplate>,
    /// Deny unknown fields during deserialization
    pub deny_unknown_fields: bool,
    /// Documentation comment for this struct
    pub doc: String,
}

/// Field information for templates
#[derive(Clone)]
pub struct FieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub is_optional: bool,
    pub needs_rename: bool,
    /// Alias names for serde(alias = "...")
    pub alias: Vec<String>,
    /// Default value for serde(default = "...")
    pub default: String,
    /// Skip if None for serde(skip_serializing_if = "Option::is_none")
    pub skip_if_none: bool,
    /// Skip if default for serde(skip_serializing_if = "...")
    pub skip_if_default: bool,
    /// Flatten for serde(flatten)
    pub flatten: bool,
    /// Documentation comment for this field
    pub doc: String,
    /// Whether this field is deprecated
    pub deprecated: bool,
}

impl FieldTemplate {
    pub fn code_name(&self) -> &str {
        &self.code_name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn needs_rename(&self) -> bool {
        self.needs_rename
    }
}

/// Template for rendering an enum
#[derive(Template)]
#[template(path = "rust/enum.rs.j2")]
pub struct EnumTemplate {
    pub derives: String,
    pub name: String,
    pub variants: Vec<String>,
}

/// Union variant types for template
#[derive(Clone)]
pub enum UnionVariantTemplate {
    /// Unit variant: no data
    Unit(String),
    /// Newtype variant: wraps a value
    Newtype { name: String, type_str: String },
}

/// Template for rendering a union (adjacently tagged enum)
#[derive(Template)]
#[template(path = "rust/union.rs.j2")]
pub struct UnionTemplate {
    pub derives: String,
    pub name: String,
    /// Tag field name (e.g., "type")
    pub tag_field: String,
    /// Content field name (e.g., "value")
    pub content_field: String,
    pub variants: Vec<UnionVariantTemplate>,
}

/// Template for rendering a list type alias
#[derive(Template)]
#[template(path = "rust/list_alias.rs.j2")]
pub struct ListAliasTemplate {
    pub name: String,
    pub item_type: String,
}

/// Template for rendering a map type alias
#[derive(Template)]
#[template(path = "rust/map_alias.rs.j2")]
pub struct MapAliasTemplate {
    pub name: String,
    pub key_type: String,
    pub value_type: String,
}

/// Template for rendering a module file
#[derive(Template)]
#[template(path = "rust/mod.rs.j2")]
pub struct ModTemplate {
    pub package: String,
    pub modules: Vec<ModuleEntry>,
}

#[derive(Clone)]
pub struct ModuleEntry {
    pub file_name: String,
}
