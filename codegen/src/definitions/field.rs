use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub configs: Option<crate::definitions::FieldConfig>,
}
