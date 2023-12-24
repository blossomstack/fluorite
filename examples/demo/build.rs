use fluorite::{
    code_gen::{
        rust::{RustOptions, RustProvider},
        CodeGenerator,
    },
    definitions::Definition,
};
use serde_yaml;
use std::fs;

fn main() -> anyhow::Result<()> {
    let d = deserialize_definition_file("./fluorite/demo.yaml")?;
    let output_dir = "./src";
    let options = RustOptions::new(output_dir.to_owned());
    let config = RustProvider::new(options);

    let generator = CodeGenerator::new(Box::new(config));
    generator.generate(&vec![d])?;
    Ok(())
}

fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}
