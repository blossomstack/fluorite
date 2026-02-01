//! Fluorite IDL Parser
//!
//! This module provides parsing for the Fluorite Interface Definition Language (.fl files).
//!
//! # Architecture
//!
//! - `lexer`: Tokenizes source code using `logos`
//! - `parser`: Parses tokens into AST using `chumsky`
//! - `ast`: AST type definitions
//! - `ast_to_ir`: Converts AST to the Intermediate Representation (IR) for code generation
//!
//! # Example
//!
//! ```rust
//! use fluorite_codegen::idl::{parse_file, parse_files, parse_string};
//!
//! // Parse a single file
//! let source = r#"
//!     package users;
//!     struct User {
//!         name: String,
//!         age: u32,
//!     }
//! "#;
//! let ast = parse_string(source).unwrap();
//! ```

pub mod ast;
pub mod ast_to_ir;
pub mod lexer;
pub mod parser;

use anyhow::{anyhow, Result};
use std::path::Path;

use crate::code_gen::ir::IRSchema;

use self::ast::AstFile;
use self::ast_to_ir::AstToIrConverter;

/// Parse a single .fl file from source string
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_string;
///
/// let source = r#"
///     package users;
///     struct User {
///         name: String,
///     }
/// "#;
/// let ast = parse_string(source).unwrap();
/// ```
pub fn parse_string(source: &str) -> Result<AstFile> {
    parser::parse_file(source).map_err(|errors| anyhow!("Parse errors: {:?}", errors))
}

/// Parse a single .fl file from disk
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_file;
/// use std::path::Path;
///
/// // let ast = parse_file(Path::new("examples/users.fl")).unwrap();
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<AstFile> {
    let source = std::fs::read_to_string(path)?;
    parse_string(&source)
}

/// Parse multiple .fl files from disk
///
/// Returns a vector of AST files, one for each successfully parsed file.
/// Files that fail to parse are skipped and errors are logged.
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_files;
/// use std::path::Path;
///
/// // let asts = parse_files(&[
/// //     Path::new("examples/users.fl"),
/// //     Path::new("examples/orders.fl"),
/// // ]).unwrap();
/// ```
pub fn parse_files<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<AstFile>> {
    let mut files = Vec::new();
    for path in paths {
        match parse_file(path) {
            Ok(ast) => files.push(ast),
            Err(e) => {
                eprintln!("Warning: Failed to parse {:?}: {}", path.as_ref(), e);
            }
        }
    }
    Ok(files)
}

/// Parse .fl files and convert to IR schema for code generation
///
/// This is the main entry point for using the IDL parser with the code generator.
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_to_ir;
/// use std::path::Path;
///
/// // let schema = parse_to_ir(&[
/// //     Path::new("examples/users.fl"),
/// //     Path::new("examples/orders.fl"),
/// // ]).unwrap();
/// ```
pub fn parse_to_ir<P: AsRef<Path>>(paths: &[P]) -> Result<IRSchema> {
    let ast_files = parse_files(paths)?;
    let converter = AstToIrConverter::new();
    converter.convert_files(&ast_files)
}

/// Parse a single .fl source string and convert to IR schema
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_string_to_ir;
///
/// let source = r#"
///     package users;
///     struct User {
///         name: String,
///     }
/// "#;
/// // let schema = parse_string_to_ir(source).unwrap();
/// ```
pub fn parse_string_to_ir(source: &str) -> Result<IRSchema> {
    let ast = parse_string(source)?;
    let converter = AstToIrConverter::new();
    converter.convert_files(&[ast])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let source = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let result = parse_string(source);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.package.value, "test");
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_parse_string_to_ir() {
        let source = r#"
            package test;
            struct User {
                name: String,
                age: u32,
            }
        "#;
        let result = parse_string_to_ir(source);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert!(schema.packages.contains_key("test"));
    }
}
