use clap::{Parser, ValueEnum};
use code_gen::{
    rust::{RustOptions, RustProvider},
    CodeGenerator,
};

use crate::utils::deserialize_definition_file;

mod code_gen;
mod definitions;
mod utils;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input definition files
    #[clap(short, long)]
    pub inputs: Vec<String>,
    /// Output directory
    #[clap(short, long)]
    pub output: String,
    /// Target language
    #[clap(short, long, value_enum, default_value_t=Language::Rust)]
    pub target: Language,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Language {
    Rust,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let definitions = args
        .inputs
        .iter()
        .map(|f| deserialize_definition_file(f))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let options = RustOptions::new(args.output.to_owned());
    let config = RustProvider::new(options);

    let generator = CodeGenerator::new(Box::new(config));
    generator.generate(&definitions)?;
    Ok(())
}
