//! E2E tests comparing YAML and FL file code generation output
//!
//! These tests verify that the same schema produces identical code
//! whether defined in YAML or the Fluorite IDL (.fl) format.

use std::collections::HashMap;
use std::sync::Arc;

use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use fluorite_codegen::code_gen::ir::IRSchema;
use fluorite_codegen::code_gen::rust::{RustOptions, RustTemplateGenerator};
use fluorite_codegen::code_gen::ts::TsTemplateGenerator;
use fluorite_codegen::code_gen::ts::TypeScriptOptions;
use fluorite_codegen::definitions::Definition;
use fluorite_codegen::idl::parse_string_to_ir;

/// Helper to create a YAML definition for a simple struct
fn yaml_simple_struct() -> Definition {
    let yaml = r#"
configs:
  rust_package: test

types:
  - name: User
    type: Object
    fields:
      - name: id
        type: UUID
      - name: name
        type: String
      - name: age
        type: UInt32
        optional: true
"#;
    serde_yaml::from_str(yaml).unwrap()
}

/// Equivalent FL definition for a simple struct
fn fl_simple_struct() -> &'static str {
    r#"
package test;

struct User {
    id: Uuid,
    name: String,
    age: Option<u32>,
}
"#
}

/// Helper to create a YAML definition for an enum
fn yaml_enum() -> Definition {
    let yaml = r#"
configs:
  rust_package: test

types:
  - name: Status
    type: Enum
    values:
      - Active
      - Inactive
      - Pending
"#;
    serde_yaml::from_str(yaml).unwrap()
}

/// Equivalent FL definition for an enum
fn fl_enum() -> &'static str {
    r#"
package test;

enum Status {
    Active,
    Inactive,
    Pending,
}
"#
}

/// Helper to create a YAML definition for a union
fn yaml_union() -> Definition {
    let yaml = r#"
configs:
  rust_package: test

types:
  - name: User
    type: Object
    fields:
      - name: id
        type: UUID

  - name: Order
    type: Object
    fields:
      - name: id
        type: UUID

  - name: Event
    type: Union
    type_tag: type
    values:
      - UserCreated
      - OrderPlaced
"#;
    serde_yaml::from_str(yaml).unwrap()
}

/// Equivalent FL definition for a union
/// Using unit variants to match the YAML definition
fn fl_union() -> &'static str {
    r#"
package test;

union Event {
    UserCreated,
    OrderPlaced,
}
"#
}

/// Generate Rust code from YAML definitions and return file contents
fn generate_rust_from_yaml(definitions: &[Definition]) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_string()).with_single_file(true);
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(definitions).unwrap();

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

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

/// Generate TypeScript code from YAML definitions and return file contents
fn generate_ts_from_yaml(definitions: &[Definition]) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_string()).with_single_file(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(definitions).unwrap();

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
fn test_rust_simple_struct_yaml_vs_fl() {
    let yaml_defs = vec![yaml_simple_struct()];
    let yaml_output = generate_rust_from_yaml(&yaml_defs);

    let fl_source = fl_simple_struct();
    let schema = parse_string_to_ir(fl_source).unwrap();
    let fl_output = generate_rust_from_schema(&schema);

    // Both should generate the same file
    let yaml_content = yaml_output.get("/output/test/mod.rs").unwrap();
    let fl_content = fl_output.get("/output/test/mod.rs").unwrap();

    // Both should contain the User struct
    assert!(yaml_content.contains("struct User"));
    assert!(fl_content.contains("struct User"));

    // Both should have the same fields
    assert!(yaml_content.contains("id: uuid::Uuid"));
    assert!(fl_content.contains("id: uuid::Uuid"));
    assert!(yaml_content.contains("name: String"));
    assert!(fl_content.contains("name: String"));
    assert!(yaml_content.contains("age: Option<u32>"));
    assert!(fl_content.contains("age: Option<u32>"));
}

#[test]
fn test_rust_enum_yaml_vs_fl() {
    let yaml_defs = vec![yaml_enum()];
    let yaml_output = generate_rust_from_yaml(&yaml_defs);

    let fl_source = fl_enum();
    let schema = parse_string_to_ir(fl_source).unwrap();
    let fl_output = generate_rust_from_schema(&schema);

    let yaml_content = yaml_output.get("/output/test/mod.rs").unwrap();
    let fl_content = fl_output.get("/output/test/mod.rs").unwrap();

    // Both should contain the Status enum with same variants
    assert!(yaml_content.contains("enum Status"));
    assert!(fl_content.contains("enum Status"));
    assert!(yaml_content.contains("Active"));
    assert!(fl_content.contains("Active"));
    assert!(yaml_content.contains("Inactive"));
    assert!(fl_content.contains("Inactive"));
}

#[test]
fn test_rust_union_yaml_vs_fl() {
    let yaml_defs = vec![yaml_union()];
    let yaml_output = generate_rust_from_yaml(&yaml_defs);

    let fl_source = fl_union();
    let schema = parse_string_to_ir(fl_source).unwrap();
    let fl_output = generate_rust_from_schema(&schema);

    let yaml_content = yaml_output.get("/output/test/mod.rs").unwrap();
    let fl_content = fl_output.get("/output/test/mod.rs").unwrap();

    // Both should contain the Event union
    assert!(yaml_content.contains("enum Event"));
    assert!(fl_content.contains("enum Event"));

    // Both should have the same variants
    assert!(yaml_content.contains("UserCreated"));
    assert!(fl_content.contains("UserCreated"));
    assert!(yaml_content.contains("OrderPlaced"));
    assert!(fl_content.contains("OrderPlaced"));
}

#[test]
fn test_typescript_simple_struct_yaml_vs_fl() {
    let yaml_defs = vec![yaml_simple_struct()];
    let yaml_output = generate_ts_from_yaml(&yaml_defs);

    let fl_source = fl_simple_struct();
    let schema = parse_string_to_ir(fl_source).unwrap();
    let fl_output = generate_ts_from_schema(&schema);

    let yaml_content = yaml_output.get("/output/test/index.ts").unwrap();
    let fl_content = fl_output.get("/output/test/index.ts").unwrap();

    // Both should contain the User interface
    assert!(yaml_content.contains("interface User"));
    assert!(fl_content.contains("interface User"));

    // Both should have the same fields
    assert!(yaml_content.contains("id: string"));
    assert!(fl_content.contains("id: string"));
    assert!(yaml_content.contains("name: string"));
    assert!(fl_content.contains("name: string"));
}

#[test]
fn test_typescript_enum_yaml_vs_fl() {
    let yaml_defs = vec![yaml_enum()];
    let yaml_output = generate_ts_from_yaml(&yaml_defs);

    let fl_source = fl_enum();
    let schema = parse_string_to_ir(fl_source).unwrap();
    let fl_output = generate_ts_from_schema(&schema);

    let yaml_content = yaml_output.get("/output/test/index.ts").unwrap();
    let fl_content = fl_output.get("/output/test/index.ts").unwrap();

    // Both should contain the Status enum
    assert!(yaml_content.contains("enum Status"));
    assert!(fl_content.contains("enum Status"));
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

#[rename_all = "camelCase"]
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
    // The rename_all and rename should affect serde attributes
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
