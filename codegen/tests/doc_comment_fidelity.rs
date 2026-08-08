//! Doc comments must reach generated code verbatim, and a malformed input must
//! fail loudly rather than silently dropping a package.
//!
//! Covers blossomstack/fluorite#12 (doc comments were HTML-escaped in the
//! TypeScript and Rust output), #10 (doc comments were rejected on union
//! variants), #9 (a parse error was a warning and exited 0) and #15 (only the
//! first line of a wrapped comment survived, and Rust enums, unions and type
//! aliases emitted no documentation at all).

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

/// Strip `\r` so assertions about comment layout read the same everywhere.
/// Windows checks the templates out with CRLF, so askama emits CRLF there.
fn lf(content: &[u8]) -> String {
    String::from_utf8_lossy(content).replace('\r', "")
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
            out.push((format!("{label}:{path}"), lf(&bytes)));
        }
    }

    Ok(out)
}

/// A schema where every doc comment wraps over three source lines. The middle
/// line is the one that used to vanish, so each is distinctive enough to be
/// searched for on its own.
const MULTI_LINE_SOURCE: &str = r#"
    package wrapped;

    /// Struct line one.
    /// Struct line two.
    /// Struct line three.
    struct Step {
        /// Field line one.
        /// Field line two.
        index: u32,
    }

    /// Enum line one.
    /// Enum line two.
    enum Phase {
        /// Enum variant line one.
        /// Enum variant line two.
        Early,
        Late,
    }

    /// Union line one.
    /// Union line two.
    #[type_tag = "kind"]
    union Event {
        /// Union variant line one.
        /// Union variant line two.
        Ran(Step),
        Stopped,
    }

    /// Alias line one.
    /// Alias line two.
    type Steps = Vec<Step>;

    /// Map alias line one.
    /// Map alias line two.
    type StepsByName = Map<String, Step>;
"#;

/// Asserts that every one of `fragments` appears in some file of each language.
fn assert_present_in_every_language(files: &[(String, String)], fragments: &[&str]) {
    for prefix in ["ts:", "rust:", "swift:"] {
        for fragment in fragments {
            let found = files
                .iter()
                .filter(|(path, _)| path.starts_with(prefix))
                .any(|(_, content)| content.contains(fragment));
            assert!(found, "no {prefix} file contained {fragment:?}");
        }
    }
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

/// #15 — the lexer emits one token per `///` line and the parser kept only the
/// first, so a wrapped comment was cut wherever the author happened to wrap it.
#[test]
fn every_line_of_a_wrapped_doc_comment_survives() -> anyhow::Result<()> {
    let files = generated_files(MULTI_LINE_SOURCE)?;

    assert_present_in_every_language(
        &files,
        &[
            "Struct line one.",
            "Struct line two.",
            "Struct line three.",
            "Field line one.",
            "Field line two.",
            "Enum line one.",
            "Enum line two.",
            "Enum variant line one.",
            "Enum variant line two.",
            "Union line one.",
            "Union line two.",
            "Union variant line one.",
            "Union variant line two.",
            "Alias line one.",
            "Alias line two.",
            "Map alias line one.",
            "Map alias line two.",
        ],
    );

    Ok(())
}

/// The lines must stay on separate comment lines, not be run together — the
/// generated comment should read exactly as the schema author wrote it.
#[test]
fn wrapped_doc_comment_lines_stay_separate() -> anyhow::Result<()> {
    let files = generated_files(MULTI_LINE_SOURCE)?;

    let rust = files
        .iter()
        .filter(|(path, _)| path.starts_with("rust:"))
        .map(|(_, content)| content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(rust
        .contains("/// Struct line one.\n/// Struct line two.\n/// Struct line three.\n#[derive("));
    assert!(rust.contains("    /// Field line one.\n    /// Field line two.\n    pub index: u32,"));

    let ts = files
        .iter()
        .filter(|(path, _)| path.starts_with("ts:"))
        .map(|(_, content)| content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(ts.contains(
        " * Struct line one.\n * Struct line two.\n * Struct line three.\n */\nexport interface Step"
    ));

    Ok(())
}

/// #15 — the Rust generator never passed a doc through for enums, unions or
/// type aliases, so their prose was dropped even before the truncation bug.
#[test]
fn rust_documents_enums_unions_and_aliases() -> anyhow::Result<()> {
    let schema = parse_string_to_ir(MULTI_LINE_SOURCE)?;
    let fs = Arc::new(MemoryFileSystem::new());
    RustTemplateGenerator::new(RustOptions::new("/out".to_owned()), fs.clone())
        .generate_from_schema(&schema)?;

    let rust = fs
        .files()
        .values()
        .map(|bytes| lf(bytes))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rust.contains("/// Enum line one.\n/// Enum line two.\n#[derive("));
    assert!(rust.contains("/// Union line one.\n/// Union line two.\n#[derive("));
    assert!(rust.contains("/// Alias line one.\n/// Alias line two.\npub type Steps"));

    Ok(())
}

/// A blank line inside a doc comment must not leave trailing whitespace behind,
/// which would trip linters on the generated code.
#[test]
fn a_blank_doc_line_carries_no_trailing_whitespace() -> anyhow::Result<()> {
    let source = r#"
        package spaced;

        /// Summary.
        ///
        /// Detail.
        struct Documented {
            name: String,
        }
    "#;

    let files = generated_files(source)?;
    assert_present_in_every_language(&files, &["Summary.", "Detail."]);

    for (path, content) in &files {
        for line in content.lines() {
            assert_eq!(
                line,
                line.trim_end(),
                "{path} has a line with trailing whitespace: {line:?}"
            );
        }
    }

    Ok(())
}

/// A field can be deprecated without being documented. The TypeScript template
/// opens its JSDoc block on `!doc.is_empty() || deprecated`, so an empty doc
/// must not take `@deprecated` down with it.
#[test]
fn deprecated_without_a_doc_still_gets_a_jsdoc_block() -> anyhow::Result<()> {
    let source = r#"
        package dep;

        struct S {
            #[deprecated]
            bare: String,
            /// Has a doc.
            #[deprecated]
            documented: String,
        }
    "#;

    let files = generated_files(source)?;
    let ts = files
        .iter()
        .filter(|(path, _)| path.starts_with("ts:"))
        .map(|(_, content)| content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(ts.contains("  /**\n   * @deprecated\n   */\n  bare: string;"));
    assert!(ts.contains("  /**\n   * Has a doc.\n   * @deprecated\n   */\n  documented: string;"));

    Ok(())
}

/// The templates loop over a doc's lines rather than testing a string for
/// emptiness, so an undocumented schema must produce no comment markers and no
/// stray blank lines where a comment would have gone.
#[test]
fn an_undocumented_schema_gets_no_comments() -> anyhow::Result<()> {
    let source = r#"
        package bare;

        struct S {
            a: u32,
            b: String,
        }

        enum E {
            One,
            Two,
        }

        #[type_tag = "t"]
        union U {
            Ran(S),
            Stopped,
        }

        type L = Vec<S>;
    "#;

    for (path, content) in generated_files(source)? {
        // The Swift barrel file is prose by design; every other file is code.
        if path.ends_with("bare.swift") {
            continue;
        }
        // `/**` covers JSDoc; a lone ` * ` would also match `export * from`.
        for marker in ["///", "/**"] {
            assert!(
                !content.contains(marker),
                "{path} emitted the comment marker {marker:?} for an undocumented \
                 schema:\n{content}"
            );
        }
        assert!(
            !content.contains("\n\n\n"),
            "{path} left a blank run where a doc comment would have gone:\n{content}"
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
