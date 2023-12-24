use std::collections::HashMap;

use crate::definitions::{CustomType, Definition};

use super::abi::{
    CodeGenContext, CodeGenProvider, EnumTypeInfo, ObjectEnumTypeInfo, ObjectEnumValue,
    ObjectField, ObjectTypeInfo, PreProcessor, TypeInfo,
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
        let type_dict = self.build_type_dict(definitions, &pre_processor)?;
        let context = pre_processor.process(type_dict)?;
        let mut packages = HashMap::new();
        for type_info in context.type_dict().values() {
            packages
                .entry(type_info.package())
                .or_insert(Vec::new())
                .push(type_info);
        }
        for (package, types) in packages {
            if let Some(package_writer) = self.config.get_package_writer() {
                package_writer.write_package(&package, &types, &context)?;
            }
            for type_info in types.into_iter().filter(|t| !t.is_object_enum_value()) {
                self.gen_code_for(type_info, &context)?;
            }
        }

        Ok(())
    }

    fn build_type_dict(
        &self,
        definitions: &Vec<Definition>,
        pre_processor: &Box<dyn PreProcessor<C>>,
    ) -> anyhow::Result<HashMap<String, TypeInfo>> {
        let mut object_enum_value_type_names = Vec::new();
        let mut all_type_names = Vec::new();
        for d in definitions {
            for t in &d.types {
                match t {
                    CustomType::Object { name, fields: _ } => all_type_names.push(name.clone()),
                    CustomType::Enum { name, values: _ } => all_type_names.push(name.clone()),
                    CustomType::ObjectEnum {
                        name,
                        type_tag: _,
                        values,
                    } => {
                        all_type_names.push(name.clone());
                        for v in values {
                            object_enum_value_type_names.push(v.clone());
                        }
                    }
                }
            }
        }

        // identify all types
        let mut all_types: HashMap<String, TypeInfo> = HashMap::new();
        for d in definitions {
            let package = pre_processor.get_package_name(d)?;
            for t in &d.types {
                match t {
                    CustomType::Object { name, fields } => {
                        let fields = fields.iter().map(|f| ObjectField::from(f)).collect();
                        let is_object_enum_value = object_enum_value_type_names.contains(name);
                        let type_info = ObjectTypeInfo {
                            package: package.clone(),
                            name: name.clone(),
                            fields,
                            is_object_enum_value,
                        };
                        all_types.insert(name.clone(), TypeInfo::Object(type_info));
                    }
                    CustomType::Enum { name, values } => {
                        let type_info = EnumTypeInfo {
                            package: package.clone(),
                            name: name.clone(),
                            values: values.clone(),
                        };
                        all_types.insert(name.clone(), TypeInfo::Enum(type_info));
                    }
                    CustomType::ObjectEnum {
                        name,
                        type_tag,
                        values,
                    } => {
                        let values = values
                            .iter()
                            .map(|v| match all_type_names.contains(v) {
                                true => ObjectEnumValue::ObjectType(v.clone()),
                                false => ObjectEnumValue::Simple(v.clone()),
                            })
                            .collect();
                        let type_info = ObjectEnumTypeInfo {
                            package: package.clone(),
                            name: name.clone(),
                            type_tag: type_tag.clone(),
                            values,
                        };
                        all_types.insert(name.clone(), TypeInfo::ObjectEnum(type_info));
                    }
                }
            }
        }
        Ok(all_types)
    }

    fn gen_code_for(&self, type_info: &TypeInfo, context: &C) -> anyhow::Result<()> {
        let type_writer = self.config.get_type_writer();
        let mut writer = type_writer.get_writer_for_type(type_info, context)?;
        match type_info {
            TypeInfo::Object(object_type_info) => {
                // write type
                type_writer.pre_write_type(&mut writer, type_info, context)?;
                let object_writer = self.config.get_object_writer();
                object_writer.pre_write_object(&mut writer, object_type_info, context)?;
                // write fields
                for field in object_type_info.fields.iter() {
                    object_writer.write_field(&mut writer, field, object_type_info, context)?;
                }
                object_writer.post_write_object(&mut writer, object_type_info, context)?;
            }
            TypeInfo::Enum(enum_type_info) => {
                // write type
                type_writer.pre_write_type(&mut writer, type_info, context)?;
                let enum_writer = self.config.get_enum_writer();

                enum_writer.pre_write_enum(&mut writer, enum_type_info, context)?;
                // write values
                for value in enum_type_info.values.iter() {
                    enum_writer.write_enum_value(&mut writer, value, enum_type_info, context)?;
                }
                enum_writer.post_write_enum(&mut writer, enum_type_info, context)?;
            }
            TypeInfo::ObjectEnum(object_enum_type_info) => {
                // write type
                type_writer.pre_write_type(&mut writer, type_info, context)?;
                let object_enum_writer = self.config.get_object_enum_writer();
                object_enum_writer.pre_write_object_enum(
                    &mut writer,
                    object_enum_type_info,
                    context,
                )?;
                for value in object_enum_type_info.values.iter() {
                    object_enum_writer.write_object_enum_value(
                        &mut writer,
                        value,
                        object_enum_type_info,
                        context,
                    )?;
                }
                object_enum_writer.post_write_object_enum(
                    &mut writer,
                    object_enum_type_info,
                    context,
                )?;
            }
        };
        Ok(())
    }
}
