use std::fs;
use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::{FileSystem, FsWriter, MemoryFileSystem},
        ir::{
            IRBuilder, IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct,
            IRType, IRTypeAlias, IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
        },
        rust::{RustOptions, RustTemplateGenerator, Visibility},
        validation::{ValidationError, Validator},
    },
    definitions::{CustomType, Definition, DefinitionConfig, Field, FieldConfig},
};

pub(crate) fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[test]
fn test_rust_code_gen() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/tmp/test_fluorite".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    // Verify key files exist
    let files = fs.files();
    assert!(files.keys().any(|k| k.contains("users/mod.rs")));
    assert!(files.keys().any(|k| k.contains("orders/mod.rs")));

    Ok(())
}

#[test]
fn test_custom_derives() {
    let options = RustOptions::new("/tmp/test".to_owned())
        .with_derives(vec!["Debug".to_string(), "Clone".to_string()]);
    assert_eq!(options.derives, vec!["Debug", "Clone"]);
}

#[test]
fn test_default_derives() {
    let options = RustOptions::new("/tmp/test".to_owned());
    assert!(options.derives.contains(&"Debug".to_string()));
    assert!(options.derives.contains(&"serde::Serialize".to_string()));
}

#[test]
fn test_ir_builder_creates_schema() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();

    let schema = IRBuilder::new().build(&[d1, d2]).unwrap();

    // Should have two packages
    assert_eq!(schema.packages.len(), 2);
    assert!(schema.packages.contains_key("protocols.users"));
    assert!(schema.packages.contains_key("protocols.orders"));

    // Users package should have User and Gender
    let users_pkg = schema.packages.get("protocols.users").unwrap();
    assert_eq!(users_pkg.types.len(), 2);
}

#[test]
fn test_ir_builder_handles_unions() {
    let d = deserialize_definition_file("../examples/orders.yml").unwrap();
    let schema = IRBuilder::new().build(&[d]).unwrap();

    let orders_pkg = schema.packages.get("protocols.orders").unwrap();
    let address_union = orders_pkg
        .types
        .iter()
        .find(|t| t.name() == "Address")
        .unwrap();

    if let IRType::Union(u) = address_union {
        assert_eq!(u.tag_field, "type");
        assert_eq!(u.variants.len(), 3);
        assert_eq!(u.style, IRUnionStyle::Inline);
    } else {
        panic!("Expected union type");
    }
}

#[test]
fn test_validation_passes_for_valid_schema() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();
    let schema = IRBuilder::new().build(&[d1, d2]).unwrap();

    let errors = Validator::new().validate(&schema);
    assert!(
        errors.is_empty(),
        "Expected no errors but got: {:?}",
        errors
    );
}

#[test]
fn test_memory_filesystem_write_and_read() {
    let fs = MemoryFileSystem::new();
    fs.write_file("test/file.txt", b"hello world").unwrap();

    let content = fs.read_file("test/file.txt").unwrap();
    assert_eq!(content, b"hello world");
}

#[test]
fn test_memory_filesystem_append() {
    let fs = MemoryFileSystem::new();
    fs.write_file("test/file.txt", b"hello").unwrap();
    fs.append_file("test/file.txt", b" world").unwrap();

    let content = fs.read_file("test/file.txt").unwrap();
    assert_eq!(content, b"hello world");
}

#[test]
fn test_template_generator_produces_valid_rust() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());

    generator.generate(&[d1, d2]).unwrap();

    // Check that files were generated
    let files = fs.files();
    assert!(files.keys().any(|k| k.contains("mod.rs")));

    // Check content of generated struct
    let users_mod = fs.get_string("/output/protocols/users/mod.rs").unwrap();
    assert!(users_mod.contains("pub struct User"));
    assert!(users_mod.contains("first_name: String"));
}

#[test]
fn test_full_integration() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned())
        .with_derives(vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ])
        .with_generate_new(false);

    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    // Verify users package
    let users_content = fs
        .get_string("/output/protocols/users/mod.rs")
        .expect("Users module should exist");

    assert!(
        users_content.contains("pub struct User"),
        "Should have User struct"
    );
    assert!(
        users_content.contains("first_name: String"),
        "Should have first_name field"
    );
    assert!(
        users_content.contains("pub enum Gender"),
        "Should have Gender enum"
    );
    assert!(users_content.contains("Male"), "Should have Male variant");
    assert!(
        !users_content.contains("derive_new::new"),
        "Should not have derive_new"
    );

    // Verify orders package
    let orders_content = fs
        .get_string("/output/protocols/orders/mod.rs")
        .expect("Orders module should exist");

    assert!(
        orders_content.contains("pub struct Order"),
        "Should have Order struct"
    );
    assert!(
        orders_content.contains("pub type OrderList = Vec<"),
        "Should have OrderList"
    );
    assert!(
        orders_content.contains("pub type OrderMap = HashMap<"),
        "Should have OrderMap"
    );
    assert!(
        orders_content.contains("#[serde(tag = \"type\")]"),
        "Should have tagged union"
    );
    assert!(
        orders_content.contains("pub enum Address"),
        "Should have Address union"
    );

    Ok(())
}

#[test]
fn test_multi_file_mode() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned()).with_single_file(false);

    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let files = fs.files();

    // Should have separate files
    assert!(
        files.contains_key("/output/protocols/users/user.rs"),
        "Should have user.rs"
    );
    assert!(
        files.contains_key("/output/protocols/users/gender.rs"),
        "Should have gender.rs"
    );
    assert!(
        files.contains_key("/output/protocols/users/mod.rs"),
        "Should have mod.rs"
    );

    // mod.rs should have module declarations
    let mod_content = fs.get_string("/output/protocols/users/mod.rs").unwrap();
    assert!(
        mod_content.contains("mod user;"),
        "Should declare user module"
    );
    assert!(
        mod_content.contains("mod gender;"),
        "Should declare gender module"
    );
    assert!(mod_content.contains("pub use"), "Should have pub use");

    Ok(())
}

#[test]
fn test_field_rename() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let orders_content = fs
        .get_string("/output/protocols/orders/mod.rs")
        .expect("Orders module should exist");

    // The field named "type" in YAML should be renamed to "order_type" in code
    // with a serde rename attribute
    assert!(
        orders_content.contains("#[serde(rename = \"type\")]"),
        "Should have serde rename attribute for type field. Content: {}",
        orders_content
    );
    assert!(
        orders_content.contains("order_type: String"),
        "Should use renamed field name. Content: {}",
        orders_content
    );

    Ok(())
}

#[test]
fn test_boxed_optional_field() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let orders_content = fs
        .get_string("/output/protocols/orders/mod.rs")
        .expect("Orders module should exist");

    // The shipping field should be Option<Box<...>>
    assert!(
        orders_content.contains("Option<Box<"),
        "Should have optional boxed field. Content: {}",
        orders_content
    );

    Ok(())
}

// ============================================================================
// Unit tests for IR types
// ============================================================================

mod ir_field_tests {
    use super::*;

    #[test]
    fn test_code_name_without_rename() {
        let field = IRField {
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
            };
        assert_eq!(field.code_name(), "first_name");
        assert_eq!(field.original_name(), "first_name");
        assert!(!field.needs_rename());
    }

    #[test]
    fn test_code_name_with_rename() {
        let field = IRField {
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
            };
        assert_eq!(field.code_name(), "order_type");
        assert_eq!(field.original_name(), "type");
        assert!(field.needs_rename());
    }

    #[test]
    fn test_field_optional_and_boxed() {
        let field = IRField {
            name: "data".to_string(),
            field_type: IRFieldType::Custom("SomeType".to_string()),
            is_optional: true,
            is_boxed: true,
            rename: None,
            doc: Some("Documentation".to_string()),
            alias: vec![],
            default: None,
            skip_if_none: false,
            skip_if_default: false,
            flatten: false,
            deprecated: false,
            };
        assert!(field.is_optional);
        assert!(field.is_boxed);
        assert_eq!(field.doc, Some("Documentation".to_string()));
    }
}

mod ir_type_tests {
    use super::*;

    #[test]
    fn test_struct_name() {
        let ir_type = IRType::Struct(IRStruct {
            name: "User".to_string(),
            fields: vec![],
            is_union_variant: false,
            doc: None,
            rename_all: None,
            deny_unknown_fields: false,
            });
        assert_eq!(ir_type.name(), "User");
        assert!(!ir_type.is_internal());
    }

    #[test]
    fn test_struct_union_variant_is_internal() {
        let ir_type = IRType::Struct(IRStruct {
            name: "PostCode".to_string(),
            fields: vec![],
            is_union_variant: true,
            doc: None,
            rename_all: None,
            deny_unknown_fields: false,
            });
        assert_eq!(ir_type.name(), "PostCode");
        assert!(ir_type.is_internal());
    }

    #[test]
    fn test_enum_name() {
        let ir_type = IRType::Enum(IREnum {
            name: "Status".to_string(),
            variants: vec!["Active".to_string(), "Inactive".to_string()],
            doc: None,
        });
        assert_eq!(ir_type.name(), "Status");
        assert!(!ir_type.is_internal());
    }

    #[test]
    fn test_union_name() {
        let ir_type = IRType::Union(IRUnion {
            name: "Address".to_string(),
            tag_field: "type".to_string(),
            variants: vec![],
            style: IRUnionStyle::Inline,
            doc: None,
        });
        assert_eq!(ir_type.name(), "Address");
        assert!(!ir_type.is_internal());
    }

    #[test]
    fn test_type_alias_name() {
        let ir_type = IRType::TypeAlias(IRTypeAlias {
            name: "OrderList".to_string(),
            target: IRTypeAliasTarget::List(IRFieldType::Custom("Order".to_string())),
            doc: None,
        });
        assert_eq!(ir_type.name(), "OrderList");
        assert!(!ir_type.is_internal());
    }
}

mod ir_union_variant_tests {
    use super::*;

    #[test]
    fn test_unit_variant_name() {
        let variant = IRUnionVariant::Unit("Empty".to_string());
        assert_eq!(variant.name(), "Empty");
    }

    #[test]
    fn test_inline_variant_name() {
        let variant = IRUnionVariant::Inline("PostCode".to_string(), vec![]);
        assert_eq!(variant.name(), "PostCode");
    }

    #[test]
    fn test_newtype_variant_name() {
        let variant = IRUnionVariant::Newtype("Info".to_string(), "AddressInfo".to_string());
        assert_eq!(variant.name(), "Info");
    }
}

// ============================================================================
// Unit tests for IRBuilder
// ============================================================================

mod ir_builder_tests {
    use super::*;

    fn create_definition(package: &str, types: Vec<CustomType>) -> Definition {
        Definition {
            configs: DefinitionConfig {
                rust_package: Some(package.to_string()),
            },
            types,
        }
    }

    #[test]
    fn test_empty_definitions() {
        let schema = IRBuilder::new().build(&[]).unwrap();
        assert!(schema.packages.is_empty());
    }

    #[test]
    fn test_missing_rust_package() {
        let def = Definition {
            configs: DefinitionConfig { rust_package: None },
            types: vec![CustomType::Enum {
                description: None,
                name: "Test".to_string(),
                values: vec!["A".to_string()],
                }],
        };

        let result = IRBuilder::new().build(&[def]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rust_package"));
    }

    #[test]
    fn test_all_primitive_types() {
        let def = create_definition(
            "test.package",
            vec![CustomType::Object {
                configs: None,
                description: None,
                name: "AllPrimitives".to_string(),
                fields: vec![
                    Field {
                        name: "s".to_string(),
                        field_type: "String".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "b".to_string(),
                        field_type: "Bool".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "dt".to_string(),
                        field_type: "DateTime".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "u32".to_string(),
                        field_type: "UInt32".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "u64".to_string(),
                        field_type: "UInt64".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "i32".to_string(),
                        field_type: "Int32".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "i64".to_string(),
                        field_type: "Int64".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "f32".to_string(),
                        field_type: "Float32".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "f64".to_string(),
                        field_type: "Float64".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                    Field {
                        name: "any".to_string(),
                        field_type: "Any".to_string(),
                        optional: None,
                        configs: None,
                        description: None,
                        deprecated: None,
                        },
                ],
            }],
        );

        let schema = IRBuilder::new().build(&[def]).unwrap();
        let pkg = schema.packages.get("test.package").unwrap();
        assert_eq!(pkg.types.len(), 1);

        if let IRType::Struct(s) = &pkg.types[0] {
            assert_eq!(s.fields.len(), 10);
            // Check each field type
            assert!(matches!(
                s.fields[0].field_type,
                IRFieldType::Primitive(IRPrimitive::String)
            ));
            assert!(matches!(
                s.fields[1].field_type,
                IRFieldType::Primitive(IRPrimitive::Bool)
            ));
            assert!(matches!(s.fields[9].field_type, IRFieldType::Any));
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_field_with_rename_config() {
        let def = create_definition(
            "test.package",
            vec![CustomType::Object {
                configs: None,
                description: None,
                name: "Test".to_string(),
                fields: vec![Field {
                    name: "type".to_string(),
                    field_type: "String".to_string(),
                    optional: None,
                    configs: Some(FieldConfig {
                        rename: Some("kind".to_string()),
                        rust_type_wrapper: None,
                        alias: None,
                        default: None,
                        rust: None,
                        }),
                    description: None,
                    deprecated: None,
                }],
            }],
        );

        let schema = IRBuilder::new().build(&[def]).unwrap();
        let pkg = schema.packages.get("test.package").unwrap();

        if let IRType::Struct(s) = &pkg.types[0] {
            assert_eq!(s.fields[0].name, "type");
            assert_eq!(s.fields[0].rename, Some("kind".to_string()));
            assert!(s.fields[0].needs_rename());
            assert_eq!(s.fields[0].code_name(), "kind");
        } else {
            panic!("Expected struct type");
        }
    }

    #[test]
    fn test_list_type() {
        let def = create_definition(
            "test.package",
            vec![CustomType::List {
                description: None,
                name: "StringList".to_string(),
                item_type: "String".to_string(),
                }],
        );

        let schema = IRBuilder::new().build(&[def]).unwrap();
        let pkg = schema.packages.get("test.package").unwrap();

        if let IRType::TypeAlias(a) = &pkg.types[0] {
            assert_eq!(a.name, "StringList");
            if let IRTypeAliasTarget::List(inner) = &a.target {
                assert!(matches!(inner, IRFieldType::Primitive(IRPrimitive::String)));
            } else {
                panic!("Expected List target");
            }
        } else {
            panic!("Expected TypeAlias");
        }
    }

    #[test]
    fn test_map_type() {
        let def = create_definition(
            "test.package",
            vec![CustomType::Map {
                description: None,
                name: "StringMap".to_string(),
                key_type: "String".to_string(),
                value_type: "Int32".to_string(),
                }],
        );

        let schema = IRBuilder::new().build(&[def]).unwrap();
        let pkg = schema.packages.get("test.package").unwrap();

        if let IRType::TypeAlias(a) = &pkg.types[0] {
            assert_eq!(a.name, "StringMap");
            if let IRTypeAliasTarget::Map(key, value) = &a.target {
                assert!(matches!(key, IRFieldType::Primitive(IRPrimitive::String)));
                assert!(matches!(value, IRFieldType::Primitive(IRPrimitive::Int32)));
            } else {
                panic!("Expected Map target");
            }
        } else {
            panic!("Expected TypeAlias");
        }
    }

    #[test]
    fn test_union_with_extern_style() {
        use fluorite_codegen::definitions::{TypeConfig, UnionStyle};

        let def = create_definition(
            "test.package",
            vec![
                CustomType::Object {
                    configs: None,
                    description: None,
                    name: "VariantA".to_string(),
                    fields: vec![],
                    },
                CustomType::Union {
                    description: None,
                    name: "MyUnion".to_string(),
                    type_tag: "type".to_string(),
                    values: vec!["VariantA".to_string()],
                    configs: Some(TypeConfig {
                        union_style: Some(UnionStyle::Extern),
                        rename_all: None,
                        rust: None,
                        }),
                },
            ],
        );

        let schema = IRBuilder::new().build(&[def]).unwrap();
        let pkg = schema.packages.get("test.package").unwrap();

        let union_type = pkg.types.iter().find(|t| t.name() == "MyUnion").unwrap();
        if let IRType::Union(u) = union_type {
            assert_eq!(u.style, IRUnionStyle::Extern);
            assert!(matches!(u.variants[0], IRUnionVariant::Newtype(_, _)));
        } else {
            panic!("Expected union type");
        }
    }
}

// ============================================================================
// Unit tests for Validator
// ============================================================================

mod validator_tests {
    use super::*;

    fn create_schema(packages: Vec<(String, Vec<IRType>)>) -> IRSchema {
        let mut pkg_map = std::collections::HashMap::new();
        for (name, types) in packages {
            pkg_map.insert(name.clone(), IRPackage { name, types });
        }
        IRSchema { packages: pkg_map }
    }

    #[test]
    fn test_valid_schema_passes() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![
                IRType::Struct(IRStruct {
                    name: "User".to_string(),
                    fields: vec![IRField {
                        name: "name".to_string(),
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
                    is_union_variant: false,
                    doc: None,
                    rename_all: None,
                    deny_unknown_fields: false,
                }),
                IRType::Enum(IREnum {
                    name: "Status".to_string(),
                    variants: vec!["Active".to_string()],
                    doc: None,
                }),
            ],
        )]);

        let errors = Validator::new().validate(&schema);
        assert!(
            errors.is_empty(),
            "Expected no errors but got: {:?}",
            errors
        );
    }

    #[test]
    fn test_unknown_type_reference() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Struct(IRStruct {
                name: "User".to_string(),
                fields: vec![IRField {
                    name: "address".to_string(),
                    field_type: IRFieldType::Custom("NonExistentType".to_string()),
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
                is_union_variant: false,
                doc: None,
                rename_all: None,
                deny_unknown_fields: false,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType {
                type_name,
                referenced_from,
                field_name,
            } if type_name == "NonExistentType"
                && referenced_from == "User"
                && *field_name == Some("address".to_string())
        ));
    }

    #[test]
    fn test_duplicate_types() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![
                IRType::Struct(IRStruct {
                    name: "User".to_string(),
                    fields: vec![],
                    is_union_variant: false,
                    doc: None,
                    rename_all: None,
                    deny_unknown_fields: false,
                    }),
                IRType::Struct(IRStruct {
                    name: "User".to_string(),
                    fields: vec![],
                    is_union_variant: false,
                    doc: None,
                    rename_all: None,
                    deny_unknown_fields: false,
                    }),
            ],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::DuplicateType {
                type_name,
                package,
            } if type_name == "User" && package == "test"
        ));
    }

    #[test]
    fn test_empty_enum() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Enum(IREnum {
                name: "EmptyEnum".to_string(),
                variants: vec![],
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::EmptyEnum { type_name } if type_name == "EmptyEnum"
        ));
    }

    #[test]
    fn test_empty_union() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Union(IRUnion {
                name: "EmptyUnion".to_string(),
                tag_field: "type".to_string(),
                variants: vec![],
                style: IRUnionStyle::Inline,
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::EmptyUnion { type_name } if type_name == "EmptyUnion"
        ));
    }

    #[test]
    fn test_unknown_type_in_union_inline_variant() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Union(IRUnion {
                name: "TestUnion".to_string(),
                tag_field: "type".to_string(),
                variants: vec![IRUnionVariant::Inline("NonExistent".to_string(), vec![])],
                style: IRUnionStyle::Inline,
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType { type_name, .. } if type_name == "NonExistent"
        ));
    }

    #[test]
    fn test_unknown_type_in_union_newtype_variant() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Union(IRUnion {
                name: "TestUnion".to_string(),
                tag_field: "type".to_string(),
                variants: vec![IRUnionVariant::Newtype(
                    "Var".to_string(),
                    "NonExistent".to_string(),
                )],
                style: IRUnionStyle::Extern,
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType { type_name, .. } if type_name == "NonExistent"
        ));
    }

    #[test]
    fn test_unknown_type_in_list_alias() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::TypeAlias(IRTypeAlias {
                name: "ItemList".to_string(),
                target: IRTypeAliasTarget::List(IRFieldType::Custom("NonExistent".to_string())),
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType { type_name, .. } if type_name == "NonExistent"
        ));
    }

    #[test]
    fn test_unknown_type_in_map_key() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::TypeAlias(IRTypeAlias {
                name: "ItemMap".to_string(),
                target: IRTypeAliasTarget::Map(
                    IRFieldType::Custom("NonExistent".to_string()),
                    IRFieldType::Primitive(IRPrimitive::String),
                ),
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType { type_name, field_name, .. }
            if type_name == "NonExistent" && *field_name == Some("key".to_string())
        ));
    }

    #[test]
    fn test_unknown_type_in_map_value() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::TypeAlias(IRTypeAlias {
                name: "ItemMap".to_string(),
                target: IRTypeAliasTarget::Map(
                    IRFieldType::Primitive(IRPrimitive::String),
                    IRFieldType::Custom("NonExistent".to_string()),
                ),
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::UnknownType { type_name, field_name, .. }
            if type_name == "NonExistent" && *field_name == Some("value".to_string())
        ));
    }

    #[test]
    fn test_cross_package_type_references() {
        let schema = create_schema(vec![
            (
                "package.a".to_string(),
                vec![IRType::Struct(IRStruct {
                    name: "TypeA".to_string(),
                    fields: vec![],
                    is_union_variant: false,
                    doc: None,
                    rename_all: None,
                    deny_unknown_fields: false,
                    })],
            ),
            (
                "package.b".to_string(),
                vec![IRType::Struct(IRStruct {
                    name: "TypeB".to_string(),
                    fields: vec![IRField {
                        name: "ref_a".to_string(),
                        field_type: IRFieldType::Custom("TypeA".to_string()),
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
                    is_union_variant: false,
                    doc: None,
                    rename_all: None,
                    deny_unknown_fields: false,
                })],
            ),
        ]);

        let errors = Validator::new().validate(&schema);
        assert!(
            errors.is_empty(),
            "Cross-package reference should be valid: {:?}",
            errors
        );
    }

    #[test]
    fn test_primitive_types_recognized() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Struct(IRStruct {
                name: "AllPrimitives".to_string(),
                fields: vec![
                    IRField {
                        name: "s".to_string(),
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
                        name: "any".to_string(),
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
                is_union_variant: false,
                doc: None,
                rename_all: None,
                deny_unknown_fields: false,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_unit_variant_always_valid() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Union(IRUnion {
                name: "TestUnion".to_string(),
                tag_field: "type".to_string(),
                variants: vec![
                    IRUnionVariant::Unit("UnitA".to_string()),
                    IRUnionVariant::Unit("UnitB".to_string()),
                ],
                style: IRUnionStyle::Inline,
                doc: None,
            })],
        )]);

        let errors = Validator::new().validate(&schema);
        assert!(errors.is_empty());
    }
}

// ============================================================================
// Unit tests for FileSystem
// ============================================================================

mod filesystem_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_memory_filesystem_exists() {
        let fs = MemoryFileSystem::new();
        assert!(!fs.exists("test/file.txt"));

        fs.write_file("test/file.txt", b"content").unwrap();
        assert!(fs.exists("test/file.txt"));
    }

    #[test]
    fn test_memory_filesystem_read_nonexistent() {
        let fs = MemoryFileSystem::new();
        let result = fs.read_file("nonexistent/file.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_memory_filesystem_create_dir_all_is_noop() {
        let fs = MemoryFileSystem::new();
        // Should not fail, just no-op
        fs.create_dir_all("some/deep/path").unwrap();
    }

    #[test]
    fn test_memory_filesystem_overwrite() {
        let fs = MemoryFileSystem::new();
        fs.write_file("test.txt", b"first").unwrap();
        fs.write_file("test.txt", b"second").unwrap();

        let content = fs.read_file("test.txt").unwrap();
        assert_eq!(content, b"second");
    }

    #[test]
    fn test_memory_filesystem_multiple_appends() {
        let fs = MemoryFileSystem::new();
        fs.append_file("test.txt", b"a").unwrap();
        fs.append_file("test.txt", b"b").unwrap();
        fs.append_file("test.txt", b"c").unwrap();

        let content = fs.read_file("test.txt").unwrap();
        assert_eq!(content, b"abc");
    }

    #[test]
    fn test_memory_filesystem_get_string() {
        let fs = MemoryFileSystem::new();
        fs.write_file("test.txt", b"hello").unwrap();

        let content = fs.get_string("test.txt");
        assert_eq!(content, Some("hello".to_string()));
    }

    #[test]
    fn test_memory_filesystem_get_string_nonexistent() {
        let fs = MemoryFileSystem::new();
        let content = fs.get_string("nonexistent.txt");
        assert_eq!(content, None);
    }

    #[test]
    fn test_memory_filesystem_files_returns_clone() {
        let fs = MemoryFileSystem::new();
        fs.write_file("a.txt", b"1").unwrap();
        fs.write_file("b.txt", b"2").unwrap();

        let files = fs.files();
        assert_eq!(files.len(), 2);
        assert!(files.contains_key("a.txt"));
        assert!(files.contains_key("b.txt"));
    }

    #[test]
    fn test_fs_writer_write_mode() {
        let fs = Arc::new(MemoryFileSystem::new());
        let mut writer = FsWriter::new(fs.clone(), "test.txt".to_string(), false);

        writer.write_all(b"hello ").unwrap();
        writer.write_all(b"world").unwrap();
        writer.flush().unwrap();

        let content = fs.get_string("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_fs_writer_append_mode() {
        let fs = Arc::new(MemoryFileSystem::new());
        fs.write_file("test.txt", b"existing ").unwrap();

        let mut writer = FsWriter::new(fs.clone(), "test.txt".to_string(), true);
        writer.write_all(b"new content").unwrap();
        writer.flush().unwrap();

        let content = fs.get_string("test.txt").unwrap();
        assert_eq!(content, "existing new content");
    }

    #[test]
    fn test_fs_writer_drop_flushes() {
        let fs = Arc::new(MemoryFileSystem::new());
        {
            let mut writer = FsWriter::new(fs.clone(), "test.txt".to_string(), false);
            writer.write_all(b"auto flush on drop").unwrap();
            // writer dropped here
        }

        let content = fs.get_string("test.txt").unwrap();
        assert_eq!(content, "auto flush on drop");
    }
}

// ============================================================================
// Unit tests for RustOptions
// ============================================================================

mod rust_options_tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = RustOptions::new("/output".to_string());

        assert_eq!(options.output_dir, "/output");
        assert!(options.single_file);
        assert_eq!(options.any_type, "fluorite::Any");
        assert!(options.generate_new);
        assert_eq!(options.visibility, Visibility::Public);
        // Default derives
        assert!(options.derives.contains(&"Debug".to_string()));
        assert!(options.derives.contains(&"Clone".to_string()));
        assert!(options.derives.contains(&"PartialEq".to_string()));
        assert!(options.derives.contains(&"serde::Serialize".to_string()));
        assert!(options.derives.contains(&"serde::Deserialize".to_string()));
    }

    #[test]
    fn test_with_single_file() {
        let options = RustOptions::new("/output".to_string()).with_single_file(false);
        assert!(!options.single_file);
    }

    #[test]
    fn test_with_any_type() {
        let options = RustOptions::new("/output".to_string()).with_any_type("serde_json::Value");
        assert_eq!(options.any_type, "serde_json::Value");
    }

    #[test]
    fn test_with_derives() {
        let options = RustOptions::new("/output".to_string())
            .with_derives(vec!["Debug".to_string(), "Hash".to_string()]);
        assert_eq!(options.derives, vec!["Debug", "Hash"]);
    }

    #[test]
    fn test_with_additional_derives() {
        let options = RustOptions::new("/output".to_string())
            .with_additional_derives(vec!["Hash".to_string(), "Eq".to_string()]);

        // Should have defaults plus extras
        assert!(options.derives.contains(&"Debug".to_string()));
        assert!(options.derives.contains(&"Hash".to_string()));
        assert!(options.derives.contains(&"Eq".to_string()));
    }

    #[test]
    fn test_with_visibility() {
        let options =
            RustOptions::new("/output".to_string()).with_visibility(Visibility::PublicCrate);
        assert_eq!(options.visibility, Visibility::PublicCrate);

        let options2 = RustOptions::new("/output".to_string()).with_visibility(Visibility::Private);
        assert_eq!(options2.visibility, Visibility::Private);
    }

    #[test]
    fn test_with_generate_new() {
        let options = RustOptions::new("/output".to_string()).with_generate_new(false);
        assert!(!options.generate_new);
    }

    #[test]
    fn test_builder_chaining() {
        let options = RustOptions::new("/output".to_string())
            .with_single_file(false)
            .with_any_type("custom::Any")
            .with_derives(vec!["Debug".to_string()])
            .with_visibility(Visibility::PublicCrate)
            .with_generate_new(false);

        assert!(!options.single_file);
        assert_eq!(options.any_type, "custom::Any");
        assert_eq!(options.derives, vec!["Debug"]);
        assert_eq!(options.visibility, Visibility::PublicCrate);
        assert!(!options.generate_new);
    }

    #[test]
    fn test_visibility_default() {
        assert_eq!(Visibility::default(), Visibility::Public);
    }
}

// ============================================================================
// Integration tests for RustTemplateGenerator
// ============================================================================

mod template_generator_tests {
    use super::*;

    #[test]
    fn test_generates_derive_new_when_enabled() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_generate_new(true);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1])?;

        let content = fs.get_string("/output/protocols/users/mod.rs").unwrap();
        assert!(
            content.contains("derive_new::new"),
            "Should have derive_new"
        );

        Ok(())
    }

    #[test]
    fn test_no_derive_new_when_disabled() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_generate_new(false);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1])?;

        let content = fs.get_string("/output/protocols/users/mod.rs").unwrap();
        assert!(
            !content.contains("derive_new"),
            "Should not have derive_new"
        );

        Ok(())
    }

    #[test]
    fn test_custom_any_type() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;
        let d2 = deserialize_definition_file("../examples/orders.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_any_type("serde_json::Value");

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1, d2])?;

        let content = fs.get_string("/output/protocols/orders/mod.rs").unwrap();
        assert!(
            content.contains("serde_json::Value"),
            "Should use custom any type"
        );
        assert!(
            !content.contains("fluorite::Any"),
            "Should not use default any type"
        );

        Ok(())
    }

    #[test]
    fn test_extra_derives() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned())
            .with_additional_derives(vec!["Hash".to_string(), "Eq".to_string()]);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1])?;

        let content = fs.get_string("/output/protocols/users/mod.rs").unwrap();
        assert!(content.contains("Hash"), "Should have Hash derive");
        assert!(content.contains("Eq"), "Should have Eq derive");

        Ok(())
    }

    #[test]
    fn test_union_generates_tag_attribute() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;
        let d2 = deserialize_definition_file("../examples/orders.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1, d2])?;

        let content = fs.get_string("/output/protocols/orders/mod.rs").unwrap();
        assert!(
            content.contains("#[serde(tag = \"type\")]"),
            "Should have serde tag"
        );

        Ok(())
    }

    #[test]
    fn test_optional_field_skip_serializing_if() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;
        let d2 = deserialize_definition_file("../examples/orders.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1, d2])?;

        let content = fs.get_string("/output/protocols/orders/mod.rs").unwrap();
        assert!(
            content.contains("skip_serializing_if"),
            "Should have skip_serializing_if for optional fields"
        );

        Ok(())
    }

    #[test]
    fn test_generates_valid_module_structure() -> anyhow::Result<()> {
        let d1 = deserialize_definition_file("../examples/users.yml")?;
        let d2 = deserialize_definition_file("../examples/orders.yml")?;

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_single_file(false);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate(&[d1, d2])?;

        // Check mod.rs has proper structure
        let users_mod = fs.get_string("/output/protocols/users/mod.rs").unwrap();
        assert!(
            users_mod.contains("mod user;"),
            "Should declare user module"
        );
        assert!(
            users_mod.contains("mod gender;"),
            "Should declare gender module"
        );
        // Re-exports use full crate path
        assert!(
            users_mod.contains("pub use crate::"),
            "Should have pub use crate re-exports"
        );
        assert!(users_mod.contains("::user::*"), "Should re-export user");
        assert!(users_mod.contains("::gender::*"), "Should re-export gender");

        // Check individual files exist
        assert!(fs.exists("/output/protocols/users/user.rs"));
        assert!(fs.exists("/output/protocols/users/gender.rs"));

        Ok(())
    }

    #[test]
    fn test_empty_definition_list() {
        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        let result = generator.generate(&[]);

        assert!(result.is_ok());
        assert!(fs.files().is_empty());
    }
}
