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
            CustomType::Object { name, fields } => {
                let is_union_variant = self.union_variant_names.contains(name);
                let ir_fields = fields.iter().map(|f| self.convert_field(f)).collect();

                Ok(IRType::Struct(IRStruct {
                    name: name.clone(),
                    fields: ir_fields,
                    is_union_variant,
                    doc: None,
                }))
            }

            CustomType::Enum { name, values } => Ok(IRType::Enum(IREnum {
                name: name.clone(),
                variants: values.clone(),
                doc: None,
            })),

            CustomType::Union {
                name,
                type_tag,
                values,
                configs,
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
                    doc: None,
                }))
            }

            CustomType::List { name, item_type } => {
                let item = self.convert_field_type(item_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::List(item),
                    doc: None,
                }))
            }

            CustomType::Map {
                name,
                key_type,
                value_type,
            } => {
                let key = self.convert_field_type(key_type);
                let value = self.convert_field_type(value_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::Map(key, value),
                    doc: None,
                }))
            }
        }
    }

    fn convert_field(&self, field: &Field) -> IRField {
        let field_type = self.convert_field_type(&field.field_type);
        let is_boxed = field
            .configs
            .as_ref()
            .and_then(|c| c.rust_type_wrapper.as_ref())
            .is_some();
        let rename = field.configs.as_ref().and_then(|c| c.rename.clone());

        IRField {
            name: field.name.clone(),
            field_type,
            is_optional: field.optional.unwrap_or(false),
            is_boxed,
            rename,
            doc: None,
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
            "String" => Some(IRPrimitive::String),
            "Bool" => Some(IRPrimitive::Bool),
            "DateTime" => Some(IRPrimitive::DateTime),
            "UInt32" => Some(IRPrimitive::UInt32),
            "UInt64" => Some(IRPrimitive::UInt64),
            "Int32" => Some(IRPrimitive::Int32),
            "Int64" => Some(IRPrimitive::Int64),
            "Float32" => Some(IRPrimitive::Float32),
            "Float64" => Some(IRPrimitive::Float64),
            _ => None,
        }
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}
