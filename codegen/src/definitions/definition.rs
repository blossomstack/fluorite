use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub types: crate::definitions::CustomTypeList,
    pub configs: crate::definitions::DefinitionConfig,
}
