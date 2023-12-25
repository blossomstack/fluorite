use std::collections::HashMap;

use crate::definitions::Definition;

use super::{
    abi::{CodeGenContext, CodeGenProvider, TypeInfo},
    utils::build_type_dict,
};

pub struct CodeGenerator<C: CodeGenContext> {
    config: Box<dyn CodeGenProvider<C>>,
}

impl<C: CodeGenContext> CodeGenerator<C> {
    pub fn new(config: Box<dyn CodeGenProvider<C>>) -> Self {
        Self { config }
    }

    pub fn generate(&self, definitions: &Vec<Definition>) -> anyhow::Result<()> {
        let pre_processor = self.config.get_pre_processor();
        let type_dict = build_type_dict(definitions, &pre_processor)?;
        let context = pre_processor.process(type_dict)?;

        // group types by packages for next code gen
        let mut packages = HashMap::new();
        for type_info in context.type_dict().values() {
            packages
                .entry(type_info.package())
                .or_insert(Vec::new())
                .push(type_info);
        }
        for (package, types) in packages {
            if let Some(package_writer) = self.config.get_package_writer() {
                package_writer.write_package(package, &types, &context)?;
            }
            for type_info in types.into_iter().filter(|t| !t.is_object_enum_value()) {
                self.gen_code_for(type_info, &context)?;
            }
        }

        Ok(())
    }

    fn gen_code_for(&self, type_info: &TypeInfo, context: &C) -> anyhow::Result<()> {
        let mut writer = context.get_writer_for_type(type_info)?;
        match type_info {
            TypeInfo::Object(object_type_info) => {
                let object_writer = self.config.get_object_writer();
                object_writer.write_object(&mut writer, object_type_info, context)?;
            }
            TypeInfo::Enum(enum_type_info) => {
                let enum_writer = self.config.get_enum_writer();

                enum_writer.write_enum(&mut writer, enum_type_info, context)?;
            }
            TypeInfo::ObjectEnum(object_enum_type_info) => {
                let object_enum_writer = self.config.get_object_enum_writer();
                object_enum_writer.write_object_enum(
                    &mut writer,
                    object_enum_type_info,
                    context,
                )?;
            }
            TypeInfo::List(list_type_info) => {
                let list_writer = self.config.get_list_writer();
                list_writer.write_list(&mut writer, list_type_info, context)?;
            }
            TypeInfo::Map(map_type_info) => {
                let map_writer = self.config.get_map_writer();
                map_writer.write_map(&mut writer, map_type_info, context)?;
            }
        };
        writer.flush()?;
        Ok(())
    }
}
