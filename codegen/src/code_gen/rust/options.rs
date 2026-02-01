use crate::code_gen::utils::to_snake_case;

#[derive(Debug, Clone)]
pub struct RustOptions {
    pub output_dir: String,
    pub single_file: bool,
    pub any_type: String,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub generate_new: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Public,
    PublicCrate,
    Private,
}

impl RustOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: true,
            any_type: "fluorite::Any".to_owned(),
            derives: Self::default_derives(),
            visibility: Visibility::Public,
            generate_new: true,
        }
    }

    pub fn default_derives() -> Vec<String> {
        vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "PartialEq".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ]
    }

    pub fn with_single_file(mut self, single_file: bool) -> Self {
        self.single_file = single_file;
        self
    }

    pub fn with_any_type(mut self, any_type: &str) -> Self {
        self.any_type = any_type.to_owned();
        self
    }

    pub fn with_derives(mut self, derives: Vec<String>) -> Self {
        self.derives = derives;
        self
    }

    pub fn with_additional_derives(mut self, derives: Vec<String>) -> Self {
        self.derives.extend(derives);
        self
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_generate_new(mut self, generate_new: bool) -> Self {
        self.generate_new = generate_new;
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_snake_case(type_name)
    }

    pub fn get_derives_string(&self) -> String {
        let mut derives = self.derives.clone();
        if self.generate_new {
            derives.push("derive_new::new".to_string());
        }
        format!("#[derive({})]", derives.join(", "))
    }

    pub fn get_visibility_string(&self) -> &'static str {
        match self.visibility {
            Visibility::Public => "pub",
            Visibility::PublicCrate => "pub(crate)",
            Visibility::Private => "",
        }
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
