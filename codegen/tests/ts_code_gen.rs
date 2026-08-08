use std::sync::Arc;

use fluorite_codegen::code_gen::{
    fs::MemoryFileSystem,
    ir::{
        IREnum, IREnumVariant, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct,
        IRType, IRTypeAlias, IRTypeAliasTarget, IRUnion, IRUnionVariant,
    },
    ts::{TsTemplateGenerator, TypeScriptOptions},
};

#[test]
fn test_typescript_options_default() {
    let options = TypeScriptOptions::new("/output".to_string());

    assert_eq!(options.output_dir, "/output");
    assert!(!options.single_file);
    assert_eq!(options.any_type, "unknown");
    assert!(!options.use_readonly);
}

#[test]
fn test_typescript_options_builder() {
    let options = TypeScriptOptions::new("/output".to_string())
        .with_single_file(true)
        .with_any_type("any")
        .with_readonly(true);

    assert!(options.single_file);
    assert_eq!(options.any_type, "any");
    assert!(options.use_readonly);
}

#[test]
fn test_ts_generates_interface() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/user.ts").unwrap();
    assert!(
        content.contains("export interface User"),
        "Should have User interface"
    );
    assert!(
        content.contains("firstName: string"),
        "Should have firstName field"
    );
    assert!(
        content.contains("lastName: string"),
        "Should have lastName field"
    );
    assert!(content.contains("age: number"), "Should have age as number");
    assert!(
        content.contains("active: boolean"),
        "Should have active as boolean"
    );

    Ok(())
}

#[test]
fn test_ts_generates_enum() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/gender.ts").unwrap();
    assert!(
        content.contains("export enum Gender"),
        "Should have Gender enum"
    );
    assert!(
        content.contains("Male = \"Male\""),
        "Should have Male variant"
    );
    assert!(
        content.contains("Female = \"Female\""),
        "Should have Female variant"
    );

    Ok(())
}

#[test]
fn test_ts_generates_discriminated_union() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/orders/address.ts").unwrap();
    assert!(
        content.contains("export type Address"),
        "Should have Address type"
    );
    assert!(
        content.contains("type: \""),
        "Should have discriminant field"
    );

    Ok(())
}

#[test]
fn test_ts_generates_type_alias() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let order_list_content = fs.get_string("/output/test/orders/orderList.ts").unwrap();
    assert!(
        order_list_content.contains("export type OrderList = Order[]"),
        "Should have list alias. Got: {}",
        order_list_content
    );

    let order_map_content = fs.get_string("/output/test/orders/orderMap.ts").unwrap();
    assert!(
        order_map_content.contains("export type OrderMap = Record<string, Order>"),
        "Should have map alias. Got: {}",
        order_map_content
    );

    Ok(())
}

#[test]
fn test_ts_single_file_mode() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_single_file(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    // Should only have index.ts
    let files = fs.files();
    assert!(
        files.contains_key("/output/test/users/index.ts"),
        "Should have index.ts"
    );
    assert!(
        !files.contains_key("/output/test/users/user.ts"),
        "Should NOT have user.ts"
    );

    let content = fs.get_string("/output/test/users/index.ts").unwrap();
    assert!(
        content.contains("export interface User"),
        "Should have User in index.ts"
    );
    assert!(
        content.contains("export enum Gender"),
        "Should have Gender in index.ts"
    );

    Ok(())
}

#[test]
fn test_ts_readonly_option() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_readonly(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/user.ts").unwrap();
    assert!(
        content.contains("readonly firstName: string"),
        "Should have readonly fields. Got: {}",
        content
    );

    Ok(())
}

#[test]
fn test_ts_any_type_option() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_any_type("any");
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    // PostCodeData has an 'instruction' field of type Any
    let content = fs
        .get_string("/output/test/orders/postCodeData.ts")
        .unwrap();
    assert!(
        content.contains("instruction: any"),
        "Should use custom any type. Got: {}",
        content
    );
    assert!(
        !content.contains("instruction: unknown"),
        "Should NOT use unknown"
    );

    Ok(())
}

#[test]
fn test_ts_optional_fields() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/orders/order.ts").unwrap();
    // shipping field is optional
    assert!(
        content.contains("shipping?:"),
        "Should have optional shipping field. Got: {}",
        content
    );

    Ok(())
}

#[test]
fn test_ts_index_file_exports() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/index.ts").unwrap();
    assert!(
        content.contains("export * from './user'"),
        "Should export user"
    );
    assert!(
        content.contains("export * from './gender'"),
        "Should export gender"
    );

    Ok(())
}

#[test]
fn test_ts_empty_definition_list() {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());

    let generator = TsTemplateGenerator::new(options, fs.clone());
    let schema = IRSchema {
        packages: std::collections::HashMap::new(),
    };
    let result = generator.generate_from_schema(&schema);

    assert!(result.is_ok());
    assert!(fs.files().is_empty());
}

// ============================================================================
// Helper functions
// ============================================================================

fn create_test_schema() -> IRSchema {
    use std::collections::HashMap;

    let mut packages = HashMap::new();

    // Users package
    let users_pkg = IRPackage {
        name: "test.users".to_string(),
        types: vec![
            IRType::Struct(IRStruct {
                name: "User".to_string(),
                fields: vec![
                    IRField {
                        name: "first_name".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "last_name".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "age".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::UInt32),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "gender".to_string(),
                        field_type: IRFieldType::Custom("Gender".to_string()),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "active".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::Bool),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                ],
                doc: None,
                deny_unknown_fields: false,
            }),
            IRType::Enum(IREnum {
                name: "Gender".to_string(),
                variants: vec![IREnumVariant::new("Male"), IREnumVariant::new("Female")],
                doc: None,
            }),
        ],
    };

    // Orders package
    let orders_pkg = IRPackage {
        name: "test.orders".to_string(),
        types: vec![
            IRType::Struct(IRStruct {
                name: "Order".to_string(),
                fields: vec![
                    IRField {
                        name: "id".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::UInt64),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "item".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "user".to_string(),
                        field_type: IRFieldType::Custom("User".to_string()),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "shipping".to_string(),
                        field_type: IRFieldType::Custom("Shipping".to_string()),
                        is_optional: true,
                        is_boxed: true,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "type".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: Some("order_type".to_string()),
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                ],
                doc: None,
                deny_unknown_fields: false,
            }),
            IRType::Struct(IRStruct {
                name: "Shipping".to_string(),
                fields: vec![
                    IRField {
                        name: "id".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "order".to_string(),
                        field_type: IRFieldType::Custom("Order".to_string()),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "address".to_string(),
                        field_type: IRFieldType::Custom("Address".to_string()),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                ],
                doc: None,
                deny_unknown_fields: false,
            }),
            // Address union with adjacently tagged format
            IRType::Union(IRUnion {
                name: "Address".to_string(),
                tag_field: "type".to_string(),
                content_field: "value".to_string(),
                variants: vec![
                    IRUnionVariant::Unit {
                        name: "Empty".to_string(),
                        doc: None,
                    },
                    IRUnionVariant::Newtype {
                        name: "PostCode".to_string(),
                        ty: IRFieldType::Custom("PostCodeData".to_string()),
                        doc: None,
                    },
                    IRUnionVariant::Newtype {
                        name: "AddressInfo".to_string(),
                        ty: IRFieldType::Custom("AddressInfoData".to_string()),
                        doc: None,
                    },
                ],
                doc: None,
            }),
            // Struct types used by union variants
            IRType::Struct(IRStruct {
                name: "PostCodeData".to_string(),
                fields: vec![
                    IRField {
                        name: "code".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "order".to_string(),
                        field_type: IRFieldType::Custom("Order".to_string()),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "instruction".to_string(),
                        field_type: IRFieldType::Any,
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                ],
                doc: None,
                deny_unknown_fields: false,
            }),
            IRType::Struct(IRStruct {
                name: "AddressInfoData".to_string(),
                fields: vec![
                    IRField {
                        name: "first_line".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                    IRField {
                        name: "second_line".to_string(),
                        field_type: IRFieldType::Primitive(IRPrimitive::String),
                        is_optional: false,
                        is_boxed: false,
                        rename: None,
                        doc: None,
                        alias: vec![],
                        default: None,
                        skip_if_none: false,
                        skip_if_default: false,
                        flatten: false,
                        deprecated: false,
                    },
                ],
                doc: None,
                deny_unknown_fields: false,
            }),
            IRType::TypeAlias(IRTypeAlias {
                name: "OrderList".to_string(),
                target: IRTypeAliasTarget::List(IRFieldType::Custom("Order".to_string())),
                doc: None,
            }),
            IRType::TypeAlias(IRTypeAlias {
                name: "OrderMap".to_string(),
                target: IRTypeAliasTarget::Map(
                    IRFieldType::Primitive(IRPrimitive::String),
                    IRFieldType::Custom("Order".to_string()),
                ),
                doc: None,
            }),
        ],
    };

    packages.insert("test.users".to_string(), users_pkg);
    packages.insert("test.orders".to_string(), orders_pkg);

    IRSchema { packages }
}
