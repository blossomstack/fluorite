use std::collections::BTreeMap;
use std::sync::Arc;

use fluorite_codegen::code_gen::{
    fs::MemoryFileSystem,
    ir::{
        IREnum, IREnumVariant, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct,
        IRType, IRTypeAlias, IRTypeAliasTarget, IRTypeRef, IRUnion, IRUnionVariant,
    },
    swift::{SwiftOptions, SwiftTemplateGenerator, SwiftVisibility},
};

#[test]
fn test_swift_options_default() {
    let options = SwiftOptions::new("/output".to_string());

    assert_eq!(options.output_dir, "/output");
    assert!(!options.single_file);
    assert_eq!(options.any_type, "AnyCodable");
    assert_eq!(options.visibility, SwiftVisibility::Public);
}

#[test]
fn test_swift_options_builder() {
    let options = SwiftOptions::new("/output".to_string())
        .with_single_file(true)
        .with_any_type("Any")
        .with_visibility(SwiftVisibility::Internal);

    assert!(options.single_file);
    assert_eq!(options.any_type, "Any");
    assert_eq!(options.visibility, SwiftVisibility::Internal);
}

#[test]
fn test_swift_generates_struct() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/User.swift").unwrap();
    assert!(
        content.contains("public struct User: Codable, Equatable, Sendable"),
        "Should have User struct with protocols"
    );
    assert!(
        content.contains("public let firstName: String"),
        "Should have firstName field"
    );
    assert!(
        content.contains("public let lastName: String"),
        "Should have lastName field"
    );
    assert!(
        content.contains("public let age: Int32"),
        "Should have age as Int32"
    );
    assert!(
        content.contains("public let active: Bool"),
        "Should have active as Bool"
    );

    Ok(())
}

#[test]
fn test_swift_generates_enum() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/users/Gender.swift").unwrap();
    assert!(
        content.contains("public enum Gender: String, Codable, Equatable, Sendable"),
        "Should have Gender enum with protocols"
    );
    assert!(
        content.contains("case male = \"Male\""),
        "Should have male variant with raw value"
    );
    assert!(
        content.contains("case female = \"Female\""),
        "Should have female variant with raw value"
    );

    Ok(())
}

#[test]
fn test_swift_generates_union() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/test/orders/Address.swift").unwrap();
    assert!(
        content.contains("public enum Address: Codable, Equatable, Sendable"),
        "Should have Address enum with protocols"
    );
    // Check for case declarations
    assert!(content.contains("case empty"), "Should have empty case");
    assert!(
        content.contains("case postCode(PostCodeData)"),
        "Should have postCode case with associated value"
    );
    // Check for custom Codable implementation
    assert!(
        content.contains("public init(from decoder: Decoder)"),
        "Should have custom decoder"
    );
    assert!(
        content.contains("public func encode(to encoder: Encoder)"),
        "Should have custom encoder"
    );
    // Check for CodingKeys
    assert!(content.contains("case type"), "Should have type CodingKey");
    assert!(
        content.contains("case value"),
        "Should have value CodingKey"
    );

    Ok(())
}

#[test]
fn test_swift_generates_type_alias() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs
        .get_string("/output/test/orders/OrderList.swift")
        .unwrap();
    assert!(
        content.contains("public typealias OrderList = [Order]"),
        "Should have OrderList typealias for array"
    );

    let content = fs.get_string("/output/test/orders/OrderMap.swift").unwrap();
    assert!(
        content.contains("public typealias OrderMap = [String: Order]"),
        "Should have OrderMap typealias for dictionary"
    );

    Ok(())
}

#[test]
fn test_swift_generates_single_file() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned()).with_single_file(true);
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    // Should generate single Users.swift file for users package
    let content = fs.get_string("/output/test/users/Users.swift").unwrap();
    assert!(content.contains("struct User"), "Should have User struct");
    assert!(content.contains("enum Gender"), "Should have Gender enum");

    // Should generate single Orders.swift file for orders package
    let content = fs.get_string("/output/test/orders/Orders.swift").unwrap();
    assert!(content.contains("struct Order"), "Should have Order struct");
    assert!(
        content.contains("enum Address"),
        "Should have Address union"
    );
    assert!(
        content.contains("typealias OrderList"),
        "Should have OrderList alias"
    );

    Ok(())
}

#[test]
fn test_swift_visibility_internal() -> anyhow::Result<()> {
    let schema = create_simple_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options =
        SwiftOptions::new("/output".to_owned()).with_visibility(SwiftVisibility::Internal);
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/simple/User.swift").unwrap();
    assert!(
        content.contains("internal struct User"),
        "Should use internal visibility"
    );
    assert!(
        content.contains("internal let id"),
        "Fields should use internal visibility"
    );

    Ok(())
}

#[test]
fn test_swift_optional_fields() -> anyhow::Result<()> {
    let schema = create_simple_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/simple/User.swift").unwrap();
    assert!(
        content.contains("email: String?"),
        "Optional field should have ? suffix"
    );

    Ok(())
}

#[test]
fn test_swift_coding_keys() -> anyhow::Result<()> {
    let schema = create_schema_with_rename();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/simple/User.swift").unwrap();
    assert!(
        content.contains("enum CodingKeys: String, CodingKey"),
        "Should have CodingKeys enum"
    );
    assert!(
        content.contains("case orderType = \"type\""),
        "Should map Swift name to JSON key"
    );

    Ok(())
}

#[test]
fn test_swift_primitive_types() -> anyhow::Result<()> {
    let schema = create_schema_with_primitives();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned());
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/simple/AllTypes.swift").unwrap();

    // Basic types
    assert!(content.contains("stringField: String"), "String mapping");
    assert!(content.contains("boolField: Bool"), "Bool mapping");
    assert!(content.contains("int32Field: Int32"), "Int32 mapping");
    assert!(content.contains("int64Field: Int64"), "Int64 mapping");
    assert!(content.contains("uint32Field: UInt32"), "UInt32 mapping");
    assert!(content.contains("uint64Field: UInt64"), "UInt64 mapping");
    assert!(
        content.contains("float32Field: Float"),
        "Float32 -> Float mapping"
    );
    assert!(
        content.contains("float64Field: Double"),
        "Float64 -> Double mapping"
    );

    // Foundation types
    assert!(content.contains("uuidField: UUID"), "UUID mapping");
    assert!(content.contains("decimalField: Decimal"), "Decimal mapping");
    assert!(
        content.contains("bytesField: Data"),
        "Bytes -> Data mapping"
    );
    assert!(content.contains("urlField: URL"), "URL mapping");
    assert!(
        content.contains("dateTimeField: Date"),
        "DateTime -> Date mapping"
    );

    Ok(())
}

#[test]
fn test_swift_any_type() -> anyhow::Result<()> {
    let schema = create_schema_with_any();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = SwiftOptions::new("/output".to_owned()).with_any_type("AnyCodable");
    let generator = SwiftTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let content = fs.get_string("/output/simple/Dynamic.swift").unwrap();
    assert!(
        content.contains("import FluoriteRuntime"),
        "Should import runtime when using Any type"
    );
    assert!(
        content.contains("data: AnyCodable"),
        "Should use AnyCodable for Any type"
    );

    Ok(())
}

// Helper functions to create test schemas

fn create_test_schema() -> IRSchema {
    let mut packages = BTreeMap::new();

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
                        field_type: IRFieldType::Primitive(IRPrimitive::Int32),
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
                fields: vec![IRField {
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
                }],
                doc: None,
                deny_unknown_fields: false,
            }),
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
                        ty: IRFieldType::Custom(IRTypeRef::new("test.orders", "PostCodeData")),
                        doc: None,
                    },
                ],
                doc: None,
            }),
            IRType::Struct(IRStruct {
                name: "PostCodeData".to_string(),
                fields: vec![IRField {
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
                }],
                doc: None,
                deny_unknown_fields: false,
            }),
            IRType::TypeAlias(IRTypeAlias {
                name: "OrderList".to_string(),
                target: IRTypeAliasTarget::List(IRFieldType::Custom(IRTypeRef::new(
                    "test.orders",
                    "Order",
                ))),
                doc: None,
            }),
            IRType::TypeAlias(IRTypeAlias {
                name: "OrderMap".to_string(),
                target: IRTypeAliasTarget::Map(
                    IRFieldType::Primitive(IRPrimitive::String),
                    IRFieldType::Custom(IRTypeRef::new("test.orders", "Order")),
                ),
                doc: None,
            }),
        ],
    };

    packages.insert("test.users".to_string(), users_pkg);
    packages.insert("test.orders".to_string(), orders_pkg);

    IRSchema { packages }
}

fn create_simple_schema() -> IRSchema {
    let mut packages = BTreeMap::new();

    let pkg = IRPackage {
        name: "simple".to_string(),
        types: vec![IRType::Struct(IRStruct {
            name: "User".to_string(),
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
                    name: "email".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::String),
                    is_optional: true,
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
        })],
    };

    packages.insert("simple".to_string(), pkg);
    IRSchema { packages }
}

fn create_schema_with_rename() -> IRSchema {
    let mut packages = BTreeMap::new();

    let pkg = IRPackage {
        name: "simple".to_string(),
        types: vec![IRType::Struct(IRStruct {
            name: "User".to_string(),
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
                    name: "order_type".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::String),
                    is_optional: false,
                    is_boxed: false,
                    rename: Some("type".to_string()),
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
        })],
    };

    packages.insert("simple".to_string(), pkg);
    IRSchema { packages }
}

fn create_schema_with_primitives() -> IRSchema {
    let mut packages = BTreeMap::new();

    let pkg = IRPackage {
        name: "simple".to_string(),
        types: vec![IRType::Struct(IRStruct {
            name: "AllTypes".to_string(),
            fields: vec![
                IRField {
                    name: "string_field".to_string(),
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
                    name: "bool_field".to_string(),
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
                IRField {
                    name: "int32_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Int32),
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
                    name: "int64_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Int64),
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
                    name: "uint32_field".to_string(),
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
                    name: "uint64_field".to_string(),
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
                    name: "float32_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Float32),
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
                    name: "float64_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Float64),
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
                    name: "uuid_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::UUID),
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
                    name: "decimal_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Decimal),
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
                    name: "bytes_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Bytes),
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
                    name: "url_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Url),
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
                    name: "date_time_field".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::DateTime),
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
        })],
    };

    packages.insert("simple".to_string(), pkg);
    IRSchema { packages }
}

fn create_schema_with_any() -> IRSchema {
    let mut packages = BTreeMap::new();

    let pkg = IRPackage {
        name: "simple".to_string(),
        types: vec![IRType::Struct(IRStruct {
            name: "Dynamic".to_string(),
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
                    name: "data".to_string(),
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
        })],
    };

    packages.insert("simple".to_string(), pkg);
    IRSchema { packages }
}
