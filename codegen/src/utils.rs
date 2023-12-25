use std::fs;

use crate::{
    code_gen::{
        rust::{RustOptions, RustProvider},
        CodeGenerator,
    },
    definitions::Definition,
};

pub fn compile(inputs: &[&str], output: &str) -> anyhow::Result<()> {
    let definitions = inputs
        .iter()
        .map(|s| deserialize_definition_file(s))
        .collect::<anyhow::Result<Vec<Definition>>>()?;
    let options = RustOptions::new(output.to_owned());
    let config = RustProvider::new(options);

    let generator = CodeGenerator::new(Box::new(config));
    generator.generate(&definitions)?;
    Ok(())
}

pub(crate) fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[cfg(test)]
mod test {
    use super::deserialize_definition_file;

    #[test]
    fn test_deserialize_definition_file() -> anyhow::Result<()> {
        deserialize_definition_file("../examples/users.yml")?;
        Ok(())
    }
}
