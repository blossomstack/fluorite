use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
}
