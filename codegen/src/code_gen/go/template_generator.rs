//! Template-based Go code generator using askama templates.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::doc::doc_lines;
use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IREnum, IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionVariant,
};
use crate::code_gen::utils::{to_camel_case, to_snake_case};
use crate::code_gen::validation::{ValidationError, Validator};

use super::naming::{align_columns, to_go_name};
use super::templates::{
    GoEnumTemplate, GoFileHeaderTemplate, GoStructTemplate, GoTypeAliasTemplate, GoUnionTemplate,
    GoUnionVariantTemplate,
};
use super::GoOptions;

/// Doc comments in Go are plain `//` lines.
fn go_doc_lines(doc: Option<&str>) -> Vec<String> {
    doc_lines(doc, "//")
}

/// Force LF line endings on rendered template output.
///
/// Askama embeds the `.j2` files as they sit on disk, and a Windows checkout
/// converts them to CRLF. Generated Go has to be byte-identical to gofmt
/// output, which is always LF, so normalising here keeps the same schema
/// producing the same bytes on every platform.
fn to_lf(rendered: String) -> String {
    if rendered.contains('\r') {
        rendered.replace("\r\n", "\n").replace('\r', "\n")
    } else {
        rendered
    }
}

/// Template-based Go code generator.
pub struct GoTemplateGenerator {
    options: GoOptions,
    fs: Arc<dyn FileSystem>,
}

impl GoTemplateGenerator {
    pub fn new(options: GoOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate Go code from a pre-built IR schema.
    pub fn generate_from_schema(&self, schema: &IRSchema) -> Result<()> {
        let errors = Validator::new().validate(schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        self.check_flat_package_collisions(schema)?;
        self.check_unsupported_attributes(schema)?;

        // One flat Go package: iterate every type in every .fl package, in a
        // stable order so output does not depend on HashMap iteration.
        let mut types: Vec<&IRType> = Vec::new();
        let mut package_names: Vec<&String> = schema.packages.keys().collect();
        package_names.sort();
        for package_name in package_names {
            if let Some(package) = schema.packages.get(package_name) {
                types.extend(package.types.iter());
            }
        }

        self.fs.create_dir_all(&self.options.output_dir)?;
        let package_name = self.options.resolved_package_name();

        // askama drops each template's trailing newline, so separators and the
        // file's final newline are added here.
        if self.options.single_file {
            let needs_json = types.iter().any(|t| matches!(t, IRType::Union(_)));
            let mut content = to_lf(
                GoFileHeaderTemplate {
                    package_name,
                    needs_json_import: needs_json,
                }
                .render()?,
            );
            for ir_type in &types {
                content.push_str("\n\n");
                content.push_str(&self.render_type(ir_type, schema)?);
            }
            content.push('\n');
            let path = format!("{}/types.go", self.options.output_dir);
            self.fs.write_file(&path, content.as_bytes())?;
        } else {
            for ir_type in &types {
                let needs_json = matches!(ir_type, IRType::Union(_));
                let mut content = to_lf(
                    GoFileHeaderTemplate {
                        package_name: package_name.clone(),
                        needs_json_import: needs_json,
                    }
                    .render()?,
                );
                content.push_str("\n\n");
                content.push_str(&self.render_type(ir_type, schema)?);
                content.push('\n');
                let path = format!(
                    "{}/{}.go",
                    self.options.output_dir,
                    to_snake_case(ir_type.name())
                );
                self.fs.write_file(&path, content.as_bytes())?;
            }
        }

        Ok(())
    }

    /// All `.fl` packages share one Go package, so a type name used twice is a
    /// hard collision. Rust and TypeScript accept it, so report it clearly
    /// rather than letting `go build` fail on redeclaration.
    fn check_flat_package_collisions(&self, schema: &IRSchema) -> Result<()> {
        let mut owners: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (package_name, package) in &schema.packages {
            for ir_type in &package.types {
                owners
                    .entry(ir_type.name())
                    .or_default()
                    .insert(package_name.as_str());
            }
        }

        let clashes: Vec<String> = owners
            .into_iter()
            .filter(|(_, pkgs)| pkgs.len() > 1)
            .map(|(name, pkgs)| {
                format!(
                    "'{}' is defined in {}",
                    name,
                    pkgs.into_iter().collect::<Vec<_>>().join(" and ")
                )
            })
            .collect();

        if clashes.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Go generates one flat package, so type names must be unique across all packages:\n  - {}",
                clashes.join("\n  - ")
            ))
        }
    }

    /// Four IR attributes have no faithful Go struct-tag equivalent. Fail
    /// rather than emit code that silently disagrees with the Rust side.
    fn check_unsupported_attributes(&self, schema: &IRSchema) -> Result<()> {
        let mut problems: Vec<String> = Vec::new();
        let mut package_names: Vec<&String> = schema.packages.keys().collect();
        package_names.sort();

        for package_name in package_names {
            let Some(package) = schema.packages.get(package_name) else {
                continue;
            };
            for ir_type in &package.types {
                let IRType::Struct(s) = ir_type else {
                    continue;
                };
                if s.deny_unknown_fields {
                    problems.push(format!(
                        "struct '{}' uses unsupported attribute 'deny_unknown_fields'",
                        s.name
                    ));
                }
                for f in &s.fields {
                    if f.flatten {
                        problems.push(format!(
                            "field '{}' of '{}' uses unsupported attribute 'flatten'",
                            f.name, s.name
                        ));
                    }
                    if f.default.is_some() {
                        problems.push(format!(
                            "field '{}' of '{}' uses unsupported attribute 'default'",
                            f.name, s.name
                        ));
                    }
                    if !f.alias.is_empty() {
                        problems.push(format!(
                            "field '{}' of '{}' uses unsupported attribute 'alias'",
                            f.name, s.name
                        ));
                    }
                }
            }
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "The Go backend does not support these attributes:\n  - {}",
                problems.join("\n  - ")
            ))
        }
    }

    fn render_type(&self, ir_type: &IRType, schema: &IRSchema) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => self.render_struct(s, schema),
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u, schema),
            IRType::TypeAlias(a) => self.render_type_alias(a, schema),
        }
    }

    /// Lay out rows into gofmt alignment runs.
    ///
    /// gofmt's aligner flushes on any line that is not part of the run, so a
    /// leading comment starts a new one. Each entry is `(comment_lines, cells)`;
    /// the result is tab-indented lines ready for the template.
    fn aligned_block(entries: Vec<(Vec<String>, Vec<String>)>) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut run: Vec<Vec<String>> = Vec::new();

        let flush = |run: &mut Vec<Vec<String>>, lines: &mut Vec<String>| {
            if !run.is_empty() {
                lines.extend(align_columns(run).into_iter().map(|l| format!("\t{}", l)));
                run.clear();
            }
        };

        for (comments, cells) in entries {
            if !comments.is_empty() {
                flush(&mut run, &mut lines);
                for comment in comments {
                    lines.push(format!("\t{}", comment));
                }
            }
            run.push(cells);
        }
        flush(&mut run, &mut lines);
        lines
    }

    fn render_struct(&self, s: &IRStruct, schema: &IRSchema) -> Result<String> {
        let mut entries = Vec::new();
        for f in &s.fields {
            let mut comments = go_doc_lines(f.doc.as_deref());
            if f.deprecated {
                comments.push("// Deprecated: this field is deprecated.".to_string());
            }
            entries.push((
                comments,
                vec![
                    to_go_name(&f.name),
                    self.format_field_type(f, schema)?,
                    format!("`json:\"{}\"`", Self::json_tag(f)),
                ],
            ));
        }

        let template = GoStructTemplate {
            name: s.name.clone(),
            doc: go_doc_lines(s.doc.as_deref()),
            field_lines: Self::aligned_block(entries),
        };
        Ok(to_lf(template.render()?))
    }

    /// The wire name, following the Rust backend: an explicit rename wins
    /// verbatim, otherwise the camelCase of the field name.
    fn json_tag(f: &IRField) -> String {
        let base = if f.needs_rename() {
            f.original_name().to_string()
        } else {
            to_camel_case(&f.name)
        };
        // The Rust template skips every `None`, and `skip_if_default` is
        // exactly Go's `omitempty`.
        if f.is_optional || f.skip_if_default {
            format!("{},omitempty", base)
        } else {
            base
        }
    }

    fn format_field_type(&self, f: &IRField, schema: &IRSchema) -> Result<String> {
        let base = self.format_type(&f.field_type, schema)?;
        if f.is_optional {
            Ok(format!("*{}", base))
        } else {
            Ok(base)
        }
    }

    fn render_enum(&self, e: &IREnum) -> Result<String> {
        let entries: Vec<(Vec<String>, Vec<String>)> = e
            .variants
            .iter()
            .map(|v| {
                (
                    go_doc_lines(v.doc.as_deref()),
                    vec![
                        format!("{}{}", e.name, to_go_name(&v.name)),
                        e.name.clone(),
                        format!("= \"{}\"", v.name),
                    ],
                )
            })
            .collect();

        let template = GoEnumTemplate {
            name: e.name.clone(),
            doc: go_doc_lines(e.doc.as_deref()),
            constant_lines: Self::aligned_block(entries),
        };
        Ok(to_lf(template.render()?))
    }

    fn render_type_alias(&self, a: &IRTypeAlias, schema: &IRSchema) -> Result<String> {
        let target_type = match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                format!("[]{}", self.format_type(item_type, schema)?)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => format!(
                "map[{}]{}",
                self.format_type(key_type, schema)?,
                self.format_type(value_type, schema)?
            ),
        };

        let template = GoTypeAliasTemplate {
            name: a.name.clone(),
            doc: go_doc_lines(a.doc.as_deref()),
            target_type,
        };
        Ok(to_lf(template.render()?))
    }

    fn render_union(&self, u: &IRUnion, schema: &IRSchema) -> Result<String> {
        let mut variants = Vec::new();
        for variant in &u.variants {
            let doc = go_doc_lines(variant.doc());
            match variant {
                IRUnionVariant::Unit { name, .. } => variants.push(GoUnionVariantTemplate::Unit {
                    struct_name: format!("{}{}", u.name, to_go_name(name)),
                    wire_name: name.clone(),
                    doc,
                }),
                IRUnionVariant::Newtype { name, ty, .. } => {
                    let type_str = self.format_type(ty, schema)?;
                    // The anonymous struct passed to json.Marshal is
                    // gofmt-aligned like any other struct.
                    let marshal_lines = align_columns(&[
                        vec![
                            "Type".to_string(),
                            "string".to_string(),
                            format!("`json:\"{}\"`", u.tag_field),
                        ],
                        vec![
                            "Value".to_string(),
                            type_str.clone(),
                            format!("`json:\"{}\"`", u.content_field),
                        ],
                    ]);
                    variants.push(GoUnionVariantTemplate::Newtype {
                        struct_name: format!("{}{}", u.name, to_go_name(name)),
                        wire_name: name.clone(),
                        type_str,
                        marshal_lines,
                        doc,
                    });
                }
            }
        }

        let envelope_lines = align_columns(&[
            vec![
                "Type".to_string(),
                "string".to_string(),
                format!("`json:\"{}\"`", u.tag_field),
            ],
            vec![
                "Value".to_string(),
                "json.RawMessage".to_string(),
                format!("`json:\"{}\"`", u.content_field),
            ],
        ]);

        let template = GoUnionTemplate {
            name: u.name.clone(),
            doc: go_doc_lines(u.doc.as_deref()),
            tag_field: u.tag_field.clone(),
            content_field: u.content_field.clone(),
            variants,
            envelope_lines,
            tag_go_name: "Type".to_string(),
            content_go_name: "Value".to_string(),
        };
        Ok(to_lf(template.render()?))
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn format_type(&self, field_type: &IRFieldType, schema: &IRSchema) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(name) => Ok(name.clone()),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item, schema)?;
                Ok(format!("[]{}", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!("map[{}]{}", key_str, value_str))
            }
        }
    }

    /// Extended primitives follow the TypeScript backend's wire treatment:
    /// everything is a string on the wire except the two timestamp forms.
    /// Keeping them as `string` and `int64` is what makes the generated
    /// package stdlib-only.
    pub fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            IRPrimitive::String => "string".to_string(),
            IRPrimitive::Bool => "bool".to_string(),
            IRPrimitive::UInt32 => "uint32".to_string(),
            IRPrimitive::UInt64 => "uint64".to_string(),
            IRPrimitive::Int32 => "int32".to_string(),
            IRPrimitive::Int64 => "int64".to_string(),
            IRPrimitive::Float32 => "float32".to_string(),
            IRPrimitive::Float64 => "float64".to_string(),
            IRPrimitive::UUID
            | IRPrimitive::Decimal
            | IRPrimitive::Bytes
            | IRPrimitive::Url
            | IRPrimitive::DateTime
            | IRPrimitive::DateTimeUtc
            | IRPrimitive::DateTimeTz
            | IRPrimitive::Date
            | IRPrimitive::Time
            | IRPrimitive::Duration => "string".to_string(),
            IRPrimitive::Timestamp | IRPrimitive::TimestampMillis => "int64".to_string(),
        }
    }

    fn format_validation_errors(&self, errors: &[ValidationError]) -> anyhow::Error {
        let messages: Vec<String> = errors
            .iter()
            .map(|e| match e {
                ValidationError::UnknownType {
                    type_name,
                    referenced_from,
                    field_name,
                } => {
                    if let Some(field) = field_name {
                        format!(
                            "Unknown type '{}' in field '{}' of '{}'",
                            type_name, field, referenced_from
                        )
                    } else {
                        format!(
                            "Unknown type '{}' referenced from '{}'",
                            type_name, referenced_from
                        )
                    }
                }
                ValidationError::DuplicateType { type_name, package } => {
                    format!("Duplicate type '{}' in package '{}'", type_name, package)
                }
                ValidationError::CircularDependency { cycle } => {
                    format!("Circular dependency: {}", cycle.join(" -> "))
                }
                ValidationError::EmptyEnum { type_name } => {
                    format!("Empty enum '{}'", type_name)
                }
                ValidationError::EmptyStruct { type_name } => {
                    format!("Empty struct '{}'", type_name)
                }
                ValidationError::EmptyUnion { type_name } => {
                    format!("Empty union '{}'", type_name)
                }
                ValidationError::InvalidUnionVariant {
                    union_name,
                    variant_name,
                    reason,
                } => {
                    format!(
                        "Invalid variant '{}' in union '{}': {}",
                        variant_name, union_name, reason
                    )
                }
            })
            .collect();

        anyhow!("Validation errors:\n  - {}", messages.join("\n  - "))
    }
}

#[cfg(test)]
mod tests {
    use super::to_lf;

    #[test]
    fn crlf_templates_still_produce_lf_output() {
        assert_eq!(to_lf("a\r\nb\r\n".to_string()), "a\nb\n");
        assert_eq!(to_lf("a\rb".to_string()), "a\nb");
    }

    #[test]
    fn lf_output_is_left_alone() {
        assert_eq!(to_lf("a\nb\n".to_string()), "a\nb\n");
    }
}
