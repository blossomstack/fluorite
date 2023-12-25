use crate::code_gen::abi::{
    CodeGenContext, CustomTypeWriter, EnumTypeInfo, EnumWriter, ListTypeInfo, ListWriter,
    MapTypeInfo, MapWriter, ObjectEnumTypeInfo, ObjectEnumValue, ObjectEnumWriter, ObjectField,
    ObjectTypeInfo, ObjectWriter, TypeInfo,
};

use super::RustContext;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::anyhow;

pub struct RustTypeWriter {}

impl RustTypeWriter {}

impl CustomTypeWriter<RustContext> for RustTypeWriter {
    fn get_writer_for_type(
        &self,
        type_info: &TypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<Box<dyn Write>> {
        let type_name = type_info.type_name();
        let output_path = format!("{}/{}", context.options.output_dir, type_info.package());
        let output_file_name = format!(
            "{}/{}.rs",
            output_path,
            context.options.type_to_file_name(type_name)
        );
        let file = File::create(output_file_name)?;
        let writer = BufWriter::new(file);
        Ok(Box::new(writer))
    }

    fn pre_write_type(
        &self,
        writer: &mut Box<dyn Write>,
        _type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("use serde::{Serialize, Deserialize};".as_bytes())?;
        writer.write_all("\n\n".as_bytes())?;
        Ok(())
    }
}

impl ObjectWriter<RustContext> for RustTypeWriter {
    fn pre_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &ObjectTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub struct {} {{\n", type_info.name).as_bytes())?;
        Ok(())
    }

    fn write_field(
        &self,
        writer: &mut dyn Write,
        field: &ObjectField,
        type_info: &ObjectTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let type_to_write = context.get_fully_qualified_type_name(&field.field_type)?;
        let type_to_write = match &field
            .config
            .as_ref()
            .and_then(|c| c.rust_type_wrapper.clone())
        {
            Some(_) => format!("Box<{}>", type_to_write),
            _ => type_to_write,
        };
        match &field.config.as_ref().and_then(|c| c.rename.clone()) {
            Some(rename) => {
                if type_info.is_object_enum_value {
                    writer.write_all(
                        format!("    #[serde(rename = \"{}\")]\n", field.name).as_bytes(),
                    )?;
                    writer.write_all(format!("    {}: {},\n", rename, type_to_write).as_bytes())?;
                } else {
                    writer.write_all(
                        format!("  #[serde(rename = \"{}\")]\n", field.name).as_bytes(),
                    )?;
                    writer
                        .write_all(format!("  pub {}: {},\n", rename, type_to_write).as_bytes())?;
                }
            }
            None => {
                if type_info.is_object_enum_value {
                    writer.write_all(
                        format!("    {}: {},\n", field.name, type_to_write).as_bytes(),
                    )?;
                } else {
                    writer.write_all(
                        format!("  pub {}: {},\n", field.name, type_to_write).as_bytes(),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn post_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        _type_info: &ObjectTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}\n".as_bytes())?;
        Ok(())
    }
}

impl EnumWriter<RustContext> for RustTypeWriter {
    fn pre_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &EnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", type_info.name).as_bytes())?;
        Ok(())
    }

    fn write_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &str,
        _type_info: &EnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all(format!("  {},\n", value).as_bytes())?;
        Ok(())
    }

    fn post_write_enum(
        &self,
        writer: &mut dyn Write,
        _type_info: &EnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}".as_bytes())?;
        Ok(())
    }
}

impl ObjectEnumWriter<RustContext> for RustTypeWriter {
    fn pre_write_object_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectEnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        // TODO: read tag from type info
        writer.write_all(format!("#[serde(tag = \"{}\")]\n", "type").as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", type_info.name).as_bytes())?;
        Ok(())
    }

    fn write_object_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &ObjectEnumValue,
        _type_info: &ObjectEnumTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        match value {
            ObjectEnumValue::Simple(simple) => {
                writer.write_all(format!("  {},\n", simple).as_bytes())?;
            }
            ObjectEnumValue::CustomType(type_name) => match context.type_dict().get(type_name) {
                Some(TypeInfo::Object(type_info)) => {
                    writer.write_all(format!("  {} {{\n", type_info.name).as_bytes())?;
                    for field in &type_info.fields {
                        self.write_field(writer, field, type_info, context)?;
                    }
                    writer.write_all("  },\n".as_bytes())?;
                }
                _ => {
                    return Err(anyhow!(
                        "Enum cannot be nested within enum object: {}",
                        type_name
                    ));
                }
            },
        }
        Ok(())
    }

    fn post_write_object_enum(
        &self,
        writer: &mut dyn Write,
        _type_info: &ObjectEnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}".as_bytes())?;
        Ok(())
    }
}

impl MapWriter<RustContext> for RustTypeWriter {
    fn write_map(
        &self,
        writer: &mut dyn Write,
        type_info: &MapTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let value_type = context.get_fully_qualified_type_name(&type_info.value_type)?;
        let key_type = context.get_fully_qualified_type_name(&type_info.key_type)?;

        writer.write_all("use std::collections::HashMap;\n\n".as_bytes())?;
        writer.write_all(
            format!(
                "pub type {} = HashMap<{}, {}>;\n",
                type_info.name, key_type, value_type
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}
impl ListWriter<RustContext> for RustTypeWriter {
    fn write_list(
        &self,
        writer: &mut dyn Write,
        type_info: &ListTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let item_type = context.get_fully_qualified_type_name(&type_info.item_type)?;
        writer
            .write_all(format!("pub type {} = Vec<{}>;\n", type_info.name, item_type).as_bytes())?;
        Ok(())
    }
}
