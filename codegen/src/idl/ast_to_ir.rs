//! Converts AST types to IR types for code generation

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::code_gen::ir::{
    IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionVariant,
};

use super::ast::{
    AstAttribute, AstEnum, AstField, AstFile, AstItem, AstStruct, AstType, AstTypeAlias, AstUnion,
    AstUnionVariant,
};

/// Converts AST files to IR schema
pub struct AstToIrConverter {
    /// All type names across all files (for resolving references)
    all_type_names: std::collections::HashSet<String>,
}

impl AstToIrConverter {
    pub fn new() -> Self {
        Self {
            all_type_names: std::collections::HashSet::new(),
        }
    }

    /// Convert multiple AST files to a single IR schema
    pub fn convert_files(mut self, files: &[AstFile]) -> Result<IRSchema> {
        // First pass: collect all type names
        self.collect_type_names(files);

        // Second pass: build IR types
        let mut packages: HashMap<String, IRPackage> = HashMap::new();

        for file in files {
            let package_name = file
                .package
                .iter()
                .map(|s| s.value.as_str())
                .collect::<Vec<_>>()
                .join(".");

            let package = packages
                .entry(package_name.clone())
                .or_insert_with(|| IRPackage {
                    name: package_name,
                    types: Vec::new(),
                });

            for item in &file.items {
                let ir_type = self.convert_item(item)?;
                package.types.push(ir_type);
            }
        }

        Ok(IRSchema { packages })
    }

    fn collect_type_names(&mut self, files: &[AstFile]) {
        for file in files {
            for item in &file.items {
                let type_name = match item {
                    AstItem::Struct(s) => s.name.value.clone(),
                    AstItem::Enum(e) => e.name.value.clone(),
                    AstItem::Union(u) => u.name.value.clone(),
                    AstItem::TypeAlias(t) => t.name.value.clone(),
                };
                self.all_type_names.insert(type_name);
            }
        }
    }

    fn convert_item(&self, item: &AstItem) -> Result<IRType> {
        match item {
            AstItem::Struct(s) => self.convert_struct(s),
            AstItem::Enum(e) => self.convert_enum(e),
            AstItem::Union(u) => self.convert_union(u),
            AstItem::TypeAlias(t) => self.convert_type_alias(t),
        }
    }

    fn convert_struct(&self, ast_struct: &AstStruct) -> Result<IRType> {
        let fields = ast_struct
            .fields
            .iter()
            .map(|f| self.convert_field(f))
            .collect();

        // Extract attributes
        let deny_unknown_fields = self.has_attr(&ast_struct.attrs, "deny_unknown_fields");

        Ok(IRType::Struct(IRStruct {
            name: ast_struct.name.value.clone(),
            fields,
            doc: ast_struct.doc.clone(),
            deny_unknown_fields,
        }))
    }

    fn convert_enum(&self, ast_enum: &AstEnum) -> Result<IRType> {
        let variants = ast_enum
            .variants
            .iter()
            .map(|v| v.name.value.clone())
            .collect();

        Ok(IRType::Enum(IREnum {
            name: ast_enum.name.value.clone(),
            variants,
            doc: ast_enum.doc.clone(),
        }))
    }

    fn convert_union(&self, ast_union: &AstUnion) -> Result<IRType> {
        // Get tag field name from attributes or default to "type"
        let tag_field = self
            .get_attr_value(&ast_union.attrs, "type_tag")
            .unwrap_or_else(|| "type".to_string());

        // Get content field name from attributes or default to "value"
        let content_field = self
            .get_attr_value(&ast_union.attrs, "content_tag")
            .unwrap_or_else(|| "value".to_string());

        let variants: Result<Vec<_>> = ast_union
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v))
            .collect();

        Ok(IRType::Union(IRUnion {
            name: ast_union.name.value.clone(),
            tag_field,
            content_field,
            variants: variants?,
            doc: ast_union.doc.clone(),
        }))
    }

    fn convert_union_variant(&self, variant: &AstUnionVariant) -> Result<IRUnionVariant> {
        match &variant.inner_type {
            Some(inner_type) => {
                // Convert the inner type to IRFieldType
                let ast_type = AstType::Named(inner_type.clone());
                let field_type = self.convert_ast_type(&ast_type);
                Ok(IRUnionVariant::Newtype(
                    variant.name.value.clone(),
                    field_type,
                ))
            }
            None => Ok(IRUnionVariant::Unit(variant.name.value.clone())),
        }
    }

    fn convert_type_alias(&self, type_alias: &AstTypeAlias) -> Result<IRType> {
        let target = match &type_alias.target {
            AstType::Vec(inner) => {
                let item_type = self.convert_ast_type(inner);
                IRTypeAliasTarget::List(item_type)
            }
            AstType::Map(key, value) => {
                let key_type = self.convert_ast_type(key);
                let value_type = self.convert_ast_type(value);
                IRTypeAliasTarget::Map(key_type, value_type)
            }
            AstType::Named(_) | AstType::Option(_) => {
                return Err(anyhow!("Type alias must be Vec<T> or Map<K, V>"))
            }
        };

        Ok(IRType::TypeAlias(IRTypeAlias {
            name: type_alias.name.value.clone(),
            target,
            doc: type_alias.doc.clone(),
        }))
    }

    fn convert_field(&self, field: &AstField) -> IRField {
        let field_type = self.convert_ast_type(&field.ty);

        // Extract attributes
        let is_boxed = self.has_attr(&field.attrs, "box");
        let rename = self.get_attr_value(&field.attrs, "rename");
        let alias = self.get_attr_values(&field.attrs, "alias");
        let default = self.get_attr_value(&field.attrs, "default");
        let skip_if_none = self.has_attr(&field.attrs, "skip_if_none");
        let skip_if_default = self.has_attr(&field.attrs, "skip_if_default");
        let flatten = self.has_attr(&field.attrs, "flatten");
        let deprecated = self.has_attr(&field.attrs, "deprecated");

        // Determine if optional
        let (is_optional, final_type) = match &field.ty {
            AstType::Option(inner) => (true, self.convert_ast_type(inner)),
            AstType::Named(_) | AstType::Vec(_) | AstType::Map(..) => (false, field_type),
        };

        IRField {
            name: field.name.value.clone(),
            field_type: final_type,
            is_optional,
            is_boxed,
            rename,
            doc: field.doc.clone(),
            alias,
            default,
            skip_if_none,
            skip_if_default,
            flatten,
            deprecated,
        }
    }

    fn convert_ast_type(&self, ast_type: &AstType) -> IRFieldType {
        match ast_type {
            AstType::Named(name) => {
                let type_name = &name.value;
                if type_name == "Any" {
                    IRFieldType::Any
                } else if let Some(primitive) = self.parse_primitive(type_name) {
                    IRFieldType::Primitive(primitive)
                } else {
                    IRFieldType::Custom(type_name.clone())
                }
            }
            AstType::Option(inner) => self.convert_ast_type(inner),
            AstType::Vec(inner) => {
                let inner_type = self.convert_ast_type(inner);
                IRFieldType::List(Box::new(inner_type))
            }
            AstType::Map(key, value) => {
                let key_type = self.convert_ast_type(key);
                let value_type = self.convert_ast_type(value);
                IRFieldType::Map(Box::new(key_type), Box::new(value_type))
            }
        }
    }

    fn parse_primitive(&self, s: &str) -> Option<IRPrimitive> {
        match s {
            "String" => Some(IRPrimitive::String),
            "bool" => Some(IRPrimitive::Bool),
            "i32" => Some(IRPrimitive::Int32),
            "i64" => Some(IRPrimitive::Int64),
            "u32" => Some(IRPrimitive::UInt32),
            "u64" => Some(IRPrimitive::UInt64),
            "f32" => Some(IRPrimitive::Float32),
            "f64" => Some(IRPrimitive::Float64),
            "DateTime" => Some(IRPrimitive::DateTime),
            "Uuid" => Some(IRPrimitive::UUID),
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

    fn has_attr(&self, attrs: &[AstAttribute], name: &str) -> bool {
        attrs.iter().any(|a| a.name.value == name)
    }

    fn get_attr_value(&self, attrs: &[AstAttribute], name: &str) -> Option<String> {
        attrs
            .iter()
            .find(|a| a.name.value == name)
            .and_then(|a| a.value.as_ref().map(|v| v.value.clone()))
    }

    fn get_attr_values(&self, attrs: &[AstAttribute], name: &str) -> Vec<String> {
        attrs
            .iter()
            .filter(|a| a.name.value == name)
            .filter_map(|a| a.value.as_ref().map(|v| v.value.clone()))
            .collect()
    }
}

impl Default for AstToIrConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::parser::parse_file;

    #[test]
    fn test_convert_simple_struct() {
        let source = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        assert!(schema.packages.contains_key("test"));
        let package = schema.packages.get("test").unwrap();
        assert_eq!(package.types.len(), 1);

        match &package.types[0] {
            IRType::Struct(s) => {
                assert_eq!(s.name, "User");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "name");
                assert_eq!(s.fields[1].name, "age");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_convert_enum() {
        let source = r#"
            package test;
            enum Status {
                Active,
                Inactive,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        let package = schema.packages.get("test").unwrap();
        match &package.types[0] {
            IRType::Enum(e) => {
                assert_eq!(e.name, "Status");
                assert_eq!(e.variants, vec!["Active", "Inactive"]);
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_convert_union() {
        let source = r#"
            package test;
            struct User {}
            struct Order {}
            union Event {
                UserCreated(User),
                OrderPlaced(Order),
                Deleted,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        let package = schema.packages.get("test").unwrap();
        match &package.types[2] {
            IRType::Union(u) => {
                assert_eq!(u.name, "Event");
                assert_eq!(u.tag_field, "type");
                assert_eq!(u.content_field, "value");
                assert_eq!(u.variants.len(), 3);

                // Check variant types
                match &u.variants[0] {
                    IRUnionVariant::Newtype(name, _) => assert_eq!(name, "UserCreated"),
                    _ => panic!("Expected Newtype variant"),
                }
                match &u.variants[2] {
                    IRUnionVariant::Unit(name) => assert_eq!(name, "Deleted"),
                    _ => panic!("Expected Unit variant"),
                }
            }
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_convert_union_with_primitives() {
        let source = r#"
            package test;
            union Message {
                Text(String),
                Count(i32),
                Empty,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        let package = schema.packages.get("test").unwrap();
        match &package.types[0] {
            IRType::Union(u) => {
                assert_eq!(u.name, "Message");
                assert_eq!(u.variants.len(), 3);

                match &u.variants[0] {
                    IRUnionVariant::Newtype(name, field_type) => {
                        assert_eq!(name, "Text");
                        assert!(matches!(
                            field_type,
                            IRFieldType::Primitive(IRPrimitive::String)
                        ));
                    }
                    _ => panic!("Expected Newtype variant"),
                }
                match &u.variants[1] {
                    IRUnionVariant::Newtype(name, field_type) => {
                        assert_eq!(name, "Count");
                        assert!(matches!(
                            field_type,
                            IRFieldType::Primitive(IRPrimitive::Int32)
                        ));
                    }
                    _ => panic!("Expected Newtype variant"),
                }
            }
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_convert_optional_field() {
        let source = r#"
            package test;
            struct User {
                name: Option<String>,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        let package = schema.packages.get("test").unwrap();
        match &package.types[0] {
            IRType::Struct(s) => {
                assert!(s.fields[0].is_optional);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_convert_simple_package() {
        let source = r#"
            package users;
            struct User {
                name: String,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        assert!(schema.packages.contains_key("users"));
        assert_eq!(schema.packages.len(), 1);
    }

    #[test]
    fn test_convert_dotted_package() {
        let source = r#"
            package com.example.users;
            struct User {
                name: String,
            }
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        assert!(schema.packages.contains_key("com.example.users"));
        assert_eq!(schema.packages.len(), 1);

        let package = schema.packages.get("com.example.users").unwrap();
        assert_eq!(package.name, "com.example.users");
        assert_eq!(package.types.len(), 1);
    }

    #[test]
    fn test_convert_deep_dotted_package() {
        let source = r#"
            package a.b.c.d.e.f;
            struct Data {}
        "#;
        let ast = parse_file(source).unwrap();
        let converter = AstToIrConverter::new();
        let schema = converter.convert_files(&[ast]).unwrap();

        assert!(schema.packages.contains_key("a.b.c.d.e.f"));
        assert_eq!(schema.packages.len(), 1);
    }
}
