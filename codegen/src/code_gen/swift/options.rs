use crate::code_gen::utils::to_pascal_case;

#[derive(Debug, Clone)]
pub struct SwiftOptions {
    pub output_dir: String,
    pub single_file: bool,
    pub any_type: String,
    pub visibility: SwiftVisibility,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SwiftVisibility {
    #[default]
    Public,
    Internal,
    Package,
}

impl SwiftOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: false,
            any_type: "AnyCodable".to_owned(),
            visibility: SwiftVisibility::Public,
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

    pub fn with_visibility(mut self, visibility: SwiftVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        // Swift uses PascalCase for file names (e.g., User.swift, UserStatus.swift)
        to_pascal_case(type_name)
    }

    pub fn get_visibility_string(&self) -> &'static str {
        match self.visibility {
            SwiftVisibility::Public => "public",
            SwiftVisibility::Internal => "internal",
            SwiftVisibility::Package => "package",
        }
    }
}
