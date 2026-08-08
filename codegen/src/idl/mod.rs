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
    parser::parse_file(source).map_err(|errors| anyhow!(render_parse_errors(None, source, &errors)))
}

/// Convert a byte offset into 1-based line and column numbers.
fn line_col(source: &str, offset: usize) -> (usize, usize) {
    // Clamp to a char boundary so a span into the middle of a multi-byte
    // character can't panic the slicing below.
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }

    let before = &source[..offset];
    let line = before.matches('\n').count() + 1;
    let col = match before.rfind('\n') {
        Some(nl) => offset - nl,
        None => offset + 1,
    };
    (line, col)
}

/// Render parse errors with real source positions.
///
/// Spans are byte offsets into `source`, so they are reported as `line:column`
/// rather than raw offsets — a raw `19..20` reads like something a caller can
/// locate but tells them nothing without counting bytes by hand.
fn render_parse_errors(path: Option<&Path>, source: &str, errors: &[parser::ParseError]) -> String {
    use chumsky::error::{RichPattern, RichReason};
    use std::fmt::Write;

    let origin = match path {
        Some(p) => p.display().to_string(),
        None => "<input>".to_owned(),
    };

    let mut out = format!("failed to parse {origin}");
    for error in errors {
        let (line, col) = line_col(source, error.span().start);
        let _ = write!(out, "\n  {origin}:{line}:{col}: ");

        match error.reason() {
            RichReason::Custom(message) => {
                let _ = write!(out, "{message}");
            }
            RichReason::ExpectedFound { found, .. } => {
                match found {
                    Some(token) => {
                        let _ = write!(out, "unexpected {:?}", &**token);
                    }
                    None => {
                        let _ = write!(out, "unexpected end of input");
                    }
                }

                // The expected set can run to every type keyword in the
                // language, which buries the useful part. Show a few.
                let mut expected: Vec<String> = error
                    .expected()
                    .map(|pattern| match pattern {
                        RichPattern::Token(token) => format!("{:?}", &**token),
                        RichPattern::Label(label) => label.to_string(),
                        RichPattern::Identifier(name) => name.clone(),
                        RichPattern::EndOfInput => "end of input".to_owned(),
                        RichPattern::Any => "any token".to_owned(),
                        RichPattern::SomethingElse => "something else".to_owned(),
                        // `RichPattern` is `#[non_exhaustive]`.
                        _ => "something else".to_owned(),
                    })
                    .collect();
                expected.sort();
                if !expected.is_empty() {
                    let shown = expected.len().min(5);
                    let _ = write!(out, ", expected {}", expected[..shown].join(" | "));
                    if expected.len() > shown {
                        let _ = write!(out, " (and {} more)", expected.len() - shown);
                    }
                }
            }
        }
    }
    out
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
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)?;
    parser::parse_file(&source)
        .map_err(|errors| anyhow!(render_parse_errors(Some(path), &source, &errors)))
}

/// Parse multiple .fl files from disk
///
/// Returns a vector of AST files, one per input. Any file that fails to parse
/// fails the whole call: a skipped file silently drops its package from the
/// output, which downstream reads as a missing type rather than a bad schema.
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
    paths.iter().map(parse_file).collect()
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

/// Parse multiple .fl source strings and convert to IR schema
///
/// This is useful for testing multi-package scenarios where types
/// from one package reference types from another.
///
/// # Example
///
/// ```rust
/// use fluorite_codegen::idl::parse_strings_to_ir;
///
/// let common = r#"
///     package common;
///     struct Address { city: String }
/// "#;
/// let users = r#"
///     package users;
///     use common.Address;
///     struct User { address: Address }
/// "#;
/// // let schema = parse_strings_to_ir(&[common, users]).unwrap();
/// ```
pub fn parse_strings_to_ir(sources: &[&str]) -> Result<IRSchema> {
    let asts: Result<Vec<_>> = sources.iter().map(|s| parse_string(s)).collect();
    let converter = AstToIrConverter::new();
    converter.convert_files(&asts?)
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
        assert_eq!(ast.package.len(), 1);
        assert_eq!(ast.package[0].value, "test");
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
