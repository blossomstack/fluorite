//! Template-based Rust code generator using askama templates

use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::doc::slash_doc_lines;
use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias, IRTypeAliasTarget,
    IRTypeRef, IRUnion, IRUnionVariant,
};
use crate::code_gen::utils::to_snake_case;
use crate::code_gen::validation::{ValidationError, Validator};

use super::templates::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, ListAliasTemplate, MapAliasTemplate,
    ModTemplate, ModuleEntry, StructTemplate, UnionTemplate, UnionVariantTemplate,
};
use super::RustOptions;

/// Template-based Rust code generator
pub struct RustTemplateGenerator {
    options: RustOptions,
    fs: Arc<dyn FileSystem>,
}

impl RustTemplateGenerator {
    pub fn new(options: RustOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate Rust code from a pre-built IR schema
    pub fn generate_from_schema(&self, schema: &IRSchema) -> Result<()> {
        // Validate
        let errors = Validator::new().validate(schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        // Generate code for each package
        for (package_name, package) in &schema.packages {
            self.generate_package(package_name, &package.types)?;
        }

        Ok(())
    }

    fn generate_package(&self, package_name: &str, types: &[IRType]) -> Result<()> {
        let package_path = package_name.replace('.', "/");
        let output_path = format!("{}/{}", self.options.output_dir, package_path);

        self.fs.create_dir_all(&output_path)?;

        if self.options.single_file {
            // Generate all types in mod.rs
            let mod_path = format!("{}/mod.rs", output_path);
            let mut content = String::new();

            for ir_type in types.iter() {
                content.push_str(&self.render_type(ir_type)?);
            }

            self.fs.write_file(&mod_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + mod.rs
            let mut modules = Vec::new();

            for ir_type in types.iter() {
                let file_name = to_snake_case(ir_type.name());
                let file_path = format!("{}/{}.rs", output_path, file_name);
                let content = self.render_type(ir_type)?;

                self.fs.write_file(&file_path, content.as_bytes())?;
                modules.push(ModuleEntry { file_name });
            }

            // Generate mod.rs
            let mod_template = ModTemplate {
                package: package_path.replace('/', "::"),
                modules,
            };
            let mod_content = mod_template.render()?;
            let mod_path = format!("{}/mod.rs", output_path);
            self.fs.write_file(&mod_path, mod_content.as_bytes())?;
        }

        Ok(())
    }

    fn render_type(&self, ir_type: &IRType) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => self.render_struct(s),
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u),
            IRType::TypeAlias(a) => self.render_type_alias(a),
        }
    }

    fn render_struct(&self, s: &IRStruct) -> Result<String> {
        let fields: Vec<FieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f))
            .collect::<Result<Vec<_>>>()?;

        let template = StructTemplate {
            derives: self.options.get_derives_string(),
            name: s.name.clone(),
            fields,
            deny_unknown_fields: s.deny_unknown_fields,
            doc: slash_doc_lines(s.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &crate::code_gen::ir::IREnum) -> Result<String> {
        let template = EnumTemplate {
            derives: self.options.get_derives_string(),
            name: e.name.clone(),
            variants: e
                .variants
                .iter()
                .map(|v| EnumVariantTemplate {
                    name: v.name.clone(),
                    doc: slash_doc_lines(v.doc.as_deref()),
                })
                .collect(),
            doc: slash_doc_lines(e.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_union(&self, u: &IRUnion) -> Result<String> {
        let variants: Vec<UnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v))
            .collect::<Result<Vec<_>>>()?;

        let template = UnionTemplate {
            derives: self.options.get_derives_string(),
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            content_field: u.content_field.clone(),
            variants,
            doc: slash_doc_lines(u.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_type_alias(&self, a: &IRTypeAlias) -> Result<String> {
        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let template = ListAliasTemplate {
                    name: a.name.clone(),
                    item_type: self.format_type(item_type)?,
                    doc: slash_doc_lines(a.doc.as_deref()),
                };
                Ok(template.render()?)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let template = MapAliasTemplate {
                    name: a.name.clone(),
                    key_type: self.format_type(key_type)?,
                    value_type: self.format_type(value_type)?,
                    doc: slash_doc_lines(a.doc.as_deref()),
                };
                Ok(template.render()?)
            }
        }
    }

    fn convert_field(&self, field: &IRField) -> Result<FieldTemplate> {
        let mut type_str = self.format_type(&field.field_type)?;

        if field.is_boxed {
            type_str = format!("Box<{}>", type_str);
        }
        if field.is_optional {
            type_str = format!("Option<{}>", type_str);
        }

        Ok(FieldTemplate {
            code_name: field.code_name().to_string(),
            original_name: field.original_name().to_string(),
            type_str,
            is_optional: field.is_optional,
            needs_rename: field.needs_rename(),
            alias: field.alias.clone(),
            default: field.default.clone().unwrap_or_default(),
            skip_if_none: field.skip_if_none,
            skip_if_default: field.skip_if_default,
            flatten: field.flatten,
            doc: slash_doc_lines(field.doc.as_deref()),
            deprecated: field.deprecated,
        })
    }

    fn convert_union_variant(&self, variant: &IRUnionVariant) -> Result<UnionVariantTemplate> {
        let doc = slash_doc_lines(variant.doc());
        match variant {
            IRUnionVariant::Unit { name, .. } => Ok(UnionVariantTemplate::Unit {
                name: name.clone(),
                doc,
            }),
            IRUnionVariant::Newtype {
                name,
                ty: field_type,
                ..
            } => {
                let type_str = self.format_type(field_type)?;
                Ok(UnionVariantTemplate::Newtype {
                    name: name.clone(),
                    type_str,
                    doc,
                })
            }
        }
    }

    fn format_type(&self, field_type: &IRFieldType) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(type_ref) => Ok(self.get_fqn_for_custom_type(type_ref)),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item)?;
                Ok(format!("Vec<{}>", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key)?;
                let value_str = self.format_type(value)?;
                Ok(format!(
                    "std::collections::HashMap<{}, {}>",
                    key_str, value_str
                ))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            // Basic primitives
            IRPrimitive::String => "String".to_string(),
            IRPrimitive::Bool => "bool".to_string(),
            IRPrimitive::DateTime => "chrono::NaiveDateTime".to_string(),
            IRPrimitive::UInt32 => "u32".to_string(),
            IRPrimitive::UInt64 => "u64".to_string(),
            IRPrimitive::Int32 => "i32".to_string(),
            IRPrimitive::Int64 => "i64".to_string(),
            IRPrimitive::Float32 => "f32".to_string(),
            IRPrimitive::Float64 => "f64".to_string(),
            // Extended primitives
            IRPrimitive::UUID => "uuid::Uuid".to_string(),
            IRPrimitive::Decimal => "rust_decimal::Decimal".to_string(),
            IRPrimitive::Bytes => "Vec<u8>".to_string(),
            IRPrimitive::Url => "url::Url".to_string(),
            IRPrimitive::Timestamp => "i64".to_string(),
            IRPrimitive::TimestampMillis => "i64".to_string(),
            IRPrimitive::DateTimeUtc => "chrono::DateTime<chrono::Utc>".to_string(),
            IRPrimitive::DateTimeTz => "chrono::DateTime<chrono::FixedOffset>".to_string(),
            IRPrimitive::Date => "chrono::NaiveDate".to_string(),
            IRPrimitive::Time => "chrono::NaiveTime".to_string(),
            IRPrimitive::Duration => "chrono::Duration".to_string(),
        }
    }

    /// The reference already names its owning package — resolution happened
    /// while lowering the AST, the only place the referencing file's `use`
    /// imports were in scope.
    fn get_fqn_for_custom_type(&self, type_ref: &IRTypeRef) -> String {
        let package_path = type_ref.package.replace('.', "::");
        format!("crate::{}::{}", package_path, type_ref.name)
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
