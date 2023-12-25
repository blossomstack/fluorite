use std::collections::HashMap;

use anyhow::anyhow;

use crate::{
    code_gen::abi::{PreProcessor, TypeInfo},
    definitions::Definition,
};

use super::{RustContext, RustOptions};

pub struct RustPreProcessor {
    pub options: RustOptions,
}

impl PreProcessor<RustContext> for RustPreProcessor {
    fn process(&self, types_dict: HashMap<String, TypeInfo>) -> anyhow::Result<Box<RustContext>> {
        let context = RustContext {
            types_dict,
            options: self.options.clone(),
        };
        Ok(Box::new(context))
    }

    fn get_package_name(&self, definition: &Definition) -> anyhow::Result<String> {
        match definition.configs.rust_package.as_ref() {
            Some(package) => Ok(package.clone()),
            _ => Err(anyhow!("cannot find package info from definition")),
        }
    }
}

impl RustPreProcessor {}
