//! Converts AST types to IR types for code generation

use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

use crate::code_gen::ir::{
    IREnum, IREnumVariant, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct,
    IRType, IRTypeAlias, IRTypeAliasTarget, IRTypeRef, IRUnion, IRUnionVariant,
};

use super::ast::{
    AstAttribute, AstEnum, AstField, AstFile, AstItem, AstStruct, AstType, AstTypeAlias, AstUnion,
    AstUnionVariant,
};

/// Where a file's bare type names may resolve to.
///
/// Built per file, because `use` imports are per file. Resolution order is
/// deliberately narrow: a name is either declared in the file's own package or
/// explicitly imported. Anything else is an error — see [`Scope::resolve`].
struct Scope<'a> {
    /// Package of the file being lowered.
    package: &'a str,
    /// Type names declared in that package.
    declared: &'a BTreeSet<String>,
    /// Bare name → package, from this file's `use` statements.
    imports: BTreeMap<String, String>,
    /// Every package that declares a given bare name, for error messages.
    owners: &'a BTreeMap<String, BTreeSet<String>>,
}

impl Scope<'_> {
    /// Resolve a bare type name to the package that declares it.
    fn resolve(&self, name: &str) -> Result<IRTypeRef> {
        let local = self.declared.contains(name);
        let imported = self.imports.get(name);

        match (local, imported) {
            // Declaring a name and importing the same name leaves the reference
            // genuinely ambiguous. Rust rejects this too (E0255).
            (true, Some(from)) => Err(anyhow!(
                "'{name}' is declared in package '{}' and also imported from '{from}'. \
                 Remove the import or rename one of the types.",
                self.package
            )),
            (true, None) => Ok(IRTypeRef::new(self.package, name)),
            (false, Some(from)) => Ok(IRTypeRef::new(from, name)),
            (false, None) => Err(self.unresolved(name)),
        }
    }

    fn unresolved(&self, name: &str) -> anyhow::Error {
        match self.owners.get(name) {
            Some(packages) => {
                let candidates = packages
                    .iter()
                    .map(|p| format!("use {p}.{name};"))
                    .collect::<Vec<_>>()
                    .join("\n  ");
                anyhow!(
                    "'{name}' is not declared in package '{}' and is not imported. \
                     It is declared in {}. Add one of:\n  {candidates}",
                    self.package,
                    packages
                        .iter()
                        .map(|p| format!("'{p}'"))
                        .collect::<Vec<_>>()
                        .join(" and "),
                )
            }
            None => anyhow!(
                "Unknown type '{name}' referenced from package '{}'",
                self.package
            ),
        }
    }
}

/// Converts AST files to IR schema
pub struct AstToIrConverter;

impl AstToIrConverter {
    pub fn new() -> Self {
        Self
    }

    /// Convert multiple AST files to a single IR schema
    pub fn convert_files(self, files: &[AstFile]) -> Result<IRSchema> {
        // First pass: which package declares which type names. Both maps are
        // needed: `declared` answers "is this name local", `owners` answers
        // "where else could this name have come from" for error messages.
        let mut declared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for file in files {
            let package_name = Self::package_name(file);
            for item in &file.items {
                let type_name = item.name().to_string();
                owners
                    .entry(type_name.clone())
                    .or_default()
                    .insert(package_name.clone());
                declared
                    .entry(package_name.clone())
                    .or_default()
                    .insert(type_name);
            }
        }

        // Second pass: build IR types, resolving references against each file's
        // own scope.
        let mut packages: BTreeMap<String, IRPackage> = BTreeMap::new();
        let empty = BTreeSet::new();

        for file in files {
            let package_name = Self::package_name(file);
            let scope = Scope {
                package: &package_name,
                declared: declared.get(&package_name).unwrap_or(&empty),
                imports: Self::imports(file, &owners)?,
                owners: &owners,
            };

            let mut converted = Vec::with_capacity(file.items.len());
            for item in &file.items {
                converted.push(Self::convert_item(item, &scope)?);
            }

            packages
                .entry(package_name.clone())
                .or_insert_with(|| IRPackage {
                    name: package_name,
                    types: Vec::new(),
                })
                .types
                .extend(converted);
        }

        Ok(IRSchema { packages })
    }

    fn package_name(file: &AstFile) -> String {
        file.package
            .iter()
            .map(|s| s.value.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Build a file's import table from its `use` statements.
    ///
    /// `use a.b.Type;` is split at the last segment: `a.b` is the package and
    /// `Type` the name. An import that names a type the package does not
    /// declare is rejected here rather than surfacing later as a confusing
    /// unresolved reference.
    fn imports(
        file: &AstFile,
        owners: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<BTreeMap<String, String>> {
        let mut imports: BTreeMap<String, String> = BTreeMap::new();

        for use_stmt in &file.uses {
            let segments: Vec<&str> = use_stmt.path.iter().map(|s| s.value.as_str()).collect();
            let Some((name, package_segments)) = segments.split_last() else {
                continue;
            };
            if package_segments.is_empty() {
                return Err(anyhow!(
                    "Import '{}' has no package. Write 'use <package>.{name};'.",
                    segments.join(".")
                ));
            }
            let package = package_segments.join(".");

            match owners.get(*name) {
                Some(packages) if packages.contains(&package) => {}
                Some(packages) => {
                    return Err(anyhow!(
                        "Import 'use {package}.{name};' does not match any declaration. \
                         '{name}' is declared in {}.",
                        packages
                            .iter()
                            .map(|p| format!("'{p}'"))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ))
                }
                None => {
                    return Err(anyhow!(
                        "Import 'use {package}.{name};' refers to an unknown type '{name}'."
                    ))
                }
            }

            if let Some(existing) = imports.insert(name.to_string(), package.clone()) {
                if existing != package {
                    return Err(anyhow!(
                        "'{name}' is imported from both '{existing}' and '{package}'. \
                         A file can import only one type per name."
                    ));
                }
            }
        }

        Ok(imports)
    }

    fn convert_item(item: &AstItem, scope: &Scope) -> Result<IRType> {
        match item {
            AstItem::Struct(s) => Self::convert_struct(s, scope),
            AstItem::Enum(e) => Self::convert_enum(e),
            AstItem::Union(u) => Self::convert_union(u, scope),
            AstItem::TypeAlias(t) => Self::convert_type_alias(t, scope),
        }
    }

    fn convert_struct(ast_struct: &AstStruct, scope: &Scope) -> Result<IRType> {
        let fields = ast_struct
            .fields
            .iter()
            .map(|f| Self::convert_field(f, scope))
            .collect::<Result<Vec<_>>>()?;

        // Extract attributes
        let deny_unknown_fields = Self::has_attr(&ast_struct.attrs, "deny_unknown_fields");

        Ok(IRType::Struct(IRStruct {
            name: ast_struct.name.value.clone(),
            fields,
            doc: ast_struct.doc.clone(),
            deny_unknown_fields,
        }))
    }

    fn convert_enum(ast_enum: &AstEnum) -> Result<IRType> {
        let variants = ast_enum
            .variants
            .iter()
            .map(|v| IREnumVariant {
                name: v.name.value.clone(),
                doc: v.doc.clone(),
            })
            .collect();

        Ok(IRType::Enum(IREnum {
            name: ast_enum.name.value.clone(),
            variants,
            doc: ast_enum.doc.clone(),
        }))
    }

    fn convert_union(ast_union: &AstUnion, scope: &Scope) -> Result<IRType> {
        // Get tag field name from attributes or default to "type"
        let tag_field = Self::get_attr_value(&ast_union.attrs, "type_tag")
            .unwrap_or_else(|| "type".to_string());

        // Get content field name from attributes or default to "value"
        let content_field = Self::get_attr_value(&ast_union.attrs, "content_tag")
            .unwrap_or_else(|| "value".to_string());

        let variants: Result<Vec<_>> = ast_union
            .variants
            .iter()
            .map(|v| Self::convert_union_variant(v, scope))
            .collect();

        Ok(IRType::Union(IRUnion {
            name: ast_union.name.value.clone(),
            tag_field,
            content_field,
            variants: variants?,
            doc: ast_union.doc.clone(),
        }))
    }

    fn convert_union_variant(variant: &AstUnionVariant, scope: &Scope) -> Result<IRUnionVariant> {
        match &variant.inner_type {
            Some(inner_type) => {
                // Convert the inner type to IRFieldType
                let ast_type = AstType::Named(inner_type.clone());
                let field_type = Self::convert_ast_type(&ast_type, scope)?;
                Ok(IRUnionVariant::Newtype {
                    name: variant.name.value.clone(),
                    ty: field_type,
                    doc: variant.doc.clone(),
                })
            }
            None => Ok(IRUnionVariant::Unit {
                name: variant.name.value.clone(),
                doc: variant.doc.clone(),
            }),
        }
    }

    fn convert_type_alias(type_alias: &AstTypeAlias, scope: &Scope) -> Result<IRType> {
        let target = match &type_alias.target {
            AstType::Vec(inner) => {
                let item_type = Self::convert_ast_type(inner, scope)?;
                IRTypeAliasTarget::List(item_type)
            }
            AstType::Map(key, value) => {
                let key_type = Self::convert_ast_type(key, scope)?;
                let value_type = Self::convert_ast_type(value, scope)?;
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

    fn convert_field(field: &AstField, scope: &Scope) -> Result<IRField> {
        let field_type = Self::convert_ast_type(&field.ty, scope)?;

        // Extract attributes
        let is_boxed = Self::has_attr(&field.attrs, "box");
        let rename = Self::get_attr_value(&field.attrs, "rename");
        let alias = Self::get_attr_values(&field.attrs, "alias");
        let default = Self::get_attr_value(&field.attrs, "default");
        let skip_if_none = Self::has_attr(&field.attrs, "skip_if_none");
        let skip_if_default = Self::has_attr(&field.attrs, "skip_if_default");
        let flatten = Self::has_attr(&field.attrs, "flatten");
        let deprecated = Self::has_attr(&field.attrs, "deprecated");

        // Determine if optional
        let (is_optional, final_type) = match &field.ty {
            AstType::Option(inner) => (true, Self::convert_ast_type(inner, scope)?),
            AstType::Named(_) | AstType::Vec(_) | AstType::Map(..) => (false, field_type),
        };

        Ok(IRField {
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
        })
    }

    fn convert_ast_type(ast_type: &AstType, scope: &Scope) -> Result<IRFieldType> {
        match ast_type {
            AstType::Named(name) => {
                let type_name = &name.value;
                if type_name == "Any" {
                    Ok(IRFieldType::Any)
                } else if let Some(primitive) = Self::parse_primitive(type_name) {
                    Ok(IRFieldType::Primitive(primitive))
                } else {
                    Ok(IRFieldType::Custom(scope.resolve(type_name)?))
                }
            }
            AstType::Option(inner) => Self::convert_ast_type(inner, scope),
            AstType::Vec(inner) => {
                let inner_type = Self::convert_ast_type(inner, scope)?;
                Ok(IRFieldType::List(Box::new(inner_type)))
            }
            AstType::Map(key, value) => {
                let key_type = Self::convert_ast_type(key, scope)?;
                let value_type = Self::convert_ast_type(value, scope)?;
                Ok(IRFieldType::Map(Box::new(key_type), Box::new(value_type)))
            }
        }
    }

    fn parse_primitive(s: &str) -> Option<IRPrimitive> {
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

    fn has_attr(attrs: &[AstAttribute], name: &str) -> bool {
        attrs.iter().any(|a| a.name.value == name)
    }

    fn get_attr_value(attrs: &[AstAttribute], name: &str) -> Option<String> {
        attrs
            .iter()
            .find(|a| a.name.value == name)
            .and_then(|a| a.value.as_ref().map(|v| v.value.clone()))
    }

    fn get_attr_values(attrs: &[AstAttribute], name: &str) -> Vec<String> {
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
                assert_eq!(
                    e.variants
                        .iter()
                        .map(|v| v.name.as_str())
                        .collect::<Vec<_>>(),
                    vec!["Active", "Inactive"]
                );
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
                    IRUnionVariant::Newtype { name, .. } => assert_eq!(name, "UserCreated"),
                    _ => panic!("Expected Newtype variant"),
                }
                match &u.variants[2] {
                    IRUnionVariant::Unit { name, .. } => assert_eq!(name, "Deleted"),
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
                    IRUnionVariant::Newtype {
                        name,
                        ty: field_type,
                        ..
                    } => {
                        assert_eq!(name, "Text");
                        assert!(matches!(
                            field_type,
                            IRFieldType::Primitive(IRPrimitive::String)
                        ));
                    }
                    _ => panic!("Expected Newtype variant"),
                }
                match &u.variants[1] {
                    IRUnionVariant::Newtype {
                        name,
                        ty: field_type,
                        ..
                    } => {
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
