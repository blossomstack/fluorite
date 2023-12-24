use std::collections::HashMap;

use crate::code_gen::abi::{CodeGenContext, TypeInfo};

use super::RustOptions;

pub struct RustContext {
    pub extra_type_infos: HashMap<String, ExtraTypeInfo>,
    pub types_dict: HashMap<String, TypeInfo>,
    pub options: RustOptions,
}

pub struct ExtraTypeInfo {
    pub cyclic_ref_group_id: Option<u32>,
}

impl CodeGenContext for RustContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        &self.types_dict
    }
}

impl RustContext {
    fn get_opt_cyclic_group_id(&self, type_name: &str) -> Option<u32> {
        self.extra_type_infos
            .get(type_name)
            .and_then(|i| i.cyclic_ref_group_id)
    }

    pub fn is_cyclic_ref(&self, type_name: &str, ref_type_name: &str) -> bool {
        let gid1 = self.get_opt_cyclic_group_id(type_name);
        let gid2 = self.get_opt_cyclic_group_id(ref_type_name);
        gid1 != None && gid1 == gid2
    }
}
