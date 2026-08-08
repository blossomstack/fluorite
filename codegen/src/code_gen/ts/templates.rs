use askama::Template;

/// Field information for TypeScript templates
#[derive(Clone)]
pub struct TsFieldTemplate {
    pub code_name: String,
    pub type_str: String,
    pub is_optional: bool,
    /// Documentation comment for this field
    pub doc: Vec<String>,
    /// Whether this field is deprecated
    pub deprecated: bool,
}

/// Import information for TypeScript templates
#[derive(Clone)]
pub struct TsImport {
    pub name: String,
    pub path: String,
}

/// Template for rendering a TypeScript interface
#[derive(Template)]
#[template(path = "ts/interface.ts.j2")]
pub struct InterfaceTemplate {
    pub name: String,
    pub fields: Vec<TsFieldTemplate>,
    pub use_readonly: bool,
    pub imports: Vec<TsImport>,
    /// Documentation comment for this interface
    pub doc: Vec<String>,
}

/// Template for rendering a TypeScript enum
#[derive(Template)]
#[template(path = "ts/enum.ts.j2")]
pub struct TsEnumTemplate {
    pub name: String,
    pub variants: Vec<TsEnumVariantTemplate>,
    /// Documentation comment for this enum
    pub doc: Vec<String>,
}

/// One enum variant for TypeScript templates
#[derive(Clone)]
pub struct TsEnumVariantTemplate {
    pub name: String,
    /// Documentation comment for this variant
    pub doc: Vec<String>,
}

/// Union variant types for template (adjacently tagged format)
#[derive(Clone)]
pub enum TsUnionVariantTemplate {
    /// Unit variant: `{ type: "Deleted" }`
    Unit { name: String, doc: Vec<String> },
    /// Newtype variant: `{ type: "Created", value: ... }`
    Newtype {
        name: String,
        type_str: String,
        doc: Vec<String>,
    },
}

/// Template for rendering a TypeScript discriminated union (adjacently tagged)
#[derive(Template)]
#[template(path = "ts/union.ts.j2")]
pub struct TsUnionTemplate {
    pub name: String,
    pub tag_field: String,
    pub content_field: String,
    pub variants: Vec<TsUnionVariantTemplate>,
    pub imports: Vec<TsImport>,
    /// Documentation comment for this union
    pub doc: Vec<String>,
}

/// Template for rendering a TypeScript type alias
#[derive(Template)]
#[template(path = "ts/type_alias.ts.j2")]
pub struct TsTypeAliasTemplate {
    pub name: String,
    pub target_type: String,
    pub imports: Vec<TsImport>,
    /// Documentation comment for this type alias
    pub doc: Vec<String>,
}

/// Template for rendering an index file
#[derive(Template)]
#[template(path = "ts/index.ts.j2")]
pub struct TsIndexTemplate {
    pub modules: Vec<TsModuleEntry>,
}

#[derive(Clone)]
pub struct TsModuleEntry {
    pub file_name: String,
}
