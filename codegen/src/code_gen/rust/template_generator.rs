//! Template-based Rust code generator using askama templates

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IRBuilder, IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};
use crate::code_gen::utils::to_snake_case;
use crate::code_gen::validation::{ValidationError, Validator};
use crate::definitions::Definition;

use super::templates::{
    EnumTemplate, FieldTemplate, ListAliasTemplate, MapAliasTemplate, ModTemplate, ModuleEntry,
    StructTemplate, UnionTemplate, UnionVariantTemplate,
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

    /// Generate Rust code from definitions
    pub fn generate(&self, definitions: &[Definition]) -> Result<()> {
        // Build IR
        let schema = IRBuilder::new().build(definitions)?;

        // Validate
        let errors = Validator::new().validate(&schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        // Resolve union variant fields
        let schema = self.resolve_union_variants(schema)?;

        // Generate code for each package
        for (package_name, package) in &schema.packages {
            self.generate_package(package_name, &package.types, &schema)?;
        }

        Ok(())
    }

    fn resolve_union_variants(&self, mut schema: IRSchema) -> Result<IRSchema> {
        // Collect all structs for lookup
        let mut structs: HashMap<String, IRStruct> = HashMap::new();
        for package in schema.packages.values() {
            for ir_type in &package.types {
                if let IRType::Struct(s) = ir_type {
                    structs.insert(s.name.clone(), s.clone());
                }
            }
        }

        // Resolve inline union variants
        for package in schema.packages.values_mut() {
            for ir_type in &mut package.types {
                if let IRType::Union(union) = ir_type {
                    if union.style == IRUnionStyle::Inline {
                        let mut resolved_variants = Vec::new();
                        for variant in &union.variants {
                            match variant {
                                IRUnionVariant::Inline(name, _) => {
                                    if let Some(struct_def) = structs.get(name) {
                                        resolved_variants.push(IRUnionVariant::Inline(
                                            name.clone(),
                                            struct_def.fields.clone(),
                                        ));
                                    } else {
                                        // Treat as unit variant if struct not found
                                        resolved_variants.push(IRUnionVariant::Unit(name.clone()));
                                    }
                                }
                                other @ IRUnionVariant::Unit(_)
                                | other @ IRUnionVariant::Newtype(..) => {
                                    resolved_variants.push(other.clone())
                                }
                            }
                        }
                        union.variants = resolved_variants;
                    }
                }
            }
        }

        Ok(schema)
    }

    fn generate_package(
        &self,
        package_name: &str,
        types: &[IRType],
        schema: &IRSchema,
    ) -> Result<()> {
        let package_path = package_name.replace('.', "/");
        let output_path = format!("{}/{}", self.options.output_dir, package_path);

        self.fs.create_dir_all(&output_path)?;

        if self.options.single_file {
            // Generate all types in mod.rs
            let mod_path = format!("{}/mod.rs", output_path);
            let mut content = String::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                content.push_str(&self.render_type(ir_type, schema)?);
            }

            self.fs.write_file(&mod_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + mod.rs
            let mut modules = Vec::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                let file_name = to_snake_case(ir_type.name());
                let file_path = format!("{}/{}.rs", output_path, file_name);
                let content = self.render_type(ir_type, schema)?;

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

    fn render_type(&self, ir_type: &IRType, schema: &IRSchema) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => self.render_struct(s, schema),
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u, schema),
            IRType::TypeAlias(a) => self.render_type_alias(a, schema),
        }
    }

    fn render_struct(&self, s: &IRStruct, schema: &IRSchema) -> Result<String> {
        let fields: Vec<FieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = StructTemplate {
            derives: self.options.get_derives_string(),
            name: s.name.clone(),
            fields,
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &crate::code_gen::ir::IREnum) -> Result<String> {
        let template = EnumTemplate {
            derives: self.options.get_derives_string(),
            name: e.name.clone(),
            variants: e.variants.clone(),
        };

        Ok(template.render()?)
    }

    fn render_union(&self, u: &IRUnion, schema: &IRSchema) -> Result<String> {
        let variants: Vec<UnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = UnionTemplate {
            derives: self.options.get_derives_string(),
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            variants,
        };

        Ok(template.render()?)
    }

    fn render_type_alias(&self, a: &IRTypeAlias, schema: &IRSchema) -> Result<String> {
        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let template = ListAliasTemplate {
                    name: a.name.clone(),
                    item_type: self.format_type(item_type, schema)?,
                };
                Ok(template.render()?)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let template = MapAliasTemplate {
                    name: a.name.clone(),
                    key_type: self.format_type(key_type, schema)?,
                    value_type: self.format_type(value_type, schema)?,
                };
                Ok(template.render()?)
            }
        }
    }

    fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<FieldTemplate> {
        let mut type_str = self.format_type(&field.field_type, schema)?;

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
        })
    }

    fn convert_union_variant(
        &self,
        variant: &IRUnionVariant,
        schema: &IRSchema,
    ) -> Result<UnionVariantTemplate> {
        match variant {
            IRUnionVariant::Unit(name) => Ok(UnionVariantTemplate::Unit(name.clone())),
            IRUnionVariant::Inline(name, fields) => {
                let field_templates: Vec<FieldTemplate> = fields
                    .iter()
                    .map(|f| self.convert_field(f, schema))
                    .collect::<Result<Vec<_>>>()?;

                Ok(UnionVariantTemplate::Inline {
                    name: name.clone(),
                    fields: field_templates,
                })
            }
            IRUnionVariant::Newtype(name, type_ref) => {
                let type_str = self.get_fqn_for_custom_type(type_ref, schema)?;
                Ok(UnionVariantTemplate::Newtype {
                    name: name.clone(),
                    type_str,
                })
            }
        }
    }

    fn format_type(&self, field_type: &IRFieldType, schema: &IRSchema) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(name) => self.get_fqn_for_custom_type(name, schema),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item, schema)?;
                Ok(format!("Vec<{}>", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!(
                    "std::collections::HashMap<{}, {}>",
                    key_str, value_str
                ))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            IRPrimitive::String => "String".to_string(),
            IRPrimitive::Bool => "bool".to_string(),
            IRPrimitive::DateTime => "DateTime".to_string(),
            IRPrimitive::UInt32 => "u32".to_string(),
            IRPrimitive::UInt64 => "u64".to_string(),
            IRPrimitive::Int32 => "i32".to_string(),
            IRPrimitive::Int64 => "i64".to_string(),
            IRPrimitive::Float32 => "f32".to_string(),
            IRPrimitive::Float64 => "f64".to_string(),
        }
    }

    fn get_fqn_for_custom_type(&self, type_name: &str, schema: &IRSchema) -> Result<String> {
        // Find the type in schema to get its package
        for (package_name, package) in &schema.packages {
            for ir_type in &package.types {
                if ir_type.name() == type_name {
                    let package_path = package_name.replace('.', "::");
                    return Ok(format!("crate::{}::{}", package_path, type_name));
                }
            }
        }

        Err(anyhow!("Unknown type: {}", type_name))
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
