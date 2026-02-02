//! Integration tests for dotted package and import paths
//!
//! These tests verify that the IDL parser correctly handles dotted package names
//! like `com.example.users` and dotted imports like `use com.example.users.User;`

use std::path::PathBuf;

use fluorite_codegen::idl::{parse_file, parse_files, parse_string, parse_string_to_ir};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("demo")
        .join("fluorite")
}

#[test]
fn test_parse_demo_users_fl() {
    // Parse examples/demo/fluorite/users.fl with dotted package
    let path = fixtures_dir().join("users.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse users.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    // Verify: package = "demo.users"
    assert_eq!(ast.package.len(), 2);
    assert_eq!(ast.package[0].value, "demo");
    assert_eq!(ast.package[1].value, "users");

    // Verify: has User, UserStatus, Gender, UserStatusChange, UserEvent
    assert!(ast.items.len() >= 3);
}

#[test]
fn test_parse_demo_orders_fl() {
    // Parse examples/demo/fluorite/orders.fl with dotted imports
    let path = fixtures_dir().join("orders.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse orders.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    // Verify: package = "demo.orders"
    assert_eq!(ast.package.len(), 2);
    assert_eq!(ast.package[0].value, "demo");
    assert_eq!(ast.package[1].value, "orders");

    // Verify: imports from demo.common and demo.users
    assert!(ast.uses.len() >= 2);
}

#[test]
fn test_multi_file_cross_package_imports() {
    // Parse all demo .fl files together
    let paths = vec![
        fixtures_dir().join("common.fl"),
        fixtures_dir().join("users.fl"),
        fixtures_dir().join("orders.fl"),
        fixtures_dir().join("notifications.fl"),
    ];
    let result = parse_files(&paths);
    assert!(result.is_ok(), "Failed to parse files: {:?}", result.err());

    let asts = result.unwrap();
    assert_eq!(asts.len(), 4);

    // Verify IR schema has all packages
    let converter = fluorite_codegen::idl::ast_to_ir::AstToIrConverter::new();
    let schema = converter.convert_files(&asts).unwrap();

    assert!(schema.packages.contains_key("demo.common"));
    assert!(schema.packages.contains_key("demo.users"));
    assert!(schema.packages.contains_key("demo.orders"));
    assert!(schema.packages.contains_key("demo.notifications"));
    assert_eq!(schema.packages.len(), 4);
}

#[test]
fn test_deeply_nested_package() {
    // Test edge case: very deep nesting
    let source = r#"
        package a.b.c.d.e.f.models;
        struct Data {
            value: String,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    assert_eq!(ast.package.len(), 7);
    assert_eq!(ast.package[0].value, "a");
    assert_eq!(ast.package[1].value, "b");
    assert_eq!(ast.package[2].value, "c");
    assert_eq!(ast.package[3].value, "d");
    assert_eq!(ast.package[4].value, "e");
    assert_eq!(ast.package[5].value, "f");
    assert_eq!(ast.package[6].value, "models");
}

#[test]
fn test_single_segment_package_still_works() {
    // Backwards compatibility: simple package names
    let source = r#"
        package users;
        struct User {
            name: String,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    // Verify: works as before (single-element path)
    assert_eq!(ast.package.len(), 1);
    assert_eq!(ast.package[0].value, "users");
}

#[test]
fn test_dotted_import_path() {
    let source = r#"
        package test;
        use com.example.users.User;
        use org.company.models.Order;
        struct Test {
            user: User,
            order: Order,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    assert_eq!(ast.uses.len(), 2);
    assert_eq!(ast.uses[0].path.len(), 4);
    assert_eq!(ast.uses[1].path.len(), 4);
}

#[test]
fn test_ir_package_name_from_dotted_path() {
    let source = r#"
        package com.example.users;
        struct User {
            name: String,
        }
    "#;
    let result = parse_string_to_ir(source);
    assert!(
        result.is_ok(),
        "Failed to convert to IR: {:?}",
        result.err()
    );

    let schema = result.unwrap();
    // Verify package name is joined with dots
    assert!(schema.packages.contains_key("com.example.users"));
    assert_eq!(schema.packages.len(), 1);

    let package = schema.packages.get("com.example.users").unwrap();
    assert_eq!(package.name, "com.example.users");
}

#[test]
fn test_ir_multiple_dotted_packages() {
    let source1 = r#"
        package com.example.users;
        struct User {
            name: String,
        }
    "#;
    let source2 = r#"
        package com.example.orders;
        struct Order {
            total: f64,
        }
    "#;

    let ast1 = parse_string(source1).unwrap();
    let ast2 = parse_string(source2).unwrap();

    let converter = fluorite_codegen::idl::ast_to_ir::AstToIrConverter::new();
    let schema = converter.convert_files(&[ast1, ast2]).unwrap();

    assert_eq!(schema.packages.len(), 2);
    assert!(schema.packages.contains_key("com.example.users"));
    assert!(schema.packages.contains_key("com.example.orders"));
}

#[test]
fn test_two_segment_package() {
    // Test minimal dotted path (two segments)
    let source = r#"
        package myapp.models;
        struct User {
            name: String,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    assert_eq!(ast.package.len(), 2);
    assert_eq!(ast.package[0].value, "myapp");
    assert_eq!(ast.package[1].value, "models");
}

#[test]
fn test_use_path_with_many_segments() {
    // Test import with many segments
    let source = r#"
        package test;
        use a.b.c.d.e.f.g.Type;
        struct Test {
            value: Type,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    assert_eq!(ast.uses.len(), 1);
    assert_eq!(ast.uses[0].path.len(), 8);
    assert_eq!(ast.uses[0].path[0].value, "a");
    assert_eq!(ast.uses[0].path[7].value, "Type");
}
