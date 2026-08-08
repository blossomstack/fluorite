//! Template-based TypeScript code generator using askama templates

use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::doc::block_doc_lines;
use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias, IRTypeAliasTarget,
    IRUnion, IRUnionVariant,
};
use crate::code_gen::utils::to_camel_case;
use crate::code_gen::validation::{ValidationError, Validator};

use super::templates::{
    InterfaceTemplate, TsEnumTemplate, TsFieldTemplate, TsImport, TsIndexTemplate, TsModuleEntry,
    TsTypeAliasTemplate, TsUnionTemplate, TsUnionVariantTemplate,
};
use super::TypeScriptOptions;

/// Template-based TypeScript code generator
pub struct TsTemplateGenerator {
    options: TypeScriptOptions,
    fs: Arc<dyn FileSystem>,
}

impl TsTemplateGenerator {
    pub fn new(options: TypeScriptOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate TypeScript code from a pre-built IR schema
    pub fn generate_from_schema(&self, schema: &IRSchema) -> Result<()> {
        // Validate
        let errors = Validator::new().validate(schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        // Generate code for each package
        for (package_name, package) in &schema.packages {
            // Use override if provided, otherwise use the package name from schema
            let output_package_name = self.options.package_name.as_ref().unwrap_or(package_name);
            self.generate_package(output_package_name, &package.types, schema)?;
        }

        Ok(())
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

        // Collect all type names in this package for import resolution
        let package_type_names: std::collections::HashSet<String> =
            types.iter().map(|t| t.name().to_string()).collect();

        if self.options.single_file {
            // Generate all types in index.ts
            let index_path = format!("{}/index.ts", output_path);
            let mut content = String::new();

            // In single-file mode, we still need cross-package imports
            for ir_type in types.iter() {
                content.push_str(&self.render_type(
                    ir_type,
                    schema,
                    package_name,
                    None, // No same-package imports in single-file mode
                )?);
            }

            self.fs.write_file(&index_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + index.ts
            let mut modules = Vec::new();

            for ir_type in types.iter() {
                let file_name = to_camel_case(ir_type.name());
                let file_path = format!("{}/{}.ts", output_path, file_name);
                let content =
                    self.render_type(ir_type, schema, package_name, Some(&package_type_names))?;

                self.fs.write_file(&file_path, content.as_bytes())?;
                modules.push(TsModuleEntry { file_name });
            }

            // Generate index.ts
            let index_template = TsIndexTemplate { modules };
            let index_content = index_template.render()?;
            let index_path = format!("{}/index.ts", output_path);
            self.fs.write_file(&index_path, index_content.as_bytes())?;
        }

        Ok(())
    }

    fn render_type(
        &self,
        ir_type: &IRType,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => {
                self.render_interface(s, schema, current_package, package_type_names)
            }
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u, schema, current_package, package_type_names),
            IRType::TypeAlias(a) => {
                self.render_type_alias(a, schema, current_package, package_type_names)
            }
        }
    }

    fn render_interface(
        &self,
        s: &IRStruct,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Result<String> {
        let fields: Vec<TsFieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f, schema))
            .collect::<Result<Vec<_>>>()?;

        // Collect imports
        let imports =
            self.collect_imports_for_struct(s, schema, current_package, package_type_names);

        let template = InterfaceTemplate {
            name: s.name.clone(),
            fields,
            use_readonly: self.options.use_readonly,
            imports,
            doc: block_doc_lines(s.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &crate::code_gen::ir::IREnum) -> Result<String> {
        let template = TsEnumTemplate {
            name: e.name.clone(),
            variants: e.variants.clone(),
            doc: block_doc_lines(e.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_union(
        &self,
        u: &IRUnion,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Result<String> {
        let variants: Vec<TsUnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v, schema))
            .collect::<Result<Vec<_>>>()?;

        // Collect imports
        let imports =
            self.collect_imports_for_union(u, schema, current_package, package_type_names);

        let template = TsUnionTemplate {
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            content_field: u.content_field.clone(),
            variants,
            imports,
            doc: block_doc_lines(u.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_type_alias(
        &self,
        a: &IRTypeAlias,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Result<String> {
        let target_type = match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let item_str = self.format_type(item_type, schema)?;
                format!("{}[]", item_str)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let key_str = self.format_type(key_type, schema)?;
                let value_str = self.format_type(value_type, schema)?;
                format!("Record<{}, {}>", key_str, value_str)
            }
        };

        // Collect imports
        let imports =
            self.collect_imports_for_type_alias(a, schema, current_package, package_type_names);

        let template = TsTypeAliasTemplate {
            name: a.name.clone(),
            target_type,
            imports,
            doc: block_doc_lines(a.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<TsFieldTemplate> {
        let type_str = self.format_type(&field.field_type, schema)?;

        // Use camelCase for TypeScript field names
        let code_name = if let Some(rename) = &field.rename {
            to_camel_case(rename)
        } else {
            to_camel_case(&field.name)
        };

        Ok(TsFieldTemplate {
            code_name,
            type_str,
            is_optional: field.is_optional,
            doc: block_doc_lines(field.doc.as_deref()),
            deprecated: field.deprecated,
        })
    }

    fn convert_union_variant(
        &self,
        variant: &IRUnionVariant,
        schema: &IRSchema,
    ) -> Result<TsUnionVariantTemplate> {
        let doc = block_doc_lines(variant.doc());
        match variant {
            IRUnionVariant::Unit { name, .. } => Ok(TsUnionVariantTemplate::Unit {
                name: name.clone(),
                doc,
            }),
            IRUnionVariant::Newtype {
                name,
                ty: field_type,
                ..
            } => {
                let type_str = self.format_type(field_type, schema)?;
                Ok(TsUnionVariantTemplate::Newtype {
                    name: name.clone(),
                    type_str,
                    doc,
                })
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn format_type(&self, field_type: &IRFieldType, schema: &IRSchema) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(name) => Ok(name.clone()),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item, schema)?;
                Ok(format!("{}[]", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!("Record<{}, {}>", key_str, value_str))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            // Basic primitives
            IRPrimitive::String => "string".to_string(),
            IRPrimitive::Bool => "boolean".to_string(),
            IRPrimitive::DateTime => "string".to_string(),
            IRPrimitive::UInt32
            | IRPrimitive::UInt64
            | IRPrimitive::Int32
            | IRPrimitive::Int64
            | IRPrimitive::Float32
            | IRPrimitive::Float64 => "number".to_string(),
            // Extended primitives - all serialize as strings in JSON except timestamps
            IRPrimitive::UUID => "string".to_string(),
            IRPrimitive::Decimal => "string".to_string(),
            IRPrimitive::Bytes => "string".to_string(), // base64 encoded
            IRPrimitive::Url => "string".to_string(),
            IRPrimitive::Timestamp => "number".to_string(), // Unix epoch seconds
            IRPrimitive::TimestampMillis => "number".to_string(), // Unix epoch milliseconds
            IRPrimitive::DateTimeUtc => "string".to_string(), // ISO 8601
            IRPrimitive::DateTimeTz => "string".to_string(), // ISO 8601 with timezone
            IRPrimitive::Date => "string".to_string(),      // ISO 8601 date
            IRPrimitive::Time => "string".to_string(),      // ISO 8601 time
            IRPrimitive::Duration => "string".to_string(),  // ISO 8601 duration
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

    // Import collection methods

    /// Find which package a type is defined in
    fn get_package_for_type(&self, type_name: &str, schema: &IRSchema) -> Option<String> {
        for (package_name, package) in &schema.packages {
            for ir_type in &package.types {
                if ir_type.name() == type_name {
                    return Some(package_name.clone());
                }
            }
        }
        None
    }

    /// Calculate relative import path between packages
    /// e.g., from "demo.orders" to "demo.common" → "../common"
    fn calculate_relative_path(&self, from_pkg: &str, to_pkg: &str) -> String {
        if from_pkg == to_pkg {
            return ".".to_string();
        }

        let from_parts: Vec<&str> = from_pkg.split('.').collect();
        let to_parts: Vec<&str> = to_pkg.split('.').collect();

        // Find common prefix length
        let common = from_parts
            .iter()
            .zip(&to_parts)
            .take_while(|(a, b)| a == b)
            .count();

        // Build relative path: go up from current, then down to target
        let ups = from_parts.len() - common;
        let downs = &to_parts[common..];

        let mut path_parts: Vec<&str> = vec![".."; ups];
        path_parts.extend(downs.iter().copied());
        path_parts.join("/")
    }

    fn collect_imports_for_struct(
        &self,
        s: &IRStruct,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<TsImport> {
        let mut same_package_types = std::collections::HashSet::new();
        let mut cross_package_types = std::collections::HashMap::new();

        for field in &s.fields {
            self.collect_type_references(
                &field.field_type,
                schema,
                current_package,
                package_type_names,
                &mut same_package_types,
                &mut cross_package_types,
            );
        }

        self.build_imports(current_package, same_package_types, cross_package_types)
    }

    fn collect_imports_for_union(
        &self,
        u: &IRUnion,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<TsImport> {
        let mut same_package_types = std::collections::HashSet::new();
        let mut cross_package_types = std::collections::HashMap::new();

        for variant in &u.variants {
            match variant {
                IRUnionVariant::Unit { .. } => {}
                IRUnionVariant::Newtype { ty: field_type, .. } => {
                    self.collect_type_references(
                        field_type,
                        schema,
                        current_package,
                        package_type_names,
                        &mut same_package_types,
                        &mut cross_package_types,
                    );
                }
            }
        }

        self.build_imports(current_package, same_package_types, cross_package_types)
    }

    fn collect_imports_for_type_alias(
        &self,
        a: &IRTypeAlias,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
    ) -> Vec<TsImport> {
        let mut same_package_types = std::collections::HashSet::new();
        let mut cross_package_types = std::collections::HashMap::new();

        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                self.collect_type_references(
                    item_type,
                    schema,
                    current_package,
                    package_type_names,
                    &mut same_package_types,
                    &mut cross_package_types,
                );
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                self.collect_type_references(
                    key_type,
                    schema,
                    current_package,
                    package_type_names,
                    &mut same_package_types,
                    &mut cross_package_types,
                );
                self.collect_type_references(
                    value_type,
                    schema,
                    current_package,
                    package_type_names,
                    &mut same_package_types,
                    &mut cross_package_types,
                );
            }
        }

        self.build_imports(current_package, same_package_types, cross_package_types)
    }

    fn collect_type_references(
        &self,
        field_type: &IRFieldType,
        schema: &IRSchema,
        current_package: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
        same_package_types: &mut std::collections::HashSet<String>,
        cross_package_types: &mut std::collections::HashMap<String, String>,
    ) {
        match field_type {
            IRFieldType::Primitive(_) | IRFieldType::Any => {}
            IRFieldType::Custom(name) => {
                // Check if it's a same-package type (only in multi-file mode)
                if let Some(pkg_types) = package_type_names {
                    if pkg_types.contains(name) {
                        same_package_types.insert(name.clone());
                        return;
                    }
                }

                // Check if it's a cross-package type
                if let Some(source_pkg) = self.get_package_for_type(name, schema) {
                    if source_pkg != current_package {
                        cross_package_types.insert(name.clone(), source_pkg);
                    }
                }
            }
            IRFieldType::List(item) => {
                self.collect_type_references(
                    item,
                    schema,
                    current_package,
                    package_type_names,
                    same_package_types,
                    cross_package_types,
                );
            }
            IRFieldType::Map(key, value) => {
                self.collect_type_references(
                    key,
                    schema,
                    current_package,
                    package_type_names,
                    same_package_types,
                    cross_package_types,
                );
                self.collect_type_references(
                    value,
                    schema,
                    current_package,
                    package_type_names,
                    same_package_types,
                    cross_package_types,
                );
            }
        }
    }

    fn build_imports(
        &self,
        current_package: &str,
        same_package_types: std::collections::HashSet<String>,
        cross_package_types: std::collections::HashMap<String, String>,
    ) -> Vec<TsImport> {
        let mut imports = Vec::new();

        // Same-package imports (multi-file mode only) - import from specific file
        for name in same_package_types {
            imports.push(TsImport {
                name: name.clone(),
                path: format!("./{}", to_camel_case(&name)),
            });
        }

        // Cross-package imports - import from package index
        for (name, source_pkg) in cross_package_types {
            let relative_path = self.calculate_relative_path(current_package, &source_pkg);
            imports.push(TsImport {
                name,
                path: relative_path,
            });
        }

        imports.sort_by(|a, b| a.path.cmp(&b.path).then(a.name.cmp(&b.name)));
        imports
    }
}
