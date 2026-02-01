//! Integration tests for the Fluorite IDL parser
//!
//! These tests verify that the IDL parser correctly parses .fl files
//! and produces the expected IR output.

use std::path::PathBuf;

use fluorite_codegen::idl::{parse_file, parse_files, parse_string, parse_string_to_ir};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
}

#[test]
fn test_parse_users_fl() {
    let path = fixtures_dir().join("users.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse users.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.package.value, "users");
    assert_eq!(ast.items.len(), 3); // User, UserStatus, UserList
}

#[test]
fn test_parse_orders_fl() {
    let path = fixtures_dir().join("orders.fl");
    let result = parse_file(&path);
    assert!(
        result.is_ok(),
        "Failed to parse orders.fl: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.package.value, "orders");
    assert_eq!(ast.uses.len(), 1);
    assert_eq!(ast.uses[0].path.len(), 2);
    assert_eq!(ast.uses[0].path[0].value, "users");
    assert_eq!(ast.uses[0].path[1].value, "User");
}

#[test]
fn test_parse_multiple_files() {
    let paths = vec![
        fixtures_dir().join("users.fl"),
        fixtures_dir().join("orders.fl"),
    ];
    let result = parse_files(&paths);
    assert!(result.is_ok(), "Failed to parse files: {:?}", result.err());

    let asts = result.unwrap();
    assert_eq!(asts.len(), 2);
}

#[test]
fn test_parse_struct_with_all_primitive_types() {
    let source = r#"
        package test;
        struct AllTypes {
            s: String,
            b: bool,
            i32: i32,
            i64: i64,
            u32: u32,
            u64: u64,
            f32: f32,
            f64: f64,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    match &ast.items[0] {
        fluorite_codegen::idl::ast::AstItem::Struct(s) => {
            assert_eq!(s.fields.len(), 8);
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_parse_struct_with_extended_types() {
    let source = r#"
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
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_generic_types() {
    let source = r#"
        package test;
        struct Generics {
            list: Vec<String>,
            optional: Option<i32>,
            map: Map<String, User>,
            nested: Vec<Option<String>>,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_enum_with_variants() {
    let source = r#"
        package test;
        enum Status {
            Pending,
            Active,
            Inactive,
            Deleted,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    match &ast.items[0] {
        fluorite_codegen::idl::ast::AstItem::Enum(e) => {
            assert_eq!(e.variants.len(), 4);
            assert_eq!(e.variants[0].name.value, "Pending");
            assert_eq!(e.variants[3].name.value, "Deleted");
        }
        _ => panic!("Expected enum"),
    }
}

#[test]
fn test_parse_union_with_variants() {
    let source = r#"
        package test;
        struct User {}
        struct Order {}
        union Event {
            UserCreated(User),
            OrderPlaced(Order),
            SimpleEvent,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    match &ast.items[2] {
        fluorite_codegen::idl::ast::AstItem::Union(u) => {
            assert_eq!(u.variants.len(), 3);
            assert_eq!(u.variants[0].name.value, "UserCreated");
            assert!(u.variants[0].inner_type.is_some());
            assert_eq!(u.variants[2].name.value, "SimpleEvent");
            assert!(u.variants[2].inner_type.is_none());
        }
        _ => panic!("Expected union"),
    }
}

#[test]
fn test_parse_type_alias() {
    let source = r#"
        package test;
        type UserList = Vec<User>;
        type UserMap = Map<String, User>;
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2);
}

#[test]
fn test_parse_doc_comments() {
    let source = r#"
        package test;
        /// A user in the system
        /// with multiple lines
        struct User {
            /// The user's name
            name: String,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    match &ast.items[0] {
        fluorite_codegen::idl::ast::AstItem::Struct(s) => {
            assert!(s.doc.as_ref().unwrap().contains("A user in the system"));
            assert!(s.fields[0].doc.as_ref().unwrap().contains("user's name"));
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_parse_attributes() {
    let source = r#"
        package test;
        #[rename_all = "camelCase"]
        #[deny_unknown_fields]
        struct User {
            #[rename = "userName"]
            #[deprecated]
            name: String,
            #[skip_if_none]
            email: Option<String>,
        }
    "#;
    let result = parse_string(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let ast = result.unwrap();
    match &ast.items[0] {
        fluorite_codegen::idl::ast::AstItem::Struct(s) => {
            assert_eq!(s.attrs.len(), 2);
            assert_eq!(s.fields[0].attrs.len(), 2);
            assert_eq!(s.fields[1].attrs.len(), 1);
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_convert_to_ir() {
    let source = r#"
        package test;
        struct User {
            name: String,
            age: Option<u32>,
        }
        enum Status {
            Active,
            Inactive,
        }
    "#;
    let result = parse_string_to_ir(source);
    assert!(
        result.is_ok(),
        "Failed to convert to IR: {:?}",
        result.err()
    );

    let schema = result.unwrap();
    assert!(schema.packages.contains_key("test"));

    let package = schema.packages.get("test").unwrap();
    assert_eq!(package.types.len(), 2);
}

#[test]
fn test_ir_field_attributes() {
    let source = r#"
        package test;
        struct User {
            #[rename = "user_name"]
            #[box]
            name: String,
            #[skip_if_none]
            email: Option<String>,
        }
    "#;
    let result = parse_string_to_ir(source);
    assert!(
        result.is_ok(),
        "Failed to convert to IR: {:?}",
        result.err()
    );

    let schema = result.unwrap();
    let package = schema.packages.get("test").unwrap();

    match &package.types[0] {
        fluorite_codegen::code_gen::ir::IRType::Struct(s) => {
            assert_eq!(s.fields[0].rename, Some("user_name".to_string()));
            assert!(s.fields[0].is_boxed);
            assert!(s.fields[1].skip_if_none);
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_ir_optional_field() {
    let source = r#"
        package test;
        struct User {
            name: String,
            age: Option<u32>,
        }
    "#;
    let result = parse_string_to_ir(source);
    assert!(
        result.is_ok(),
        "Failed to convert to IR: {:?}",
        result.err()
    );

    let schema = result.unwrap();
    let package = schema.packages.get("test").unwrap();

    match &package.types[0] {
        fluorite_codegen::code_gen::ir::IRType::Struct(s) => {
            assert!(!s.fields[0].is_optional);
            assert!(s.fields[1].is_optional);
        }
        _ => panic!("Expected struct"),
    }
}
