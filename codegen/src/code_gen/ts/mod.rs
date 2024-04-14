use super::abi::{
    CodeGenContext, CodeGenProvider, EnumWriter, ListWriter, MapWriter, ObjectEnumWriter,
    ObjectWriter, PackageWriter, PreProcessor, TypeInfo,
};
use std::collections::HashMap;
use std::io::Write;

pub struct TsOptions {}

pub struct TsContext {}
impl CodeGenContext for TsContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        todo!()
    }

    fn get_writer_for_type(&self, _type_info: &TypeInfo) -> anyhow::Result<Box<dyn Write>> {
        todo!()
    }
}
pub struct TsProvider {
    options: TsOptions,
}

impl TsProvider {
    pub fn new(options: TsOptions) -> Self {
        Self { options }
    }
}
impl CodeGenProvider<TsContext> for TsProvider {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<TsContext>> {
        todo!()
    }

    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<TsContext>>> {
        todo!()
    }

    fn get_object_writer(&self) -> Box<dyn ObjectWriter<TsContext>> {
        todo!()
    }

    fn get_enum_writer(&self) -> Box<dyn EnumWriter<TsContext>> {
        todo!()
    }

    fn get_object_enum_writer(&self) -> Box<dyn ObjectEnumWriter<TsContext>> {
        todo!()
    }

    fn get_list_writer(&self) -> Box<dyn ListWriter<TsContext>> {
        todo!()
    }

    fn get_map_writer(&self) -> Box<dyn MapWriter<TsContext>> {
        todo!()
    }
}
