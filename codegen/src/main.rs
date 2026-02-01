#![allow(dead_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::wildcard_enum_match_arm)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        deprecated
    )
)]

use std::fs;
use std::path::Path;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use code_gen::fs::RealFileSystem;
use code_gen::ir::IRSchema;
use code_gen::rust::{RustOptions, RustTemplateGenerator, Visibility};
use definitions::Definition;

mod code_gen;
mod definitions;
mod idl;
mod utils;

#[derive(Parser)]
#[command(name = "fluorite")]
#[command(about = "Code generator from YAML schema definitions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust code from YAML definitions
    Rust {
        /// Input YAML files
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Generate all types in a single mod.rs file per package
        #[arg(long, default_value = "true")]
        single_file: bool,

        /// Custom type to use for 'Any' fields
        #[arg(long, default_value = "fluorite::Any")]
        any_type: String,

        /// Custom derives (comma-separated, replaces defaults)
        #[arg(long)]
        derives: Option<String>,

        /// Additional derives to add to defaults (comma-separated)
        #[arg(long)]
        extra_derives: Option<String>,

        /// Generate derive_new::new implementation
        #[arg(long, default_value = "true")]
        generate_new: bool,

        /// Visibility for generated types (public, pub_crate, private)
        #[arg(long, default_value = "public")]
        visibility: String,
    },

    /// Generate TypeScript code from YAML definitions
    Ts {
        /// Input YAML files
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Generate all types in a single index.ts file per package
        #[arg(long, default_value = "false")]
        single_file: bool,

        /// Custom type to use for 'Any' fields (default: unknown)
        #[arg(long, default_value = "unknown")]
        any_type: String,

        /// Generate readonly properties
        #[arg(long, default_value = "false")]
        readonly: bool,

        /// Override package directory name
        #[arg(long)]
        package_name: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rust {
            inputs,
            output,
            single_file,
            any_type,
            derives,
            extra_derives,
            generate_new,
            visibility,
        } => {
            // Load inputs (YAML or .fl files)
            let (definitions, ir_schema) = load_inputs(&inputs)?;

            // Build options
            let mut options = RustOptions::new(output)
                .with_single_file(single_file)
                .with_any_type(&any_type)
                .with_generate_new(generate_new);

            // Handle derives
            if let Some(custom_derives) = derives {
                let derives: Vec<String> = custom_derives
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                options = options.with_derives(derives);
            }

            if let Some(extra) = extra_derives {
                let extra: Vec<String> = extra
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                options = options.with_additional_derives(extra);
            }

            // Handle visibility
            let vis = match visibility.to_lowercase().as_str() {
                "public" | "pub" => Visibility::Public,
                "pub_crate" | "pub(crate)" => Visibility::PublicCrate,
                "private" => Visibility::Private,
                _ => {
                    eprintln!("Unknown visibility '{}', using 'public'", visibility);
                    Visibility::Public
                }
            };
            options = options.with_visibility(vis);

            // Generate
            let fs = Arc::new(RealFileSystem::new());
            let generator = RustTemplateGenerator::new(options, fs);

            if let Some(schema) = ir_schema {
                generator.generate_from_schema(&schema)?;
            } else {
                generator.generate(&definitions)?;
            }

            println!("Code generation complete!");
        }

        Commands::Ts {
            inputs,
            output,
            single_file,
            any_type,
            readonly,
            package_name,
        } => {
            // Load inputs (YAML or .fl files)
            let (definitions, ir_schema) = load_inputs(&inputs)?;

            // Build options
            let mut options = code_gen::ts::TypeScriptOptions::new(output)
                .with_single_file(single_file)
                .with_any_type(&any_type)
                .with_readonly(readonly);

            if let Some(name) = package_name {
                options = options.with_package_name(&name);
            }

            // Generate
            let fs = Arc::new(RealFileSystem::new());
            let generator = code_gen::ts::TsTemplateGenerator::new(options, fs);

            if let Some(schema) = ir_schema {
                generator.generate_from_schema(&schema)?;
            } else {
                generator.generate(&definitions)?;
            }

            println!("TypeScript code generation complete!");
        }
    }

    Ok(())
}

/// Load definitions from input files, detecting YAML or .fl format
fn load_inputs(inputs: &[String]) -> anyhow::Result<(Vec<Definition>, Option<IRSchema>)> {
    let mut yaml_defs: Vec<Definition> = Vec::new();
    let mut fl_files: Vec<String> = Vec::new();

    for path in inputs {
        if path.ends_with(".fl") {
            fl_files.push(path.clone());
        } else {
            // Assume YAML
            let content = fs::read_to_string(path)?;
            let def: Definition = serde_yaml::from_str(&content)?;
            yaml_defs.push(def);
        }
    }

    let ir_schema = if !fl_files.is_empty() {
        let paths: Vec<&Path> = fl_files.iter().map(Path::new).collect();
        Some(idl::parse_to_ir(&paths)?)
    } else {
        None
    };

    Ok((yaml_defs, ir_schema))
}
