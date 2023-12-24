use std::collections::HashMap;

use crate::definitions::{CustomType, Definition};

use super::abi::{CodeGenConfig, CodeGenContext, TypeInfo};

pub struct CodeGenerator<C: CodeGenContext> {
    config: Box<dyn CodeGenConfig<C>>,
}

impl<C: CodeGenContext> CodeGenerator<C> {
    pub fn new(config: Box<dyn CodeGenConfig<C>>) -> Self {
        Self { config }
    }

    pub fn generate(&self, definitions: &Vec<Definition>) -> anyhow::Result<()> {
        let pre_processor = self.config.get_pre_processor();
        let context = pre_processor.process(&definitions)?;
        let mut packages = HashMap::new();
        for type_info in context.type_dict().values() {
            packages
                .entry(type_info.package.clone())
                .or_insert(Vec::new())
                .push(type_info);
        }
        for (package, types) in packages {
            if let Some(package_writer) = self.config.get_package_writer() {
                package_writer.write_package(&package, &types, &context)?;
            }
            for type_info in types {
                self.gen_code_for(type_info, &context)?;
            }
        }

        Ok(())
    }

    fn gen_code_for(&self, type_info: &TypeInfo, context: &C) -> anyhow::Result<()> {
        let type_writer = self.config.get_type_writer();
        let mut writer = type_writer.get_writer_for_type(type_info, context)?;
        match &type_info.type_def {
            CustomType::Object { name: _, fields } => {
                // write type
                type_writer.pre_write_type(&mut writer, type_info, context)?;
                type_writer.pre_write_object(&mut writer, type_info, context)?;
                // write fields
                for field in fields.iter() {
                    type_writer.write_field(&mut writer, field, type_info, context)?;
                }
                type_writer.post_write_object(&mut writer, type_info, context)?;
            }
            CustomType::Enum { name: _, values } => {
                // write type
                type_writer.pre_write_type(&mut writer, type_info, context)?;
                type_writer.pre_write_enum(&mut writer, type_info, context)?;
                // write fields
                for value in values.iter() {
                    type_writer.write_enum_value(&mut writer, value, type_info, context)?;
                }
                type_writer.post_write_enum(&mut writer, type_info, context)?;
            }
        };
        Ok(())
    }
}
