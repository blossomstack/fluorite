//! E2E tests for dotted package syntax
//!
//! These tests verify that dotted package names work correctly end-to-end
//! with both Rust and TypeScript code generation.

use std::collections::HashMap;
use std::sync::Arc;

use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use fluorite_codegen::code_gen::ir::IRSchema;
use fluorite_codegen::code_gen::rust::{RustOptions, RustTemplateGenerator};
use fluorite_codegen::code_gen::ts::TsTemplateGenerator;
use fluorite_codegen::code_gen::ts::TypeScriptOptions;
use fluorite_codegen::idl::parse_string_to_ir;

/// Generate Rust code from FL schema and return file contents
fn generate_rust_from_schema(schema: &IRSchema) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_string()).with_single_file(true);
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(schema).unwrap();

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

/// Generate TypeScript code from FL schema and return file contents
fn generate_ts_from_schema(schema: &IRSchema) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_string()).with_single_file(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(schema).unwrap();

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

/// Get the Rust output path for a dotted package name
/// The generator creates nested directories: com/example/users/mod.rs
fn rust_output_path(package: &str) -> String {
    format!("/output/{}/mod.rs", package.replace('.', "/"))
}

/// Get the TypeScript output path for a dotted package name
fn ts_output_path(package: &str) -> String {
    format!("/output/{}/index.ts", package.replace('.', "/"))
}

#[test]
fn test_e2e_dotted_package_rust_codegen() {
    // Parse .fl file with dotted package
    let fl_source = r#"
        package com.example.users;

        struct User {
            id: Uuid,
            name: String,
            email: Option<String>,
        }

        enum UserStatus {
            Active,
            Inactive,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    // Verify output file path uses the package name (dots become directory separators)
    let content = output.get(&rust_output_path("com.example.users")).unwrap();

    // Verify generated code contains expected types
    assert!(content.contains("struct User"));
    assert!(content.contains("id: uuid::Uuid"));
    assert!(content.contains("name: String"));
    assert!(content.contains("email: Option<String>"));
    assert!(content.contains("enum UserStatus"));
    assert!(content.contains("Active"));
    assert!(content.contains("Inactive"));
}

#[test]
fn test_e2e_dotted_package_typescript_codegen() {
    // Parse .fl file with dotted package
    let fl_source = r#"
        package com.example.users;

        struct User {
            id: Uuid,
            name: String,
            email: Option<String>,
        }

        enum UserStatus {
            Active,
            Inactive,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_ts_from_schema(&schema);

    // Verify output file path uses the package name (dots become directory separators)
    let content = output.get(&ts_output_path("com.example.users")).unwrap();

    // Verify generated code contains expected types
    assert!(content.contains("interface User"));
    assert!(content.contains("id: string"));
    assert!(content.contains("name: string"));
    assert!(content.contains("email?: string"));
    assert!(content.contains("enum UserStatus"));
}

#[test]
fn test_e2e_simple_package_still_works() {
    // Verify backwards compatibility with simple package names
    let fl_source = r#"
        package users;

        struct User {
            name: String,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("users")).unwrap();
    assert!(content.contains("struct User"));
}

#[test]
fn test_e2e_two_segment_package() {
    // Test minimal dotted path (two segments)
    let fl_source = r#"
        package myapp.models;

        struct User {
            name: String,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("myapp.models")).unwrap();
    assert!(content.contains("struct User"));
}

#[test]
fn test_e2e_deeply_nested_package() {
    // Test very deep nesting
    let fl_source = r#"
        package a.b.c.d.e.f.models;

        struct Data {
            value: String,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("a.b.c.d.e.f.models")).unwrap();
    assert!(content.contains("struct Data"));
}

#[test]
fn test_e2e_multiple_dotted_packages() {
    // Test multiple files with different dotted packages
    let users_fl = r#"
        package com.example.users;

        struct User {
            name: String,
        }
    "#;

    let orders_fl = r#"
        package com.example.orders;

        struct Order {
            total: f64,
        }
    "#;

    let users_ast = fluorite_codegen::idl::parse_string(users_fl).unwrap();
    let orders_ast = fluorite_codegen::idl::parse_string(orders_fl).unwrap();

    let converter = fluorite_codegen::idl::ast_to_ir::AstToIrConverter::new();
    let schema = converter.convert_files(&[users_ast, orders_ast]).unwrap();

    // Generate code for both packages
    let output = generate_rust_from_schema(&schema);

    // Both packages should have generated files (dots become directory separators)
    assert!(output.contains_key(&rust_output_path("com.example.users")));
    assert!(output.contains_key(&rust_output_path("com.example.orders")));

    let users_content = output.get(&rust_output_path("com.example.users")).unwrap();
    let orders_content = output.get(&rust_output_path("com.example.orders")).unwrap();

    assert!(users_content.contains("struct User"));
    assert!(orders_content.contains("struct Order"));
}

#[test]
fn test_e2e_dotted_package_with_extended_types() {
    // Test dotted package with all extended types
    let fl_source = r#"
        package com.example.models;

        struct ExtendedTypes {
            id: Uuid,
            amount: Decimal,
            data: Bytes,
            url: Url,
            ts: Timestamp,
            ts_millis: TimestampMillis,
            dt_utc: DateTimeUtc,
            dt_tz: DateTimeTz,
            date: Date,
            time: Time,
            duration: Duration,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("com.example.models")).unwrap();

    // Check that extended types are properly mapped
    assert!(content.contains("uuid::Uuid"));
    assert!(content.contains("rust_decimal::Decimal"));
    assert!(content.contains("Vec<u8>"));
    assert!(content.contains("url::Url"));
}

#[test]
fn test_e2e_dotted_package_with_generics() {
    // Test dotted package with generic types
    let fl_source = r#"
        package com.example.models;

        struct Container {
            items: Vec<String>,
            mapping: Map<String, User>,
            maybe: Option<i32>,
        }

        struct User {
            name: String,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("com.example.models")).unwrap();

    // Check that generic types are properly rendered
    assert!(content.contains("Vec<String>"));
    assert!(content.contains("std::collections::HashMap<String"));
    assert!(content.contains("Option<i32>"));
}

#[test]
fn test_e2e_dotted_package_with_union() {
    // Test dotted package with union type (using unit variants to avoid validation issues)
    let fl_source = r#"
        package com.example.events;

        union Event {
            UserCreated,
            OrderPlaced,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get(&rust_output_path("com.example.events")).unwrap();

    // Check that union is properly generated
    assert!(content.contains("enum Event"));
    assert!(content.contains("UserCreated"));
    assert!(content.contains("OrderPlaced"));
}

#[test]
fn test_e2e_dotted_package_with_doc_comments() {
    // Test dotted package with doc comments preserved
    let fl_source = r#"
        package com.example.models;

        /// A user in the system
        struct User {
            /// The user's name
            name: String,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();

    // Check that doc comments are in the IR
    let package = schema.packages.get("com.example.models").unwrap();
    match &package.types[0] {
        fluorite_codegen::code_gen::ir::IRType::Struct(s) => {
            assert_eq!(s.doc.as_ref().unwrap(), "A user in the system");
            assert_eq!(s.fields[0].doc.as_ref().unwrap(), "The user's name");
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_e2e_dotted_package_ts_output_structure() {
    // Test TypeScript output structure with dotted packages
    let fl_source = r#"
        package com.example.users;

        struct User {
            id: Uuid,
            name: String,
        }

        enum Status {
            Active,
            Inactive,
        }
    "#;

    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_ts_from_schema(&schema);

    // Verify the file is in the correct location (dots become directory separators)
    assert!(output.contains_key(&ts_output_path("com.example.users")));

    let content = output.get(&ts_output_path("com.example.users")).unwrap();

    // Verify both interface and enum are generated
    assert!(content.contains("export interface User"));
    assert!(content.contains("export enum Status"));

    // Verify TypeScript type mappings
    assert!(content.contains("id: string")); // UUID maps to string
    assert!(content.contains("name: string"));
}
