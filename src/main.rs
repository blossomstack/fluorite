use clap::Parser;

use crate::utils::deserialize_definition_file;

mod definitions;
mod utils;

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
