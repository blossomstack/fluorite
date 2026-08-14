//! AST types for the Fluorite IDL

use std::ops::Range;

/// Source span for error reporting
pub type Span = Range<usize>;

/// A spanned value
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// A complete .fl file
#[derive(Debug, Clone)]
pub struct AstFile {
    pub package: Vec<Spanned<String>>,
    pub uses: Vec<AstUse>,
    pub items: Vec<AstItem>,
}

/// A use statement: `use foo::bar::Type;`
#[derive(Debug, Clone)]
pub struct AstUse {
    pub path: Vec<Spanned<String>>,
    pub span: Span,
}

/// A top-level item
#[derive(Debug, Clone)]
pub enum AstItem {
    Struct(AstStruct),
    Enum(AstEnum),
    Union(AstUnion),
    TypeAlias(AstTypeAlias),
}

impl AstItem {
    /// The declared type name, whichever kind of item this is.
    pub fn name(&self) -> &str {
        match self {
            AstItem::Struct(s) => &s.name.value,
            AstItem::Enum(e) => &e.name.value,
            AstItem::Union(u) => &u.name.value,
            AstItem::TypeAlias(t) => &t.name.value,
        }
    }
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct AstStruct {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub fields: Vec<AstField>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Field in a struct
#[derive(Debug, Clone)]
pub struct AstField {
    pub name: Spanned<String>,
    pub ty: AstType,
    pub attrs: Vec<AstAttribute>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Type expression
#[derive(Debug, Clone)]
pub enum AstType {
    /// Simple type name: `String`, `User`
    Named(Spanned<String>),
    /// Optional type: `Option<T>`
    Option(Box<AstType>),
    /// List type: `Vec<T>`
    Vec(Box<AstType>),
    /// Map type: `Map<K, V>`
    Map(Box<AstType>, Box<AstType>),
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct AstEnum {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub variants: Vec<AstEnumVariant>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct AstEnumVariant {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Union definition (tagged union)
#[derive(Debug, Clone)]
pub struct AstUnion {
    pub name: Spanned<String>,
    pub attrs: Vec<AstAttribute>,
    pub variants: Vec<AstUnionVariant>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Union variant
#[derive(Debug, Clone)]
pub struct AstUnionVariant {
    pub name: Spanned<String>,
    pub inner_type: Option<Spanned<String>>,
    pub doc: Option<String>,
    pub span: Span,
}

/// Type alias: `type OrderList = Vec<Order>;`
#[derive(Debug, Clone)]
pub struct AstTypeAlias {
    pub name: Spanned<String>,
    pub target: AstType,
    pub doc: Option<String>,
    pub span: Span,
}

/// Attribute: `#[rename = "value"]` or `#[deprecated]`
#[derive(Debug, Clone)]
pub struct AstAttribute {
    pub name: Spanned<String>,
    pub value: Option<Spanned<String>>,
    pub span: Span,
}
