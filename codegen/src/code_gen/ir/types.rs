//! Language-agnostic Intermediate Representation for code generation.
//!
//! This IR sits between the parsed YAML definitions and language-specific
//! code generation, providing a clean abstraction layer.

use std::collections::HashMap;

/// Represents a complete schema ready for code generation
#[derive(Debug, Clone)]
pub struct IRSchema {
    pub packages: HashMap<String, IRPackage>,
}

/// A package/module containing types
#[derive(Debug, Clone)]
pub struct IRPackage {
    pub name: String,
    pub types: Vec<IRType>,
}

/// A type in the IR
#[derive(Debug, Clone)]
pub enum IRType {
    Struct(IRStruct),
    Enum(IREnum),
    Union(IRUnion),
    TypeAlias(IRTypeAlias),
}

impl IRType {
    pub fn name(&self) -> &str {
        match self {
            IRType::Struct(s) => &s.name,
            IRType::Enum(e) => &e.name,
            IRType::Union(u) => &u.name,
            IRType::TypeAlias(a) => &a.name,
        }
    }

    pub fn is_internal(&self) -> bool {
        match self {
            IRType::Struct(s) => s.is_union_variant,
            IRType::Enum(_) | IRType::Union(_) | IRType::TypeAlias(_) => false,
        }
    }
}

/// A struct type
#[derive(Debug, Clone)]
pub struct IRStruct {
    pub name: String,
    pub fields: Vec<IRField>,
    pub is_union_variant: bool,
    pub doc: Option<String>,
    /// Rename all fields according to case convention
    pub rename_all: Option<String>,
    /// Deny unknown fields during deserialization
    pub deny_unknown_fields: bool,
}

/// A field within a struct
#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRFieldType,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub rename: Option<String>,
    pub doc: Option<String>,
    /// Alternative names for this field when deserializing
    pub alias: Vec<String>,
    /// Default value expression for this field
    pub default: Option<String>,
    /// Skip serialization if None
    pub skip_if_none: bool,
    /// Skip serialization if equal to default
    pub skip_if_default: bool,
    /// Flatten this field
    pub flatten: bool,
    /// Whether this field is deprecated
    pub deprecated: bool,
}

impl IRField {
    /// Returns the name to use in generated code (respects rename)
    pub fn code_name(&self) -> &str {
        self.rename.as_deref().unwrap_or(&self.name)
    }

    /// Returns the original name (for serde rename attribute)
    pub fn original_name(&self) -> &str {
        &self.name
    }

    /// Whether this field needs a serde rename attribute
    pub fn needs_rename(&self) -> bool {
        self.rename.is_some()
    }

    /// Whether this field has alias attributes
    pub fn has_alias(&self) -> bool {
        !self.alias.is_empty()
    }
}

/// Field type representation
#[derive(Debug, Clone)]
pub enum IRFieldType {
    Primitive(IRPrimitive),
    Custom(String),
    Any,
    List(Box<IRFieldType>),
    Map(Box<IRFieldType>, Box<IRFieldType>),
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRPrimitive {
    // Basic primitives
    String,
    Bool,
    DateTime,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float32,
    Float64,
    // Extended primitives
    #[allow(clippy::upper_case_acronyms)]
    UUID,
    Decimal,
    Bytes,
    Url,
    Timestamp,
    TimestampMillis,
    DateTimeUtc,
    DateTimeTz,
    Date,
    Time,
    Duration,
}

/// An enum type (simple variants without data)
#[derive(Debug, Clone)]
pub struct IREnum {
    pub name: String,
    pub variants: Vec<String>,
    pub doc: Option<String>,
}

/// A tagged union type
#[derive(Debug, Clone)]
pub struct IRUnion {
    pub name: String,
    pub tag_field: String,
    pub variants: Vec<IRUnionVariant>,
    pub style: IRUnionStyle,
    pub doc: Option<String>,
}

/// Union variant
#[derive(Debug, Clone)]
pub enum IRUnionVariant {
    /// Simple variant with no data (unit variant)
    Unit(String),
    /// Variant with inlined struct fields
    Inline(String, Vec<IRField>),
    /// Variant wrapping another type
    Newtype(String, String),
}

impl IRUnionVariant {
    pub fn name(&self) -> &str {
        match self {
            IRUnionVariant::Unit(n) => n,
            IRUnionVariant::Inline(n, _) => n,
            IRUnionVariant::Newtype(n, _) => n,
        }
    }
}

/// How to generate the union
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRUnionStyle {
    /// Inline fields into enum variants
    Inline,
    /// Use newtype pattern wrapping external types
    Extern,
}

/// Type alias (for List and Map types)
#[derive(Debug, Clone)]
pub struct IRTypeAlias {
    pub name: String,
    pub target: IRTypeAliasTarget,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IRTypeAliasTarget {
    List(IRFieldType),
    Map(IRFieldType, IRFieldType),
}
