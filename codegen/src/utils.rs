use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::code_gen::fs::RealFileSystem;
use crate::code_gen::rust::{RustOptions, RustTemplateGenerator};

/// Collect `.fl` files from a list of paths that may include both files and directories.
///
/// For each input path:
/// - If it's a file, it is included as-is.
/// - If it's a directory, all `.fl` files under it are collected recursively.
pub fn collect_fl_files<P: AsRef<Path>>(inputs: &[P]) -> anyhow::Result<Vec<PathBuf>> {
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

/// Compile .fl definitions to Rust code with custom options.
///
/// `fl_files` may contain paths to `.fl` files or directories containing `.fl` files.
pub fn compile_with_options(options: RustOptions, fl_files: &[&str]) -> anyhow::Result<()> {
    let collected = collect_fl_files(fl_files)?;
    let paths: Vec<&Path> = collected.iter().map(|p| p.as_path()).collect();
    let schema = crate::idl::parse_to_ir(&paths)?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate_from_schema(&schema)?;

    Ok(())
}

/// Compile .fl definitions to Rust code with default options.
///
/// `fl_files` may contain paths to `.fl` files or directories containing `.fl` files.
pub fn compile(output_dir: &str, fl_files: &[&str]) -> anyhow::Result<()> {
    let options = RustOptions::new(output_dir.to_string());
    compile_with_options(options, fl_files)
}
