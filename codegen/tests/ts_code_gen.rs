use std::fs;
use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::MemoryFileSystem,
        ts::{TsTemplateGenerator, TypeScriptOptions},
    },
    definitions::Definition,
};

fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

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
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/user.ts").unwrap();
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/gender.ts").unwrap();
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let content = fs
        .get_string("/output/protocols/orders/address.ts")
        .unwrap();
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let order_list_content = fs
        .get_string("/output/protocols/orders/orderList.ts")
        .unwrap();
    assert!(
        order_list_content.contains("export type OrderList = Order[]"),
        "Should have list alias. Got: {}",
        order_list_content
    );

    let order_map_content = fs
        .get_string("/output/protocols/orders/orderMap.ts")
        .unwrap();
    assert!(
        order_map_content.contains("export type OrderMap = Record<string, Order>"),
        "Should have map alias. Got: {}",
        order_map_content
    );

    Ok(())
}

#[test]
fn test_ts_single_file_mode() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_single_file(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    // Should only have index.ts
    let files = fs.files();
    assert!(
        files.contains_key("/output/protocols/users/index.ts"),
        "Should have index.ts"
    );
    assert!(
        !files.contains_key("/output/protocols/users/user.ts"),
        "Should NOT have user.ts"
    );

    let content = fs.get_string("/output/protocols/users/index.ts").unwrap();
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_readonly(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/user.ts").unwrap();
    assert!(
        content.contains("readonly firstName: string"),
        "Should have readonly fields. Got: {}",
        content
    );

    Ok(())
}

#[test]
fn test_ts_any_type_option() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_any_type("any");
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    // PostCode has an 'instruction' field of type Any, which is inlined into Address union
    let content = fs
        .get_string("/output/protocols/orders/address.ts")
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let content = fs.get_string("/output/protocols/orders/order.ts").unwrap();
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
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/index.ts").unwrap();
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
    let result = generator.generate(&[]);

    assert!(result.is_ok());
    assert!(fs.files().is_empty());
}
