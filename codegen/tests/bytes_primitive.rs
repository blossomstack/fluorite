//! Every backend must agree on what a `Bytes` field is on the wire: a base64
//! string.
//!
//! `Bytes` was not untested before this — each backend had its own assertion
//! (`go -> "string"`, `swift -> "Data"`, `rust -> "Vec<u8>"`). What no test did
//! was compare them, or serialize a single value. So every generator passed
//! against its own expectation while Rust emitted a JSON array of numbers and
//! the other three emitted a base64 string, and the assertions locked that
//! disagreement in rather than catching it.
//!
//! This test compares the four in one place. Adding a backend that forgets
//! `Bytes` fails here, rather than in somebody's cross-language integration.

use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::MemoryFileSystem,
        go::{GoOptions, GoTemplateGenerator},
        rust::{RustOptions, RustTemplateGenerator},
        swift::{SwiftOptions, SwiftTemplateGenerator},
        ts::{TsTemplateGenerator, TypeScriptOptions},
    },
    idl::parse_string_to_ir,
};

const SCHEMA: &str = r#"
package blobs;

/// An artifact with its bytes inline.
struct Blob {
    data: Bytes,
    maybe: Option<Bytes>,
    many: Vec<Bytes>,
}
"#;

fn schema() -> fluorite_codegen::code_gen::ir::IRSchema {
    parse_string_to_ir(SCHEMA).expect("the Bytes schema should parse")
}

#[test]
fn rust_emits_the_base64_newtype_not_a_byte_vec() {
    let fs = Arc::new(MemoryFileSystem::new());
    let generator = RustTemplateGenerator::new(RustOptions::new("/out".to_owned()), fs.clone());
    generator.generate_from_schema(&schema()).unwrap();

    let out = fs.get_string("/out/blobs/mod.rs").unwrap();

    assert!(
        out.contains("fluorite::Bytes"),
        "a Bytes field must map to the base64 newtype; got:\n{out}"
    );
    assert!(
        !out.contains("Vec<u8>"),
        "`Vec<u8>` serializes as a JSON array of numbers, which no other \
         backend reads. Output:\n{out}"
    );
    // The newtype has to compose, since these shapes get no serde attribute.
    assert!(out.contains("Option<fluorite::Bytes>"));
    assert!(out.contains("Vec<fluorite::Bytes>"));
}

#[test]
fn typescript_emits_a_string() {
    let fs = Arc::new(MemoryFileSystem::new());
    let generator = TsTemplateGenerator::new(TypeScriptOptions::new("/out".to_owned()), fs.clone());
    generator.generate_from_schema(&schema()).unwrap();

    let files = fs.files();
    let out: String = files
        .iter()
        .filter(|(k, _)| k.contains("blob"))
        .map(|(_, v)| String::from_utf8_lossy(v).into_owned())
        .collect();

    assert!(
        out.contains("data: string"),
        "TypeScript reads a Bytes field as a base64 string; got:\n{out}"
    );
}

#[test]
fn go_emits_a_string() {
    let fs = Arc::new(MemoryFileSystem::new());
    let generator = GoTemplateGenerator::new(GoOptions::new("/out".to_owned()), fs.clone());
    generator.generate_from_schema(&schema()).unwrap();

    let files = fs.files();
    let out: String = files
        .values()
        .map(|v| String::from_utf8_lossy(v).into_owned())
        .collect();

    // gofmt aligns struct fields into columns, so match on the field line
    // rather than on single-spaced text.
    let data_line = out
        .lines()
        .find(|l| l.contains(r#"json:"data""#))
        .unwrap_or_else(|| panic!("no `data` field in the Go output:\n{out}"));

    assert!(
        data_line.split_whitespace().any(|t| t == "string"),
        "Go reads a Bytes field as a base64 string; got: {data_line}"
    );
}

#[test]
fn swift_emits_data() {
    let fs = Arc::new(MemoryFileSystem::new());
    let generator = SwiftTemplateGenerator::new(SwiftOptions::new("/out".to_owned()), fs.clone());
    generator.generate_from_schema(&schema()).unwrap();

    let files = fs.files();
    let out: String = files
        .values()
        .map(|v| String::from_utf8_lossy(v).into_owned())
        .collect();

    // Codable encodes `Data` as a base64 string by default, which is what
    // makes it agree with the other three.
    assert!(
        out.contains(": Data"),
        "Swift reads a Bytes field as Data; got:\n{out}"
    );
}
