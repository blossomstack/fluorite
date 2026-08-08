use askama::Template;

/// Package clause and, when the file contains a union, its imports.
#[derive(Template)]
#[template(path = "go/file_header.go.j2")]
pub struct GoFileHeaderTemplate {
    pub package_name: String,
    pub needs_json_import: bool,
}

/// A struct. `field_lines` are fully rendered, tab-indented lines including
/// any doc comments — gofmt alignment runs cannot be computed in a template.
#[derive(Template)]
#[template(path = "go/struct.go.j2")]
pub struct GoStructTemplate {
    pub name: String,
    pub doc: Vec<String>,
    pub field_lines: Vec<String>,
}

/// A string-typed enum. `constant_lines` are rendered the same way as a
/// struct's fields, for the same reason.
#[derive(Template)]
#[template(path = "go/enum.go.j2")]
pub struct GoEnumTemplate {
    pub name: String,
    pub doc: Vec<String>,
    pub constant_lines: Vec<String>,
}

/// A list or map type alias.
#[derive(Template)]
#[template(path = "go/type_alias.go.j2")]
pub struct GoTypeAliasTemplate {
    pub name: String,
    pub doc: Vec<String>,
    pub target_type: String,
}

/// One union variant, pre-rendered for the union template.
#[derive(Clone)]
pub enum GoUnionVariantTemplate {
    /// `{ "type": "Deleted" }`
    Unit {
        /// The Go type name, `{Union}{Variant}`.
        struct_name: String,
        /// The variant name as it appears on the wire.
        wire_name: String,
        doc: Vec<String>,
    },
    /// `{ "type": "Created", "value": ... }`
    Newtype {
        struct_name: String,
        wire_name: String,
        type_str: String,
        /// Aligned field lines for the anonymous struct in `MarshalJSON`.
        marshal_lines: Vec<String>,
        doc: Vec<String>,
    },
}

/// An adjacently tagged union: sealed interface, variant types, and the
/// wrapper's `MarshalJSON` / `UnmarshalJSON`.
#[derive(Template)]
#[template(path = "go/union.go.j2")]
pub struct GoUnionTemplate {
    pub name: String,
    pub doc: Vec<String>,
    pub tag_field: String,
    pub content_field: String,
    pub variants: Vec<GoUnionVariantTemplate>,
    /// Aligned field lines for the anonymous envelope struct in `UnmarshalJSON`.
    pub envelope_lines: Vec<String>,
    /// Always `Type`; the envelope is a local struct, so only its tag varies.
    pub tag_go_name: String,
    /// Always `Value`.
    pub content_go_name: String,
}
