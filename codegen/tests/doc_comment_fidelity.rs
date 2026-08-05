//! Doc comments must reach generated code verbatim, and a malformed input must
//! fail loudly rather than silently dropping a package.
//!
//! Covers blossomstack/fluorite#12 (doc comments were HTML-escaped in the
//! TypeScript and Rust output), #10 (doc comments were rejected on union
//! variants) and #9 (a parse error was a warning and exited 0).

use std::path::PathBuf;
use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::MemoryFileSystem,
        rust::{RustOptions, RustTemplateGenerator},
        swift::{SwiftOptions, SwiftTemplateGenerator},
        ts::{TsTemplateGenerator, TypeScriptOptions},
    },
    idl::{parse_files, parse_string_to_ir},
};

/// Every character a doc comment might plausibly contain that an HTML escaper
/// would mangle. None are special in Rust, TypeScript or Swift comments, so all
/// must survive verbatim.
const PUNCTUATION: &str = r#""quoted", it's, <angled>, & ampersand"#;

/// Entities an HTML escaper would emit for the characters above. Both the hex
/// and decimal spellings are listed: askama switched from one to the other, and
/// neither belongs in generated code.
const ENTITIES: &[&str] = &[
    "&#34;", "&#39;", "&#60;", "&#62;", "&#38;", "&#x27;", "&quot;", "&apos;", "&lt;", "&gt;",
    "&amp;",
];

fn source_with_docs() -> String {
    format!(
        r#"
        package fidelity;

        /// Struct doc: {PUNCTUATION}
        struct Status {{
            /// Field doc: {PUNCTUATION}
            model_id: String,
        }}

        /// Enum doc: {PUNCTUATION}
        enum Kind {{
            /// Enum variant doc: {PUNCTUATION}
            First,
            Second,
        }}

        /// Union doc: {PUNCTUATION}
        #[type_tag = "kind"]
        union Outcome {{
            /// Union variant doc: {PUNCTUATION}
            Ran(Status),
            Stopped,
        }}
        "#
    )
}

/// Generate all three languages from one source, returning `(label:path, content)`.
fn generated_files(source: &str) -> anyhow::Result<Vec<(String, String)>> {
    let schema = parse_string_to_ir(source)?;
    let mut out = Vec::new();

    let ts = Arc::new(MemoryFileSystem::new());
    TsTemplateGenerator::new(TypeScriptOptions::new("/out".to_owned()), ts.clone())
        .generate_from_schema(&schema)?;

    let rust = Arc::new(MemoryFileSystem::new());
    RustTemplateGenerator::new(RustOptions::new("/out".to_owned()), rust.clone())
        .generate_from_schema(&schema)?;

    let swift = Arc::new(MemoryFileSystem::new());
    SwiftTemplateGenerator::new(SwiftOptions::new("/out".to_owned()), swift.clone())
        .generate_from_schema(&schema)?;

    for (label, fs) in [("ts", ts), ("rust", rust), ("swift", swift)] {
        for (path, bytes) in fs.files() {
            out.push((
                format!("{label}:{path}"),
                String::from_utf8_lossy(&bytes).into_owned(),
            ));
        }
    }

    Ok(out)
}

/// Create a unique scratch directory without pulling in a dev-dependency.
fn scratch_dir(name: &str) -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("fluorite-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// #12 — generated code is not HTML, so no entity should ever appear in it.
#[test]
fn doc_comments_are_not_html_escaped() -> anyhow::Result<()> {
    for (path, content) in generated_files(&source_with_docs())? {
        for entity in ENTITIES {
            assert!(
                !content.contains(entity),
                "{path} contains HTML entity {entity}; doc comments must be verbatim.\n{content}"
            );
        }
    }
    Ok(())
}

/// The positive half of #12: the real characters must be present, so the test
/// above cannot pass by the doc comment going missing entirely.
#[test]
fn doc_comment_punctuation_survives_verbatim() -> anyhow::Result<()> {
    let files = generated_files(&source_with_docs())?;

    for prefix in ["ts:", "rust:", "swift:"] {
        let found = files
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .any(|(_, content)| content.contains(PUNCTUATION));
        assert!(
            found,
            "no {prefix} file carried the doc punctuation verbatim"
        );
    }

    Ok(())
}

/// #10 — a doc comment on a union variant used to be a parse error, even though
/// it is accepted on enum variants, struct fields and the union itself.
#[test]
fn union_variants_accept_doc_comments() -> anyhow::Result<()> {
    let schema = parse_string_to_ir(&source_with_docs())?;
    assert!(
        !schema.packages.is_empty(),
        "schema should contain the fidelity package"
    );
    Ok(())
}

/// #10 — and the doc must reach the generated Rust and TypeScript, not merely
/// parse and then get dropped on the floor.
#[test]
fn union_variant_docs_reach_generated_code() -> anyhow::Result<()> {
    let files = generated_files(&source_with_docs())?;

    for prefix in ["ts:", "rust:", "swift:"] {
        let found = files
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .any(|(_, content)| content.contains("Union variant doc:"));
        assert!(
            found,
            "{prefix} output dropped the union variant doc comment"
        );
    }

    Ok(())
}

/// #9 — a file that fails to parse must surface as an error. It used to be
/// logged as a warning while the caller carried on and reported success, so a
/// malformed schema silently produced no output for its package.
#[test]
fn parse_failure_is_an_error_not_a_warning() -> anyhow::Result<()> {
    let dir = scratch_dir("parse-fail")?;

    let bad = dir.join("bad.fl");
    std::fs::write(&bad, "package p;\nstruct A { x: String\n")?;
    let good = dir.join("good.fl");
    std::fs::write(&good, "package q;\nstruct B { y: String }\n")?;

    let result = parse_files(&[bad.as_path(), good.as_path()]);

    assert!(
        result.is_err(),
        "a malformed .fl must fail the parse, not be skipped with a warning"
    );

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

/// #9 — the good-input path must keep working, so the fix above cannot pass by
/// simply rejecting everything.
#[test]
fn parse_succeeds_when_every_input_is_valid() -> anyhow::Result<()> {
    let dir = scratch_dir("parse-ok")?;

    let a = dir.join("a.fl");
    std::fs::write(&a, "package p;\nstruct A { x: String }\n")?;
    let b = dir.join("b.fl");
    std::fs::write(&b, "package q;\nstruct B { y: String }\n")?;

    let files = parse_files(&[a.as_path(), b.as_path()])?;
    assert_eq!(files.len(), 2);

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
