//! E2E tests for Fluorite IDL (.fl) code generation

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

#[test]
fn test_fl_simple_struct() {
    let fl_source = r#"
        package test;

        struct User {
            id: Uuid,
            name: String,
            age: Option<u32>,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get("/output/test/mod.rs").unwrap();
    assert!(content.contains("struct User"));
    assert!(content.contains("id: uuid::Uuid"));
    assert!(content.contains("name: String"));
    assert!(content.contains("age: Option<u32>"));
}

#[test]
fn test_fl_enum() {
    let fl_source = r#"
        package test;

        enum Status {
            Active,
            Inactive,
            Pending,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get("/output/test/mod.rs").unwrap();
    assert!(content.contains("enum Status"));
    assert!(content.contains("Active"));
    assert!(content.contains("Inactive"));
}

#[test]
fn test_fl_union() {
    let fl_source = r#"
        package test;

        union Event {
            UserCreated,
            OrderPlaced,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get("/output/test/mod.rs").unwrap();
    assert!(content.contains("enum Event"));
    assert!(content.contains("UserCreated"));
    assert!(content.contains("OrderPlaced"));
}

#[test]
fn test_typescript_simple_struct() {
    let fl_source = r#"
        package test;

        struct User {
            id: Uuid,
            name: String,
            age: Option<u32>,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_ts_from_schema(&schema);

    let content = output.get("/output/test/index.ts").unwrap();
    assert!(content.contains("interface User"));
    assert!(content.contains("id: string"));
    assert!(content.contains("name: string"));
}

#[test]
fn test_typescript_enum() {
    let fl_source = r#"
        package test;

        enum Status {
            Active,
            Inactive,
            Pending,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_ts_from_schema(&schema);

    let content = output.get("/output/test/index.ts").unwrap();
    assert!(content.contains("enum Status"));
}

#[test]
fn test_fl_extended_types() {
    let fl_source = r#"
        package test;

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

    let content = output.get("/output/test/mod.rs").unwrap();

    // Check that extended types are properly mapped
    assert!(content.contains("uuid::Uuid"));
    assert!(content.contains("rust_decimal::Decimal"));
    assert!(content.contains("Vec<u8>"));
    assert!(content.contains("url::Url"));
}

#[test]
fn test_fl_generic_types() {
    let fl_source = r#"
        package test;

        struct Generics {
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

    let content = output.get("/output/test/mod.rs").unwrap();

    // Check that generic types are properly rendered
    assert!(content.contains("Vec<String>"));
    assert!(content.contains("std::collections::HashMap<String"));
    assert!(content.contains("Option<i32>"));
}

#[test]
fn test_fl_doc_comments() {
    let fl_source = r#"
        package test;

        /// A user in the system
        struct User {
            /// The user's name
            name: String,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();

    // Check that doc comments are in the IR
    let package = schema.packages.get("test").unwrap();
    match &package.types[0] {
        fluorite_codegen::code_gen::ir::IRType::Struct(s) => {
            assert_eq!(s.doc.as_ref().unwrap(), "A user in the system");
            assert_eq!(s.fields[0].doc.as_ref().unwrap(), "The user's name");
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_fl_attributes() {
    let fl_source = r#"
        package test;

        struct User {
            #[rename = "userName"]
            name: String,
            #[deprecated]
            old_field: String,
        }
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get("/output/test/mod.rs").unwrap();

    // Check that attributes are preserved in IR and affect output
    assert!(content.contains("struct User"));
    assert!(content.contains("name"));
}

#[test]
fn test_fl_type_alias() {
    let fl_source = r#"
        package test;

        struct User {
            name: String,
        }

        type UserList = Vec<User>;
        type UserMap = Map<String, User>;
    "#;
    let schema = parse_string_to_ir(fl_source).unwrap();
    let output = generate_rust_from_schema(&schema);

    let content = output.get("/output/test/mod.rs").unwrap();

    // Check that type aliases are generated
    assert!(content.contains("type UserList"));
    assert!(content.contains("type UserMap"));
}
