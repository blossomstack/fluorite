//! Shared rendering of doc comments into per-line comment text.

/// Turns a doc comment into one comment line per source line, each already
/// carrying `marker` (`///` for Rust and Swift, `*` for a TypeScript block).
///
/// Templates emit these lines verbatim, so a wrapped comment survives codegen
/// intact instead of being flattened onto a single line. Returns an empty
/// vector when there is nothing to document, which templates use to decide
/// whether to emit a comment at all.
pub fn doc_lines(doc: Option<&str>, marker: &str) -> Vec<String> {
    let Some(doc) = doc else {
        return Vec::new();
    };
    let doc = doc.trim_end();
    if doc.is_empty() {
        return Vec::new();
    }

    doc.lines()
        .map(|line| {
            let line = line.trim_end();
            if line.is_empty() {
                marker.to_string()
            } else {
                format!("{marker} {line}")
            }
        })
        .collect()
}

/// `doc_lines` for a `///`-style comment.
pub fn slash_doc_lines(doc: Option<&str>) -> Vec<String> {
    doc_lines(doc, "///")
}

/// `doc_lines` for the body of a `/** ... */` block comment.
pub fn block_doc_lines(doc: Option<&str>) -> Vec<String> {
    doc_lines(doc, "*")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_doc_yields_no_lines() {
        assert!(slash_doc_lines(None).is_empty());
        assert!(slash_doc_lines(Some("")).is_empty());
        assert!(slash_doc_lines(Some("   \n  ")).is_empty());
    }

    #[test]
    fn every_line_is_kept_and_marked() {
        assert_eq!(
            slash_doc_lines(Some("first line\nsecond line\nthird line")),
            vec!["/// first line", "/// second line", "/// third line"],
        );
    }

    #[test]
    fn blank_lines_do_not_get_trailing_whitespace() {
        assert_eq!(
            block_doc_lines(Some("summary\n\ndetail")),
            vec!["* summary", "*", "* detail"],
        );
    }
}
