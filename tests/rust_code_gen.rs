use std::fs;

use fluorite::{
    code_gen::{
        rust::{RustOptions, RustProvider},
        CodeGenerator,
    },
    definitions::Definition,
};

pub(crate) fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[test]
fn test_rust_code_gen() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("examples/users.yml")?;
    let d2 = deserialize_definition_file("examples/orders.yml")?;
    let output_dir = "/tmp/test1";
    let options = RustOptions::new(output_dir.to_owned());
    let config = RustProvider::new(options);

    let generator = CodeGenerator::new(Box::new(config));
    generator.generate(&vec![d1, d2])?;
    Ok(())
}
