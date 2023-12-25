use std::collections::HashMap;

use crate::code_gen::abi::{CodeGenContext, TypeInfo, TypeName};

use super::RustOptions;
use anyhow::anyhow;

pub struct RustContext {
    pub types_dict: HashMap<String, TypeInfo>,
    pub options: RustOptions,
}

impl CodeGenContext for RustContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        &self.types_dict
    }
}

impl RustContext {
    pub fn get_fully_qualified_type_name(&self, type_name: &TypeName) -> anyhow::Result<String> {
        let full_type_name = match type_name {
            TypeName::Simple(t) => self.options.get_simple_type(t),
            TypeName::CustomType(name) => {
                let type_info = self
                    .types_dict
                    .get(name)
                    .ok_or_else(|| anyhow!("Cannot find custom type: {}", name))?;
                format!("crate::{}::{}", type_info.package(), type_info.type_name())
            }
        };
        Ok(full_type_name)
    }
}
