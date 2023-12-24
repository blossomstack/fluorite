use std::collections::HashMap;

use anyhow::anyhow;

use crate::{
    code_gen::abi::{PreProcessor, TypeInfo},
    definitions::Definition,
};

use super::{type_graph::TypeGraph, ExtraTypeInfo, RustContext, RustOptions};

pub struct RustPreProcessor {
    pub options: RustOptions,
}

impl PreProcessor<RustContext> for RustPreProcessor {
    fn process(&self, types_dict: HashMap<String, TypeInfo>) -> anyhow::Result<Box<RustContext>> {
        // Detect the cyclic referenced types
        let graph = TypeGraph::new(&types_dict)?;
        let type_sub_graphs = graph.group_cyclic_referenced_types()?;

        let mut extra_type_infos = HashMap::new();

        let mut group_id = 0;
        for sub_graph in type_sub_graphs {
            let opt_group_id = if sub_graph.len() <= 1 {
                None
            } else {
                group_id = group_id + 1;
                Some(group_id)
            };
            for type_info in sub_graph {
                let extra_type_info = ExtraTypeInfo {
                    cyclic_ref_group_id: opt_group_id,
                };
                extra_type_infos.insert(type_info.type_name().to_string(), extra_type_info);
            }
        }

        let context = RustContext {
            extra_type_infos,
            types_dict,
            options: self.options.clone(),
        };
        Ok(Box::new(context))
    }

    fn get_package_name(&self, definition: &Definition) -> anyhow::Result<String> {
        match definition.configs.as_ref().map(|c| c.rust_package.clone()) {
            Some(package) => Ok(package),
            _ => Err(anyhow!("cannot find package info from definition")),
        }
    }
}

impl RustPreProcessor {}
