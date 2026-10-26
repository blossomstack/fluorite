use crate::code_gen::utils::to_camel_case;

#[derive(Debug, Clone)]
pub struct TypeScriptOptions {
    pub output_dir: String,
    pub single_file: bool,
    pub package_name: Option<String>, // Override package directory name
    pub any_type: String,
    pub use_readonly: bool,
    pub runtime_decoders: bool,
    pub reject_unknown_fields: bool,
}

impl TypeScriptOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: false,
            package_name: None,
            any_type: "unknown".to_owned(),
            use_readonly: false,
            runtime_decoders: false,
            reject_unknown_fields: false,
        }
    }

    pub fn with_runtime_decoders(mut self, enabled: bool) -> Self {
        self.runtime_decoders = enabled;
        self
    }

    /// Reject undeclared object fields in generated runtime decoders (opt-in).
    pub fn with_reject_unknown_fields(mut self, enabled: bool) -> Self {
        self.reject_unknown_fields = enabled;
        self
    }

    pub fn with_single_file(mut self, single_file: bool) -> Self {
        self.single_file = single_file;
        self
    }

    pub fn with_any_type(mut self, any_type: &str) -> Self {
        self.any_type = any_type.to_owned();
        self
    }

    pub fn with_readonly(mut self, use_readonly: bool) -> Self {
        self.use_readonly = use_readonly;
        self
    }

    pub fn with_package_name(mut self, package_name: &str) -> Self {
        self.package_name = Some(package_name.to_owned());
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_camel_case(type_name)
    }
}
