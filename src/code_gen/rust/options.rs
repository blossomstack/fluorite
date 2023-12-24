use crate::code_gen::utils::to_snake_case;

#[derive(Debug, Clone)]
pub struct RustOptions {
    pub output_dir: String,
}

impl RustOptions {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_snake_case(type_name)
    }
}
