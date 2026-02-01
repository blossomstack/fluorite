use std::fs;
use std::sync::Arc;

use crate::code_gen::fs::RealFileSystem;
use crate::code_gen::rust::{RustOptions, RustTemplateGenerator};
use crate::definitions::Definition;

/// Compile YAML definitions to Rust code with custom options
pub fn compile_with_options(options: RustOptions, yaml_files: &[&str]) -> anyhow::Result<()> {
    let definitions: Vec<Definition> = yaml_files
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path)?;
            let def: Definition = serde_yaml::from_str(&content)?;
            Ok(def)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate(&definitions)?;

    Ok(())
}

/// Compile YAML definitions to Rust code with default options
pub fn compile(output_dir: &str, yaml_files: &[&str]) -> anyhow::Result<()> {
    let options = RustOptions::new(output_dir.to_string());
    compile_with_options(options, yaml_files)
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
