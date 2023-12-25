use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionConfig {
    pub rust_package: Option<String>,
}
