//! Builds IR from YAML definitions

use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

use crate::definitions::{CustomType, Definition, Field, UnionStyle};

use super::{
    IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};

/// Builds an IRSchema from definitions
pub struct IRBuilder {
    /// All type names across all definitions (for resolving references)
    all_type_names: HashSet<String>,
    /// Types that are used as inline union variants
    union_variant_names: HashSet<String>,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            all_type_names: HashSet::new(),
            union_variant_names: HashSet::new(),
        }
    }

    /// Build IR schema from definitions
    pub fn build(mut self, definitions: &[Definition]) -> Result<IRSchema> {
        // First pass: collect all type names and identify union variants
        self.collect_type_info(definitions);

        // Second pass: build IR types
        let mut packages: HashMap<String, IRPackage> = HashMap::new();

        for def in definitions {
            let package_name = def
                .configs
                .rust_package
                .as_ref()
                .ok_or_else(|| anyhow!("Missing rust_package in definition"))?
                .clone();

            let package = packages
                .entry(package_name.clone())
                .or_insert_with(|| IRPackage {
                    name: package_name,
                    types: Vec::new(),
                });

            for custom_type in &def.types {
                let ir_type = self.convert_type(custom_type)?;
                package.types.push(ir_type);
            }
        }

        Ok(IRSchema { packages })
    }

    fn collect_type_info(&mut self, definitions: &[Definition]) {
        // First pass: collect all type names
        for def in definitions {
            for t in &def.types {
                let type_name = match t {
                    CustomType::Object { name, .. } => name.clone(),
                    CustomType::Enum { name, .. } => name.clone(),
                    CustomType::Union { name, .. } => name.clone(),
                    CustomType::List { name, .. } => name.clone(),
                    CustomType::Map { name, .. } => name.clone(),
                };
                self.all_type_names.insert(type_name);
            }
        }

        // Second pass: identify inline union variants
        for def in definitions {
            for t in &def.types {
                if let CustomType::Union {
                    values, configs, ..
                } = t
                {
                    let is_inline = configs
                        .as_ref()
                        .and_then(|c| c.union_style.as_ref())
                        .map(|s| *s != UnionStyle::Extern)
                        .unwrap_or(true);

                    if is_inline {
                        for v in values {
                            if self.all_type_names.contains(v) {
                                self.union_variant_names.insert(v.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn convert_type(&self, custom_type: &CustomType) -> Result<IRType> {
        match custom_type {
            CustomType::Object {
                name,
                fields,
                configs,
                description,
            } => {
                let is_union_variant = self.union_variant_names.contains(name);
                let ir_fields = fields.iter().map(|f| self.convert_field(f)).collect();

                // Extract type-level config
                let rename_all = configs.as_ref().and_then(|c| c.rename_all.clone());
                let deny_unknown_fields = configs
                    .as_ref()
                    .and_then(|c| c.rust.as_ref())
                    .and_then(|r| r.deny_unknown_fields)
                    .unwrap_or(false);

                Ok(IRType::Struct(IRStruct {
                    name: name.clone(),
                    fields: ir_fields,
                    is_union_variant,
                    doc: description.clone(),
                    rename_all,
                    deny_unknown_fields,
                }))
            }

            CustomType::Enum {
                name,
                values,
                description,
            } => Ok(IRType::Enum(IREnum {
                name: name.clone(),
                variants: values.clone(),
                doc: description.clone(),
            })),

            CustomType::Union {
                name,
                type_tag,
                values,
                configs,
                description,
            } => {
                let style = configs
                    .as_ref()
                    .and_then(|c| c.union_style.as_ref())
                    .map(|s| match s {
                        UnionStyle::Inline => IRUnionStyle::Inline,
                        UnionStyle::Extern => IRUnionStyle::Extern,
                    })
                    .unwrap_or(IRUnionStyle::Inline);

                let variants = values
                    .iter()
                    .map(|v| {
                        if self.all_type_names.contains(v) {
                            // Reference to a custom type
                            match style {
                                IRUnionStyle::Inline => {
                                    // Will be resolved later during generation
                                    IRUnionVariant::Inline(v.clone(), Vec::new())
                                }
                                IRUnionStyle::Extern => {
                                    IRUnionVariant::Newtype(v.clone(), v.clone())
                                }
                            }
                        } else {
                            // Simple unit variant
                            IRUnionVariant::Unit(v.clone())
                        }
                    })
                    .collect();

                Ok(IRType::Union(IRUnion {
                    name: name.clone(),
                    tag_field: type_tag.clone(),
                    variants,
                    style,
                    doc: description.clone(),
                }))
            }

            CustomType::List {
                name,
                item_type,
                description,
            } => {
                let item = self.convert_field_type(item_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::List(item),
                    doc: description.clone(),
                }))
            }

            CustomType::Map {
                name,
                key_type,
                value_type,
                description,
            } => {
                let key = self.convert_field_type(key_type);
                let value = self.convert_field_type(value_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::Map(key, value),
                    doc: description.clone(),
                }))
            }
        }
    }

    fn convert_field(&self, field: &Field) -> IRField {
        let field_type = self.convert_field_type(&field.field_type);
        let configs = field.configs.as_ref();

        let is_boxed = configs.and_then(|c| c.rust_type_wrapper.as_ref()).is_some();
        let rename = configs.and_then(|c| c.rename.clone());
        let alias = configs.and_then(|c| c.alias.clone()).unwrap_or_default();
        let default = configs.and_then(|c| c.default.clone());

        // Extract Rust-specific config
        let rust_config = configs.and_then(|c| c.rust.as_ref());
        let skip_if_none = rust_config.and_then(|r| r.skip_if_none).unwrap_or(false);
        let skip_if_default = rust_config.and_then(|r| r.skip_if_default).unwrap_or(false);
        let flatten = rust_config.and_then(|r| r.flatten).unwrap_or(false);

        IRField {
            name: field.name.clone(),
            field_type,
            is_optional: field.optional.unwrap_or(false),
            is_boxed,
            rename,
            doc: field.description.clone(),
            alias,
            default,
            skip_if_none,
            skip_if_default,
            flatten,
            deprecated: field.deprecated.unwrap_or(false),
        }
    }

    fn convert_field_type(&self, type_str: &str) -> IRFieldType {
        if type_str == "Any" {
            return IRFieldType::Any;
        }

        if let Some(primitive) = self.parse_primitive(type_str) {
            return IRFieldType::Primitive(primitive);
        }

        IRFieldType::Custom(type_str.to_owned())
    }

    fn parse_primitive(&self, s: &str) -> Option<IRPrimitive> {
        match s {
            // Basic primitives
            "String" => Some(IRPrimitive::String),
            "Bool" => Some(IRPrimitive::Bool),
            "DateTime" => Some(IRPrimitive::DateTime),
            "UInt32" => Some(IRPrimitive::UInt32),
            "UInt64" => Some(IRPrimitive::UInt64),
            "Int32" => Some(IRPrimitive::Int32),
            "Int64" => Some(IRPrimitive::Int64),
            "Float32" => Some(IRPrimitive::Float32),
            "Float64" => Some(IRPrimitive::Float64),
            // Extended primitives
            "UUID" => Some(IRPrimitive::UUID),
            "Decimal" => Some(IRPrimitive::Decimal),
            "Bytes" => Some(IRPrimitive::Bytes),
            "Url" => Some(IRPrimitive::Url),
            "Timestamp" => Some(IRPrimitive::Timestamp),
            "TimestampMillis" => Some(IRPrimitive::TimestampMillis),
            "DateTimeUtc" => Some(IRPrimitive::DateTimeUtc),
            "DateTimeTz" => Some(IRPrimitive::DateTimeTz),
            "Date" => Some(IRPrimitive::Date),
            "Time" => Some(IRPrimitive::Time),
            "Duration" => Some(IRPrimitive::Duration),
            _ => None,
        }
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
