use std::collections::HashMap;

use crate::code_gen::{
    abi::{CodeGenContext, TypeInfo},
    utils::get_type_name,
};

use super::RustOptions;

pub struct RustContext {
    pub extra_type_infos: HashMap<String, ExtraTypeInfo>,
    pub type_dict: HashMap<String, TypeInfo>,
    pub options: RustOptions,
}

pub struct ExtraTypeInfo {
    pub cyclic_ref_group_id: Option<u32>,
}

impl CodeGenContext for RustContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        &self.type_dict
    }
}

impl RustContext {
    fn get_opt_cyclic_group_id(&self, type_info: &TypeInfo) -> Option<u32> {
        self.extra_type_infos
            .get(&get_type_name(&type_info.type_def))
            .and_then(|i| i.cyclic_ref_group_id)
    }

    pub fn is_cyclic_ref(&self, type_info: &TypeInfo, ref_type_info: &TypeInfo) -> bool {
        let gid1 = self.get_opt_cyclic_group_id(type_info);
        let gid2 = self.get_opt_cyclic_group_id(ref_type_info);
        gid1 != None && gid1 == gid2
    }
}
