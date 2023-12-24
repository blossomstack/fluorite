use std::collections::HashMap;

use crate::code_gen::utils::get_type_name;
mod type_graph;

mod pre_processor;
pub use pre_processor::*;
mod package_writer;
pub use package_writer::*;
mod type_writer;
pub use type_writer::*;

use super::abi::{
    CodeGenConfig, CodeGenContext, CustomTypeWriter, PackageWriter, PreProcessor, TypeInfo,
};
pub struct RustCodeGenConfig {}

struct ExtraTypeInfo {
    cyclic_ref_group_id: Option<u32>,
}

pub struct RustCodeGenContext {
    extra_type_infos: HashMap<String, ExtraTypeInfo>,
    type_dict: HashMap<String, TypeInfo>,
}
impl CodeGenConfig<RustCodeGenContext> for RustCodeGenConfig {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<RustCodeGenContext>> {
        Box::new(RustPreProcessor {})
    }

    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<RustCodeGenContext>>> {
        Some(Box::new(RustPackageWriter {}))
    }

    fn get_type_writer(&self) -> Box<dyn CustomTypeWriter<RustCodeGenContext>> {
        Box::new(RustTypeWriter {})
    }
}

impl CodeGenContext for RustCodeGenContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        &self.type_dict
    }
}

impl RustCodeGenContext {
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

impl RustCodeGenConfig {
    pub fn new() -> Self {
        Self {}
    }
}
