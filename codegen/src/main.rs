use clap::{Parser, Subcommand};
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
    /// Sub commands for different target languages
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Rust {
        /// Input definition files
        #[clap(short, long)]
        inputs: Vec<String>,
        /// Output directory
        #[clap(short, long)]
        output: String,

        /// Output codes to a single mod file for each package
        #[clap(short, long, default_value_t = true)]
        single_file: bool,
    },
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Rust {
            inputs,
            output,
            single_file,
        } => {
            let definitions = inputs
                .iter()
                .map(|f| deserialize_definition_file(f))
                .collect::<anyhow::Result<Vec<_>>>()?;

            let options = RustOptions::new(output.to_owned()).with_single_file(single_file);
            let config = RustProvider::new(options);

            let generator = CodeGenerator::new(Box::new(config));
            generator.generate(&definitions)?;
        }
    }
    Ok(())
}
