//! Go identifier naming and gofmt-compatible column alignment.

/// Segments uppercased wholesale when converting a name to Go.
///
/// Changing this list renames exported fields, which is a breaking change for
/// any consumer of generated code. Add to it deliberately.
const GO_INITIALISMS: &[&str] = &[
    "ACL", "API", "CLI", "CPU", "DB", "EOF", "GID", "HTML", "HTTP", "HTTPS", "ID", "IP", "JSON",
    "LLM", "MCP", "OS", "RAM", "RPC", "SDK", "SQL", "SSE", "TLS", "TTL", "UID", "URI", "URL",
    "UUID", "XML",
];

/// Convert a schema name to an exported Go identifier.
///
/// Splits on `_` and `-`, uppercases any segment that is a known initialism,
/// and title-cases the rest. Go keywords are all lowercase and the result is
/// always capitalised, so a keyword collision is impossible.
pub fn to_go_name(s: &str) -> String {
    let mut out = String::new();
    for segment in s.split(['_', '-']).filter(|seg| !seg.is_empty()) {
        let upper = segment.to_ascii_uppercase();
        if GO_INITIALISMS.contains(&upper.as_str()) {
            out.push_str(&upper);
        } else {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                out.push(first.to_ascii_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    // A Go identifier cannot start with a digit.
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'F');
    }
    out
}

/// Pad every cell but the last in each row to its column width, joined by a
/// single space.
///
/// gofmt aligns contiguous runs of struct fields and const entries this way:
/// tabs for indentation, spaces for alignment. Callers are responsible for
/// splitting rows into runs — a preceding comment line flushes gofmt's
/// alignment block, so a field carrying a doc comment starts a new run.
pub fn align_columns(rows: &[Vec<String>]) -> Vec<String> {
    let column_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths = vec![0usize; column_count];
    for row in rows {
        // The final cell of a row is never padded, so it never sets a width.
        for (i, cell) in row.iter().enumerate().take(row.len().saturating_sub(1)) {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }

    rows.iter()
        .map(|row| {
            let mut line = String::new();
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    line.push(' ');
                }
                line.push_str(cell);
                if i + 1 != row.len() {
                    for _ in cell.chars().count()..widths[i] {
                        line.push(' ');
                    }
                }
            }
            line
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_snake_case_with_initialisms() {
        assert_eq!(to_go_name("id"), "ID");
        assert_eq!(to_go_name("api_key"), "APIKey");
        assert_eq!(to_go_name("base_url"), "BaseURL");
        assert_eq!(to_go_name("model_id"), "ModelID");
        assert_eq!(to_go_name("mcp_servers"), "MCPServers");
        assert_eq!(to_go_name("thinking_effort"), "ThinkingEffort");
        assert_eq!(to_go_name("name"), "Name");
    }

    #[test]
    fn leaves_pascal_case_segments_alone() {
        assert_eq!(to_go_name("Active"), "Active");
        assert_eq!(to_go_name("PlainText"), "PlainText");
    }

    #[test]
    fn prefixes_names_that_would_start_with_a_digit() {
        assert_eq!(to_go_name("2fa_secret"), "F2faSecret");
    }

    #[test]
    fn aligns_all_but_the_last_column() {
        let rows = vec![
            vec![
                "Name".to_string(),
                "string".to_string(),
                "`json:\"name\"`".to_string(),
            ],
            vec![
                "ID".to_string(),
                "int64".to_string(),
                "`json:\"id\"`".to_string(),
            ],
        ];
        assert_eq!(
            align_columns(&rows),
            vec![
                "Name string `json:\"name\"`".to_string(),
                "ID   int64  `json:\"id\"`".to_string(),
            ]
        );
    }

    #[test]
    fn aligns_a_single_row_without_padding() {
        let rows = vec![vec!["Name".to_string(), "string".to_string()]];
        assert_eq!(align_columns(&rows), vec!["Name string".to_string()]);
    }
}
