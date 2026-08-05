use std::sync::Arc;

use fluorite_codegen::code_gen::{
    fs::{FileSystem, FsWriter, MemoryFileSystem},
    ir::{
        IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct, IRType,
        IRTypeAlias, IRTypeAliasTarget, IRUnion, IRUnionVariant,
    },
    rust::{RustOptions, RustTemplateGenerator, Visibility},
    validation::{ValidationError, Validator},
};

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
fn test_validation_passes_for_valid_schema() {
    let schema = create_test_schema();

    let errors = Validator::new().validate(&schema);
    assert!(
        errors.is_empty(),
        "Expected no errors but got: {:?}",
        errors
    );
}

#[test]
fn test_template_generator_produces_valid_rust() {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());

    generator.generate_from_schema(&schema).unwrap();

    // Check that files were generated
    let files = fs.files();
    assert!(files.keys().any(|k| k.contains("mod.rs")));

    // Check content of generated struct
    let users_mod = fs.get_string("/output/test/users/mod.rs").unwrap();
    assert!(users_mod.contains("pub struct User"));
    assert!(users_mod.contains("first_name: String"));
}

#[test]
fn test_full_integration() -> anyhow::Result<()> {
    let schema = create_test_schema();

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
    generator.generate_from_schema(&schema)?;

    // Verify users package
    let users_content = fs
        .get_string("/output/test/users/mod.rs")
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
        .get_string("/output/test/orders/mod.rs")
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
        orders_content.contains("#[serde(tag = \"type\", content = \"value\")]"),
        "Should have adjacently tagged union"
    );
    assert!(
        orders_content.contains("pub enum Address"),
        "Should have Address union"
    );

    Ok(())
}

#[test]
fn test_multi_file_mode() -> anyhow::Result<()> {
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned()).with_single_file(false);

    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let files = fs.files();

    // Should have separate files
    assert!(
        files.contains_key("/output/test/users/user.rs"),
        "Should have user.rs"
    );
    assert!(
        files.contains_key("/output/test/users/gender.rs"),
        "Should have gender.rs"
    );
    assert!(
        files.contains_key("/output/test/users/mod.rs"),
        "Should have mod.rs"
    );

    // mod.rs should have module declarations
    let mod_content = fs.get_string("/output/test/users/mod.rs").unwrap();
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
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let orders_content = fs
        .get_string("/output/test/orders/mod.rs")
        .expect("Orders module should exist");

    // The field named "type" should be renamed to "order_type" in code
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
    let schema = create_test_schema();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate_from_schema(&schema)?;

    let orders_content = fs
        .get_string("/output/test/orders/mod.rs")
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
            doc: None,
            deny_unknown_fields: false,
        });
        assert_eq!(ir_type.name(), "User");
    }

    #[test]
    fn test_enum_name() {
        let ir_type = IRType::Enum(IREnum {
            name: "Status".to_string(),
            variants: vec!["Active".to_string(), "Inactive".to_string()],
            doc: None,
        });
        assert_eq!(ir_type.name(), "Status");
    }

    #[test]
    fn test_union_name() {
        let ir_type = IRType::Union(IRUnion {
            name: "Address".to_string(),
            tag_field: "type".to_string(),
            content_field: "value".to_string(),
            variants: vec![],
            doc: None,
        });
        assert_eq!(ir_type.name(), "Address");
    }

    #[test]
    fn test_type_alias_name() {
        let ir_type = IRType::TypeAlias(IRTypeAlias {
            name: "OrderList".to_string(),
            target: IRTypeAliasTarget::List(IRFieldType::Custom("Order".to_string())),
            doc: None,
        });
        assert_eq!(ir_type.name(), "OrderList");
    }
}

mod ir_union_variant_tests {
    use super::*;

    #[test]
    fn test_unit_variant_name() {
        let variant = IRUnionVariant::Unit {
            name: "Empty".to_string(),
            doc: None,
        };
        assert_eq!(variant.name(), "Empty");
    }

    #[test]
    fn test_newtype_variant_name() {
        let variant = IRUnionVariant::Newtype {
            name: "Info".to_string(),
            ty: IRFieldType::Custom("AddressInfo".to_string()),
            doc: None,
        };
        assert_eq!(variant.name(), "Info");
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
                    doc: None,
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
                doc: None,
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
                    doc: None,
                    deny_unknown_fields: false,
                }),
                IRType::Struct(IRStruct {
                    name: "User".to_string(),
                    fields: vec![],
                    doc: None,
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
                content_field: "value".to_string(),
                variants: vec![],
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
    fn test_unknown_type_in_union_newtype_variant() {
        let schema = create_schema(vec![(
            "test".to_string(),
            vec![IRType::Union(IRUnion {
                name: "TestUnion".to_string(),
                tag_field: "type".to_string(),
                content_field: "value".to_string(),
                variants: vec![IRUnionVariant::Newtype {
                    name: "Var".to_string(),
                    ty: IRFieldType::Custom("NonExistent".to_string()),
                    doc: None,
                }],
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
                    doc: None,
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
                    doc: None,
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
                doc: None,
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
                content_field: "value".to_string(),
                variants: vec![
                    IRUnionVariant::Unit {
                        name: "UnitA".to_string(),
                        doc: None,
                    },
                    IRUnionVariant::Unit {
                        name: "UnitB".to_string(),
                        doc: None,
                    },
                ],
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
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_generate_new(true);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/users/mod.rs").unwrap();
        assert!(
            content.contains("derive_new::new"),
            "Should have derive_new"
        );

        Ok(())
    }

    #[test]
    fn test_no_derive_new_when_disabled() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_generate_new(false);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/users/mod.rs").unwrap();
        assert!(
            !content.contains("derive_new"),
            "Should not have derive_new"
        );

        Ok(())
    }

    #[test]
    fn test_custom_any_type() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_any_type("serde_json::Value");

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/orders/mod.rs").unwrap();
        println!("Generated content:\n{}", content);
        assert!(
            content.contains("serde_json::Value"),
            "Should use custom any type. Got: {}",
            content
        );
        assert!(
            !content.contains("fluorite::Any"),
            "Should not use default any type"
        );

        Ok(())
    }

    #[test]
    fn test_extra_derives() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned())
            .with_additional_derives(vec!["Hash".to_string(), "Eq".to_string()]);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/users/mod.rs").unwrap();
        assert!(content.contains("Hash"), "Should have Hash derive");
        assert!(content.contains("Eq"), "Should have Eq derive");

        Ok(())
    }

    #[test]
    fn test_union_generates_tag_attribute() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/orders/mod.rs").unwrap();
        assert!(
            content.contains("#[serde(tag = \"type\", content = \"value\")]"),
            "Should have adjacently tagged union"
        );

        Ok(())
    }

    #[test]
    fn test_optional_field_skip_serializing_if() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        let content = fs.get_string("/output/test/orders/mod.rs").unwrap();
        assert!(
            content.contains("skip_serializing_if"),
            "Should have skip_serializing_if for optional fields"
        );

        Ok(())
    }

    #[test]
    fn test_generates_valid_module_structure() -> anyhow::Result<()> {
        let schema = create_test_schema();

        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned()).with_single_file(false);

        let generator = RustTemplateGenerator::new(options, fs.clone());
        generator.generate_from_schema(&schema)?;

        // Check mod.rs has proper structure
        let users_mod = fs.get_string("/output/test/users/mod.rs").unwrap();
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
        assert!(fs.exists("/output/test/users/user.rs"));
        assert!(fs.exists("/output/test/users/gender.rs"));

        Ok(())
    }

    #[test]
    fn test_empty_definition_list() {
        let fs = Arc::new(MemoryFileSystem::new());
        let options = RustOptions::new("/output".to_owned());

        let generator = RustTemplateGenerator::new(options, fs.clone());
        let schema = IRSchema {
            packages: std::collections::HashMap::new(),
        };
        let result = generator.generate_from_schema(&schema);

        assert!(result.is_ok());
        assert!(fs.files().is_empty());
    }
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
                variants: vec!["Male".to_string(), "Female".to_string()],
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
