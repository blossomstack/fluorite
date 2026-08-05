use askama::Template;

/// Field information for Swift templates
#[derive(Clone)]
pub struct SwiftFieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub needs_rename: bool,
    /// Documentation comment for this field
    pub doc: String,
    /// Whether this field is deprecated
    pub deprecated: bool,
}

/// Import information for Swift templates
#[derive(Clone)]
pub struct SwiftImport {
    pub name: String,
}

/// Enum variant for Swift templates
#[derive(Clone)]
pub struct SwiftEnumVariant {
    pub code_name: String,
    pub original_name: String,
    pub needs_rename: bool,
}

/// Template for rendering a Swift struct
#[derive(Template)]
#[template(path = "swift/struct.swift.j2")]
pub struct SwiftStructTemplate {
    pub name: String,
    pub fields: Vec<SwiftFieldTemplate>,
    pub visibility: String,
    pub needs_coding_keys: bool,
    pub imports: Vec<SwiftImport>,
    /// Documentation comment for this struct
    pub doc: String,
}

/// Template for rendering a Swift enum
#[derive(Template)]
#[template(path = "swift/enum.swift.j2")]
pub struct SwiftEnumTemplate {
    pub name: String,
    pub variants: Vec<SwiftEnumVariant>,
    pub visibility: String,
    /// Documentation comment for this enum
    pub doc: String,
}

/// Union variant types for template (adjacently tagged format)
#[derive(Clone)]
pub enum SwiftUnionVariantTemplate {
    /// Unit variant: `.deleted`
    Unit {
        case_name: String,
        serialized_name: String,
        doc: String,
    },
    /// Newtype variant: `.created(User)`
    Newtype {
        case_name: String,
        serialized_name: String,
        type_str: String,
        doc: String,
    },
}

/// Template for rendering a Swift discriminated union (adjacently tagged with custom Codable)
#[derive(Template)]
#[template(path = "swift/union.swift.j2")]
pub struct SwiftUnionTemplate {
    pub name: String,
    pub tag_field: String,
    pub content_field: String,
    pub variants: Vec<SwiftUnionVariantTemplate>,
    pub visibility: String,
    pub imports: Vec<SwiftImport>,
    /// Documentation comment for this union
    pub doc: String,
}

/// Template for rendering a Swift type alias
#[derive(Template)]
#[template(path = "swift/type_alias.swift.j2")]
pub struct SwiftTypeAliasTemplate {
    pub name: String,
    pub target_type: String,
    pub visibility: String,
    pub imports: Vec<SwiftImport>,
    /// Documentation comment for this type alias
    pub doc: String,
}

/// Template for rendering a barrel file (module documentation)
#[derive(Template)]
#[template(path = "swift/barrel.swift.j2")]
pub struct SwiftBarrelTemplate {
    pub modules: Vec<SwiftModuleEntry>,
}

#[derive(Clone)]
pub struct SwiftModuleEntry {
    pub type_name: String,
    pub file_name: String,
}
