use crate::code_gen::utils::to_snake_case;

#[derive(Debug, Clone)]
pub struct RustOptions {
    pub output_dir: String,
    pub single_file: bool,
}

impl RustOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: true,
        }
    }

    pub fn with_single_file(&mut self, single_file: bool) -> &mut RustOptions {
        self.single_file = single_file;
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_snake_case(type_name)
    }

    pub(crate) fn get_simple_type(&self, t: &crate::definitions::SimpleType) -> String {
        match t {
            crate::definitions::SimpleType::String => "String".to_string(),
            crate::definitions::SimpleType::Bool => "bool".to_string(),
            crate::definitions::SimpleType::DateTime => "DateTime".to_string(),
            crate::definitions::SimpleType::UInt32 => "u32".to_string(),
            crate::definitions::SimpleType::UInt64 => "u64".to_string(),
            crate::definitions::SimpleType::Int32 => "i32".to_string(),
            crate::definitions::SimpleType::Int64 => "i64".to_string(),
            crate::definitions::SimpleType::Float32 => "f32".to_string(),
            crate::definitions::SimpleType::Float64 => "f64".to_string(),
        }
    }
}
