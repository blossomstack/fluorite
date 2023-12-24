use std::collections::HashMap;

use anyhow::anyhow;

use crate::{
    code_gen::{
        abi::{PreProcessor, TypeInfo},
        utils::get_type_name,
    },
    definitions::{CustomType, Definition},
};

use super::{type_graph::TypeGraph, ExtraTypeInfo, RustCodeGenContext};

pub struct RustPreProcessor {}

impl PreProcessor<RustCodeGenContext> for RustPreProcessor {
    fn process(&self, definitions: &Vec<Definition>) -> anyhow::Result<Box<RustCodeGenContext>> {
        // Detect the cyclic referenced types
        let mut all_types: Vec<TypeInfo> = Vec::new();
        for d in definitions {
            for t in &d.custom_types {
                all_types.push(Self::create_type_info(&d, t.clone())?);
            }
        }
        let graph = TypeGraph::new(&all_types)?;
        let type_sub_graphs = graph.group_cyclic_referenced_types(&all_types)?;

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
                extra_type_infos.insert(get_type_name(&type_info.type_def), extra_type_info);
            }
        }

        let type_dict: HashMap<String, TypeInfo> = all_types
            .into_iter()
            .map(|t| (get_type_name(&t.type_def), t))
            .collect();
        let context = RustCodeGenContext {
            extra_type_infos,
            type_dict,
        };
        Ok(Box::new(context))
    }
}

impl RustPreProcessor {
    fn create_type_info(definition: &Definition, t: CustomType) -> anyhow::Result<TypeInfo> {
        match definition.configs.get("rust_package") {
            Some(serde_yaml::Value::String(package)) => Ok(TypeInfo::new(t, package.clone())),
            _ => Err(anyhow!("cannot find package info from definition")),
        }
    }
}
