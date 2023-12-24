use std::fs;

use fluorite::{code_gen::rust::RustCodeGen, definitions::Definition};

pub(crate) fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[test]
fn test_rust_code_gen() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("examples/users.yml")?;
    let d2 = deserialize_definition_file("examples/orders.yml")?;

    RustCodeGen::code_gen(vec![d1, d2])?;
    Ok(())
}
