use std::path::Path;
use std::sync::Arc;

use crate::code_gen::fs::RealFileSystem;
use crate::code_gen::rust::{RustOptions, RustTemplateGenerator};

/// Compile .fl definitions to Rust code with custom options
pub fn compile_with_options(options: RustOptions, fl_files: &[&str]) -> anyhow::Result<()> {
    let paths: Vec<&Path> = fl_files.iter().map(Path::new).collect();
    let schema = crate::idl::parse_to_ir(&paths)?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate_from_schema(&schema)?;

    Ok(())
}

/// Compile .fl definitions to Rust code with default options
pub fn compile(output_dir: &str, fl_files: &[&str]) -> anyhow::Result<()> {
    let options = RustOptions::new(output_dir.to_string());
    compile_with_options(options, fl_files)
}
