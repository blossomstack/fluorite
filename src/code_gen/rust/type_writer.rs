use crate::code_gen::{
    abi::{CustomTypeWriter, TypeInfo},
    utils::{get_field_type, FieldType},
};

use super::RustContext;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::anyhow;

use crate::definitions::Field;

pub struct RustTypeWriter {}

impl RustTypeWriter {
    fn get_fully_qualified_type_name(
        &self,
        type_info: &TypeInfo,
        _context: &RustContext,
    ) -> String {
        format!("crate::{}::{}", type_info.package, type_info.type_name(),)
    }
}

impl CustomTypeWriter<RustContext> for RustTypeWriter {
    fn get_writer_for_type(
        &self,
        type_info: &TypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<Box<dyn Write>> {
        let type_name = type_info.type_name();
        let output_path = format!("{}/{}", context.options.output_dir, type_info.package);
        let output_file_name = format!(
            "{}/{}.rs",
            output_path,
            context.options.type_to_file_name(type_name.as_str())
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

    fn pre_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub struct {} {{\n", type_info.type_name()).as_bytes())?;
        Ok(())
    }

    fn write_field(
        &self,
        writer: &mut Box<dyn Write>,
        field: &Field,
        type_info: &TypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let type_to_write = match get_field_type(&field.field_type) {
            FieldType::Simple(t) => context.options.get_simple_type(&t),
            FieldType::Custom { name } => {
                // check if the field type and current type is in same cyclic ref group
                let ref_type = context
                    .type_dict
                    .get(&name)
                    .ok_or_else(|| anyhow!("Cannot find field type: {}", name))?;
                let full_type_name = self.get_fully_qualified_type_name(ref_type, context);
                if context.is_cyclic_ref(type_info, ref_type) {
                    format!("Box<{}>", full_type_name)
                } else {
                    full_type_name
                }
            }
        };
        writer.write_all(format!("  pub {}: {},\n", field.name, type_to_write).as_bytes())?;
        Ok(())
    }

    fn post_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        _type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}\n".as_bytes())?;
        Ok(())
    }

    fn pre_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", type_info.type_name()).as_bytes())?;
        Ok(())
    }

    fn write_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &str,
        _type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all(format!("  {},\n", value).as_bytes())?;
        Ok(())
    }

    fn post_write_enum(
        &self,
        writer: &mut dyn Write,
        _type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}".as_bytes())?;
        Ok(())
    }
}
