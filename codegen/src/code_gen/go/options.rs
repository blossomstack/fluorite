#[derive(Debug, Clone)]
pub struct GoOptions {
    pub output_dir: String,
    pub single_file: bool,
    /// Override for the emitted `package` clause. Defaults to the output
    /// directory's basename.
    pub package_name: Option<String>,
    pub any_type: String,
}

impl GoOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: false,
            package_name: None,
            any_type: "any".to_owned(),
        }
    }

    pub fn with_single_file(mut self, single_file: bool) -> Self {
        self.single_file = single_file;
        self
    }

    pub fn with_any_type(mut self, any_type: &str) -> Self {
        self.any_type = any_type.to_owned();
        self
    }

    pub fn with_package_name(mut self, package_name: &str) -> Self {
        self.package_name = Some(package_name.to_owned());
        self
    }

    /// The name for the `package` clause: the override if given, otherwise the
    /// output directory's basename, sanitised into a legal Go identifier.
    pub fn resolved_package_name(&self) -> String {
        let raw = match &self.package_name {
            Some(name) => name.clone(),
            None => self
                .output_dir
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or("generated")
                .to_string(),
        };
        sanitize_package_name(&raw)
    }
}

/// Lowercase, replace anything that is not a letter or digit with `_`, and
/// prefix a leading digit. Empty input falls back to `generated`.
fn sanitize_package_name(raw: &str) -> String {
    let mut out: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() {
        return "generated".to_string();
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'p');
    }
    out
}
