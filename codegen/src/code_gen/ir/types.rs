//! Language-agnostic Intermediate Representation for code generation.
//!
//! This IR sits between the parsed YAML definitions and language-specific
//! code generation, providing a clean abstraction layer.

use std::collections::BTreeMap;

/// Represents a complete schema ready for code generation.
///
/// Packages are held in a `BTreeMap` so every traversal — file emission, module
/// declarations, import lists — visits them in the same order on every run.
/// A `HashMap` here reseeds per process, which made generated output vary
/// between otherwise identical builds.
#[derive(Debug, Clone)]
pub struct IRSchema {
    pub packages: BTreeMap<String, IRPackage>,
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
}

/// A struct type
#[derive(Debug, Clone)]
pub struct IRStruct {
    pub name: String,
    pub fields: Vec<IRField>,
    pub doc: Option<String>,
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
    Custom(IRTypeRef),
    Any,
    List(Box<IRFieldType>),
    Map(Box<IRFieldType>, Box<IRFieldType>),
}

/// A reference to a user-defined type, resolved to the package that declares it.
///
/// Resolution happens once, while lowering the AST, because that is the only
/// point where the referencing file's package and `use` imports are known. A
/// bare name alone cannot be resolved: two packages may declare the same name,
/// and a generator scanning the schema for a match has no way to pick the right
/// one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IRTypeRef {
    /// Dotted package name that declares the type, e.g. `demo.common`.
    pub package: String,
    /// Bare type name, e.g. `Address`.
    pub name: String,
}

impl IRTypeRef {
    pub fn new(package: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            name: name.into(),
        }
    }
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
    pub variants: Vec<IREnumVariant>,
    pub doc: Option<String>,
}

/// One variant of an enum. Carries a doc so `/// ...` above a variant reaches
/// generated code, matching [`IRUnionVariant`].
#[derive(Debug, Clone)]
pub struct IREnumVariant {
    pub name: String,
    pub doc: Option<String>,
}

impl IREnumVariant {
    /// A variant with no documentation — the common case in tests and in
    /// schemas that only document the enum itself.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            doc: None,
        }
    }
}

/// A tagged union type (adjacently tagged: `{tag_field: "Variant", content_field: value}`)
#[derive(Debug, Clone)]
pub struct IRUnion {
    pub name: String,
    /// Field name for the type discriminator (e.g., "type")
    pub tag_field: String,
    /// Field name for the content (e.g., "value")
    pub content_field: String,
    pub variants: Vec<IRUnionVariant>,
    pub doc: Option<String>,
}

/// Union variant
#[derive(Debug, Clone)]
pub enum IRUnionVariant {
    /// Simple variant with no data (unit variant): `{ type: "Deleted" }`
    Unit { name: String, doc: Option<String> },
    /// Variant with data: `{ type: "Created", value: ... }`
    /// `ty` is the type being wrapped
    Newtype {
        name: String,
        ty: IRFieldType,
        doc: Option<String>,
    },
}

impl IRUnionVariant {
    pub fn name(&self) -> &str {
        match self {
            IRUnionVariant::Unit { name, .. } => name,
            IRUnionVariant::Newtype { name, .. } => name,
        }
    }

    pub fn doc(&self) -> Option<&str> {
        match self {
            IRUnionVariant::Unit { doc, .. } => doc.as_deref(),
            IRUnionVariant::Newtype { doc, .. } => doc.as_deref(),
        }
    }
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
