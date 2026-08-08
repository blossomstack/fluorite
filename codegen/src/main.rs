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

use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};

use code_gen::fs::RealFileSystem;
use code_gen::rust::{RustOptions, RustTemplateGenerator, Visibility};

mod code_gen;
mod idl;

#[derive(Parser)]
#[command(name = "fluorite")]
#[command(about = "Code generator from Fluorite IDL (.fl) schema definitions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Delete a generated output tree
    ///
    /// Generation overwrites files but never removes them, so a renamed or
    /// moved type leaves its old file behind. Run this before a generate to
    /// make the output tree a pure function of the schema.
    Clean {
        /// Output directory to delete, recursively
        #[arg(short, long)]
        output: String,
    },

    /// Generate Rust code from .fl definitions
    Rust {
        /// Input .fl files or directories containing .fl files
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

    /// Generate TypeScript code from .fl definitions
    Ts {
        /// Input .fl files or directories containing .fl files
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

    /// Generate Swift code from .fl definitions
    Swift {
        /// Input .fl files or directories containing .fl files
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Generate all types in a single file per package
        #[arg(long, default_value = "false")]
        single_file: bool,

        /// Custom type to use for 'Any' fields (default: AnyCodable)
        #[arg(long, default_value = "AnyCodable")]
        any_type: String,

        /// Visibility for generated types (public, internal, package)
        #[arg(long, default_value = "public")]
        visibility: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clean { output } => {
            let fs = RealFileSystem::new();
            code_gen::clean::clean_output_dir(&fs, &output)?;
            println!("Cleaned {output}");
        }

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
            // Load inputs from .fl files
            let schema = load_fl_inputs(&inputs)?;

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
            generator.generate_from_schema(&schema)?;

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
            // Load inputs from .fl files
            let schema = load_fl_inputs(&inputs)?;

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
            generator.generate_from_schema(&schema)?;

            println!("TypeScript code generation complete!");
        }

        Commands::Swift {
            inputs,
            output,
            single_file,
            any_type,
            visibility,
        } => {
            // Load inputs from .fl files
            let schema = load_fl_inputs(&inputs)?;

            // Build options
            let mut options = code_gen::swift::SwiftOptions::new(output)
                .with_single_file(single_file)
                .with_any_type(&any_type);

            // Handle visibility
            let vis = match visibility.to_lowercase().as_str() {
                "public" | "pub" => code_gen::swift::SwiftVisibility::Public,
                "internal" => code_gen::swift::SwiftVisibility::Internal,
                "package" => code_gen::swift::SwiftVisibility::Package,
                _ => {
                    eprintln!("Unknown visibility '{}', using 'public'", visibility);
                    code_gen::swift::SwiftVisibility::Public
                }
            };
            options = options.with_visibility(vis);

            // Generate
            let fs = Arc::new(RealFileSystem::new());
            let generator = code_gen::swift::SwiftTemplateGenerator::new(options, fs);
            generator.generate_from_schema(&schema)?;

            println!("Swift code generation complete!");
        }
    }

    Ok(())
}

/// Load IR schema from .fl input files or directories containing .fl files
fn load_fl_inputs(inputs: &[String]) -> anyhow::Result<code_gen::ir::IRSchema> {
    let collected = collect_fl_files(inputs)?;
    if collected.is_empty() {
        anyhow::bail!("No .fl files found in the provided inputs");
    }
    let paths: Vec<&Path> = collected.iter().map(|p| p.as_path()).collect();
    idl::parse_to_ir(&paths)
}

/// Collect `.fl` files from a list of paths that may include both files and directories.
fn collect_fl_files<P: AsRef<Path>>(inputs: &[P]) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for input in inputs {
        let path = input.as_ref();
        if path.is_dir() {
            collect_fl_files_recursive(path, &mut files)?;
        } else {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn collect_fl_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_fl_files_recursive(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("fl") {
            files.push(path);
        }
    }
    Ok(())
}
