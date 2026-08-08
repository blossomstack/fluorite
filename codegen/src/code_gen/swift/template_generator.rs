//! Template-based Swift code generator using askama templates

use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::doc::slash_doc_lines;
use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IREnum, IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionVariant,
};
use crate::code_gen::utils::{to_camel_case, to_pascal_case};
use crate::code_gen::validation::{ValidationError, Validator};

use super::options::SwiftOptions;
use super::templates::{
    SwiftBarrelTemplate, SwiftEnumTemplate, SwiftEnumVariant, SwiftFieldTemplate, SwiftImport,
    SwiftModuleEntry, SwiftStructTemplate, SwiftTypeAliasTemplate, SwiftUnionTemplate,
    SwiftUnionVariantTemplate,
};

/// Template-based Swift code generator
pub struct SwiftTemplateGenerator {
    options: SwiftOptions,
    fs: Arc<dyn FileSystem>,
}

impl SwiftTemplateGenerator {
    pub fn new(options: SwiftOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate Swift code from a pre-built IR schema
    pub fn generate_from_schema(&self, schema: &IRSchema) -> Result<()> {
        // Validate
        let errors = Validator::new().validate(schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        // Generate code for each package
        for (package_name, package) in &schema.packages {
            self.generate_package(package_name, &package.types, schema)?;
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

        // Get visibility string
        let visibility = self.options.get_visibility_string().to_string();

        if self.options.single_file {
            // Generate all types in a single file
            let barrel_name =
                to_pascal_case(package_name.split('.').next_back().unwrap_or(package_name));
            let file_path = format!("{}/{}.swift", output_path, barrel_name);
            let mut content = String::new();

            // Add import header once
            let needs_runtime = self.package_needs_runtime(types, schema);
            if needs_runtime {
                content.push_str("import Foundation\nimport FluoriteRuntime\n\n");
            } else {
                content.push_str("import Foundation\n\n");
            }

            for ir_type in types.iter() {
                content.push_str(&self.render_type(ir_type, schema, &visibility, None, true)?);
                content.push('\n');
            }

            self.fs.write_file(&file_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + barrel file
            let mut modules = Vec::new();

            for ir_type in types.iter() {
                let file_name = to_pascal_case(ir_type.name());
                let file_path = format!("{}/{}.swift", output_path, file_name);
                let content = self.render_type(
                    ir_type,
                    schema,
                    &visibility,
                    Some(&package_type_names),
                    false,
                )?;

                self.fs.write_file(&file_path, content.as_bytes())?;
                modules.push(SwiftModuleEntry {
                    type_name: ir_type.name().to_string(),
                    file_name,
                });
            }

            // Generate barrel file (documentation only in Swift)
            let barrel_name =
                to_pascal_case(package_name.split('.').next_back().unwrap_or(package_name));
            let barrel_template = SwiftBarrelTemplate { modules };
            let barrel_content = barrel_template.render()?;
            let barrel_path = format!("{}/{}.swift", output_path, barrel_name);
            self.fs
                .write_file(&barrel_path, barrel_content.as_bytes())?;
        }

        Ok(())
    }

    fn package_needs_runtime(&self, types: &[IRType], schema: &IRSchema) -> bool {
        for ir_type in types {
            if self.type_needs_runtime(ir_type, schema) {
                return true;
            }
        }
        false
    }

    fn type_needs_runtime(&self, ir_type: &IRType, _schema: &IRSchema) -> bool {
        match ir_type {
            IRType::Struct(s) => s
                .fields
                .iter()
                .any(|f| self.field_type_needs_runtime(&f.field_type)),
            IRType::Union(u) => u.variants.iter().any(|v| match v {
                IRUnionVariant::Unit { .. } => false,
                IRUnionVariant::Newtype { ty: ft, .. } => self.field_type_needs_runtime(ft),
            }),
            IRType::TypeAlias(a) => match &a.target {
                IRTypeAliasTarget::List(item) => self.field_type_needs_runtime(item),
                IRTypeAliasTarget::Map(k, v) => {
                    self.field_type_needs_runtime(k) || self.field_type_needs_runtime(v)
                }
            },
            IRType::Enum(_) => false,
        }
    }

    fn field_type_needs_runtime(&self, field_type: &IRFieldType) -> bool {
        match field_type {
            IRFieldType::Any => true,
            IRFieldType::List(item) => self.field_type_needs_runtime(item),
            IRFieldType::Map(k, v) => {
                self.field_type_needs_runtime(k) || self.field_type_needs_runtime(v)
            }
            IRFieldType::Primitive(_) | IRFieldType::Custom(_) => false,
        }
    }

    fn render_type(
        &self,
        ir_type: &IRType,
        schema: &IRSchema,
        visibility: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
        skip_imports: bool,
    ) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => {
                self.render_struct(s, schema, visibility, package_type_names, skip_imports)
            }
            IRType::Enum(e) => self.render_enum(e, visibility),
            IRType::Union(u) => {
                self.render_union(u, schema, visibility, package_type_names, skip_imports)
            }
            IRType::TypeAlias(a) => {
                self.render_type_alias(a, schema, visibility, package_type_names, skip_imports)
            }
        }
    }

    fn render_struct(
        &self,
        s: &IRStruct,
        schema: &IRSchema,
        visibility: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
        skip_imports: bool,
    ) -> Result<String> {
        let fields: Vec<SwiftFieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f, schema))
            .collect::<Result<Vec<_>>>()?;

        // Check if any field needs renaming (for CodingKeys)
        let needs_coding_keys = fields.iter().any(|f| f.needs_rename);

        // Collect imports for multi-file mode
        let imports = if skip_imports {
            Vec::new()
        } else if let Some(type_names) = package_type_names {
            self.collect_imports_for_struct(s, type_names)
        } else {
            Vec::new()
        };

        let template = SwiftStructTemplate {
            name: s.name.clone(),
            fields,
            visibility: visibility.to_string(),
            needs_coding_keys,
            imports,
            doc: slash_doc_lines(s.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &IREnum, visibility: &str) -> Result<String> {
        let variants: Vec<SwiftEnumVariant> = e
            .variants
            .iter()
            .map(|v| {
                let code_name = to_camel_case(v);
                let needs_rename = code_name != *v;
                SwiftEnumVariant {
                    code_name,
                    original_name: v.clone(),
                    needs_rename,
                }
            })
            .collect();

        let template = SwiftEnumTemplate {
            name: e.name.clone(),
            variants,
            visibility: visibility.to_string(),
            doc: slash_doc_lines(e.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_union(
        &self,
        u: &IRUnion,
        schema: &IRSchema,
        visibility: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
        skip_imports: bool,
    ) -> Result<String> {
        let variants: Vec<SwiftUnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v, schema))
            .collect::<Result<Vec<_>>>()?;

        // Collect imports for multi-file mode
        let imports = if skip_imports {
            Vec::new()
        } else if let Some(type_names) = package_type_names {
            self.collect_imports_for_union(u, type_names)
        } else {
            Vec::new()
        };

        let template = SwiftUnionTemplate {
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            content_field: u.content_field.clone(),
            variants,
            visibility: visibility.to_string(),
            imports,
            doc: slash_doc_lines(u.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn render_type_alias(
        &self,
        a: &IRTypeAlias,
        schema: &IRSchema,
        visibility: &str,
        package_type_names: Option<&std::collections::HashSet<String>>,
        skip_imports: bool,
    ) -> Result<String> {
        let target_type = match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let item_str = self.format_type(item_type, schema)?;
                format!("[{}]", item_str)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let key_str = self.format_type(key_type, schema)?;
                let value_str = self.format_type(value_type, schema)?;
                format!("[{}: {}]", key_str, value_str)
            }
        };

        // Collect imports for multi-file mode
        let imports = if skip_imports {
            Vec::new()
        } else if let Some(type_names) = package_type_names {
            self.collect_imports_for_type_alias(a, type_names)
        } else {
            Vec::new()
        };

        let template = SwiftTypeAliasTemplate {
            name: a.name.clone(),
            target_type,
            visibility: visibility.to_string(),
            imports,
            doc: slash_doc_lines(a.doc.as_deref()),
        };

        Ok(template.render()?)
    }

    fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<SwiftFieldTemplate> {
        let base_type = self.format_type(&field.field_type, schema)?;
        let type_str = if field.is_optional {
            format!("{}?", base_type)
        } else {
            base_type
        };

        // Swift property name is always camelCase
        let code_name = to_camel_case(&field.name);

        // JSON key is determined by: explicit rename > camelCase
        let json_key = if let Some(rename) = &field.rename {
            rename.clone()
        } else {
            to_camel_case(&field.name)
        };

        let needs_rename = code_name != json_key;

        Ok(SwiftFieldTemplate {
            code_name,
            original_name: json_key,
            type_str,
            needs_rename,
            doc: slash_doc_lines(field.doc.as_deref()),
            deprecated: field.deprecated,
        })
    }

    fn convert_union_variant(
        &self,
        variant: &IRUnionVariant,
        schema: &IRSchema,
    ) -> Result<SwiftUnionVariantTemplate> {
        let doc = slash_doc_lines(variant.doc());
        match variant {
            IRUnionVariant::Unit { name, .. } => Ok(SwiftUnionVariantTemplate::Unit {
                case_name: to_camel_case(name),
                serialized_name: name.clone(),
                doc,
            }),
            IRUnionVariant::Newtype {
                name,
                ty: field_type,
                ..
            } => {
                let type_str = self.format_type(field_type, schema)?;
                Ok(SwiftUnionVariantTemplate::Newtype {
                    case_name: to_camel_case(name),
                    serialized_name: name.clone(),
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
                Ok(format!("[{}]", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!("[{}: {}]", key_str, value_str))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            // Basic primitives
            IRPrimitive::String => "String".to_string(),
            IRPrimitive::Bool => "Bool".to_string(),
            IRPrimitive::Int32 => "Int32".to_string(),
            IRPrimitive::Int64 => "Int64".to_string(),
            IRPrimitive::UInt32 => "UInt32".to_string(),
            IRPrimitive::UInt64 => "UInt64".to_string(),
            IRPrimitive::Float32 => "Float".to_string(),
            IRPrimitive::Float64 => "Double".to_string(),
            // Foundation types
            IRPrimitive::UUID => "UUID".to_string(),
            IRPrimitive::Decimal => "Decimal".to_string(),
            IRPrimitive::Bytes => "Data".to_string(),
            IRPrimitive::Url => "URL".to_string(),
            // Date/Time types
            IRPrimitive::DateTime => "Date".to_string(),
            IRPrimitive::DateTimeUtc => "Date".to_string(),
            IRPrimitive::DateTimeTz => "Date".to_string(),
            IRPrimitive::Timestamp => "Date".to_string(),
            IRPrimitive::TimestampMillis => "Date".to_string(),
            IRPrimitive::Duration => "TimeInterval".to_string(),
            // String-based types
            IRPrimitive::Date => "String".to_string(), // ISO8601 date string
            IRPrimitive::Time => "String".to_string(), // ISO8601 time string
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

    // Import collection methods for multi-file mode

    fn collect_imports_for_struct(
        &self,
        s: &IRStruct,
        package_type_names: &std::collections::HashSet<String>,
    ) -> Vec<SwiftImport> {
        let mut referenced_types = std::collections::HashSet::new();

        for field in &s.fields {
            self.collect_type_references(
                &field.field_type,
                package_type_names,
                &mut referenced_types,
            );
        }

        // Check if any field uses Any type
        let needs_runtime = s
            .fields
            .iter()
            .any(|f| self.field_type_needs_runtime(&f.field_type));

        self.build_imports(referenced_types, needs_runtime)
    }

    fn collect_imports_for_union(
        &self,
        u: &IRUnion,
        package_type_names: &std::collections::HashSet<String>,
    ) -> Vec<SwiftImport> {
        let mut referenced_types = std::collections::HashSet::new();
        let mut needs_runtime = false;

        for variant in &u.variants {
            match variant {
                IRUnionVariant::Unit { .. } => {}
                IRUnionVariant::Newtype { ty: field_type, .. } => {
                    self.collect_type_references(
                        field_type,
                        package_type_names,
                        &mut referenced_types,
                    );
                    if self.field_type_needs_runtime(field_type) {
                        needs_runtime = true;
                    }
                }
            }
        }

        self.build_imports(referenced_types, needs_runtime)
    }

    fn collect_imports_for_type_alias(
        &self,
        a: &IRTypeAlias,
        package_type_names: &std::collections::HashSet<String>,
    ) -> Vec<SwiftImport> {
        let mut referenced_types = std::collections::HashSet::new();
        let mut needs_runtime = false;

        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                self.collect_type_references(item_type, package_type_names, &mut referenced_types);
                if self.field_type_needs_runtime(item_type) {
                    needs_runtime = true;
                }
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                self.collect_type_references(key_type, package_type_names, &mut referenced_types);
                self.collect_type_references(value_type, package_type_names, &mut referenced_types);
                if self.field_type_needs_runtime(key_type)
                    || self.field_type_needs_runtime(value_type)
                {
                    needs_runtime = true;
                }
            }
        }

        self.build_imports(referenced_types, needs_runtime)
    }

    fn collect_type_references(
        &self,
        field_type: &IRFieldType,
        package_type_names: &std::collections::HashSet<String>,
        referenced_types: &mut std::collections::HashSet<String>,
    ) {
        match field_type {
            IRFieldType::Primitive(_) => {}
            IRFieldType::Custom(name) => {
                if package_type_names.contains(name) {
                    referenced_types.insert(name.clone());
                }
            }
            IRFieldType::Any => {}
            IRFieldType::List(item) => {
                self.collect_type_references(item, package_type_names, referenced_types);
            }
            IRFieldType::Map(key, value) => {
                self.collect_type_references(key, package_type_names, referenced_types);
                self.collect_type_references(value, package_type_names, referenced_types);
            }
        }
    }

    fn build_imports(
        &self,
        _referenced_types: std::collections::HashSet<String>,
        needs_runtime: bool,
    ) -> Vec<SwiftImport> {
        // In Swift, files in the same module don't need explicit imports
        // We only need to track if FluoriteRuntime is needed
        if needs_runtime {
            vec![SwiftImport {
                name: "FluoriteRuntime".to_string(),
            }]
        } else {
            Vec::new()
        }
    }
}
