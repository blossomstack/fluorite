//! Tests for TypeScript cross-package import generation

use std::collections::HashMap;
use std::sync::Arc;

use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use fluorite_codegen::code_gen::ir::IRSchema;
use fluorite_codegen::code_gen::ts::{TsTemplateGenerator, TypeScriptOptions};
use fluorite_codegen::idl::parse_strings_to_ir;

/// Generate TypeScript and return all file contents
fn generate_ts(schema: &IRSchema, single_file: bool) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_string()).with_single_file(single_file);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator
        .generate_from_schema(schema)
        .expect("Failed to generate");

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

#[test]
fn test_cross_package_import_in_struct() {
    // common.fl defines Address
    let common_fl = r#"
        package demo.common;

        struct Address {
            city: String,
            country: String,
        }
    "#;

    // users.fl imports Address from common
    let users_fl = r#"
        package demo.users;

        use demo.common.Address;

        struct User {
            name: String,
            home_address: Address,
        }
    "#;

    let schema = parse_strings_to_ir(&[common_fl, users_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false); // multi-file mode

    // Get the user.ts file
    let user_content = output
        .get("/output/demo/users/user.ts")
        .expect("user.ts should exist");

    // Should have cross-package import for Address
    assert!(
        user_content.contains("import { Address }"),
        "Should import Address. Got:\n{}",
        user_content
    );
    assert!(
        user_content.contains("../common"),
        "Should import from ../common. Got:\n{}",
        user_content
    );
}

#[test]
fn test_cross_package_import_in_union() {
    let common_fl = r#"
        package demo.common;
        struct Payload { data: String }
    "#;

    let events_fl = r#"
        package demo.events;

        use demo.common.Payload;

        union Event {
            Created(Payload),
            Deleted,
        }
    "#;

    let schema = parse_strings_to_ir(&[common_fl, events_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false);

    let event_content = output
        .get("/output/demo/events/event.ts")
        .expect("event.ts should exist");

    assert!(
        event_content.contains("import { Payload }"),
        "Should import Payload. Got:\n{}",
        event_content
    );
    assert!(
        event_content.contains("../common"),
        "Should import from ../common. Got:\n{}",
        event_content
    );
}

#[test]
fn test_cross_package_import_in_type_alias() {
    let users_fl = r#"
        package demo.users;
        struct User { name: String }
    "#;

    let collections_fl = r#"
        package demo.collections;

        use demo.users.User;

        type UserList = Vec<User>;
        type UserMap = Map<String, User>;
    "#;

    let schema = parse_strings_to_ir(&[users_fl, collections_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false);

    let user_list_content = output
        .get("/output/demo/collections/userList.ts")
        .expect("userList.ts should exist");

    assert!(
        user_list_content.contains("import { User }"),
        "Should import User. Got:\n{}",
        user_list_content
    );

    let user_map_content = output
        .get("/output/demo/collections/userMap.ts")
        .expect("userMap.ts should exist");

    assert!(
        user_map_content.contains("import { User }"),
        "Should import User. Got:\n{}",
        user_map_content
    );
}

#[test]
fn test_multiple_cross_package_imports() {
    let common_fl = r#"
        package demo.common;
        struct Address { city: String }
    "#;

    let users_fl = r#"
        package demo.users;
        struct User { name: String }
    "#;

    let orders_fl = r#"
        package demo.orders;

        use demo.common.Address;
        use demo.users.User;

        struct Order {
            user: User,
            shipping_address: Address,
        }
    "#;

    let schema = parse_strings_to_ir(&[common_fl, users_fl, orders_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false);

    let order_content = output
        .get("/output/demo/orders/order.ts")
        .expect("order.ts should exist");

    // Should have imports for both cross-package types
    assert!(
        order_content.contains("import { Address }"),
        "Should import Address. Got:\n{}",
        order_content
    );
    assert!(
        order_content.contains("import { User }"),
        "Should import User. Got:\n{}",
        order_content
    );
    assert!(
        order_content.contains("../common"),
        "Should import from ../common. Got:\n{}",
        order_content
    );
    assert!(
        order_content.contains("../users"),
        "Should import from ../users. Got:\n{}",
        order_content
    );
}

#[test]
fn test_same_package_import_still_works() {
    // This should continue to work as before
    let users_fl = r#"
        package demo.users;

        struct Address { city: String }

        struct User {
            name: String,
            address: Address,
        }
    "#;

    let schema = parse_strings_to_ir(&[users_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false);

    let user_content = output
        .get("/output/demo/users/user.ts")
        .expect("user.ts should exist");

    // Same-package import uses ./
    assert!(
        user_content.contains("import { Address } from './address'"),
        "Should have same-package import. Got:\n{}",
        user_content
    );
}

#[test]
fn test_nested_package_relative_path() {
    // Test deeply nested packages
    let common_fl = r#"
        package a.b.common;
        struct Shared { value: String }
    "#;

    let deep_fl = r#"
        package a.b.c.d.deep;

        use a.b.common.Shared;

        struct Deep {
            shared: Shared,
        }
    "#;

    let schema = parse_strings_to_ir(&[common_fl, deep_fl]).expect("Failed to parse");
    let output = generate_ts(&schema, false);

    let deep_content = output
        .get("/output/a/b/c/d/deep/deep.ts")
        .expect("deep.ts should exist");

    // Should navigate up 3 levels (c/d/deep -> common)
    assert!(
        deep_content.contains("import { Shared }"),
        "Should import Shared. Got:\n{}",
        deep_content
    );
    // Path should be ../../../common (up from deep, d, c to b, then into common)
    assert!(
        deep_content.contains("../../../common"),
        "Should have correct relative path. Got:\n{}",
        deep_content
    );
}
