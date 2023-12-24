use std::fs;

use crate::definitions::Definition;

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
        deserialize_definition_file("examples/users.yml")?;
        Ok(())
    }
}
