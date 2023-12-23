use std::fs;

use clap::Parser;

use crate::definitions::Definition;

pub mod definitions;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    pub definition_file: String,
    #[clap(short, long)]
    pub output: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let definition = deserialize_definition_file(args.definition_file.as_str())?;
    println!("{:?}", definition);
    Ok(())
}

fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[cfg(test)]
mod test {
    use crate::deserialize_definition_file;

    #[test]
    fn test_deserialize_definition_file() -> anyhow::Result<()> {
        deserialize_definition_file("examples/simple.yml")?;
        Ok(())
    }
}
