use crate::code_gen::abi::{CustomTypeWriter, TypeInfo};

use super::RustCodeGenContext;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::anyhow;

use crate::definitions::{Field, FieldType};

pub struct RustTypeWriter {}

impl RustTypeWriter {
    fn get_fully_qualified_type_name(
        &self,
        type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> String {
        format!("crate::{}::{}", type_info.package, type_info.type_name())
    }
}

impl CustomTypeWriter<RustCodeGenContext> for RustTypeWriter {
    fn get_writer_for_type(
        &self,
        type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<Box<dyn Write>> {
        let type_name = type_info.type_name();
        let output_path = format!("/tmp/test_gen/{}", type_info.package);
        let output_file_name = format!("{}/{}.rs", output_path, type_name);
        let file = File::create(output_file_name)?;
        let writer = BufWriter::new(file);
        Ok(Box::new(writer))
    }

    fn pre_write_type(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        if type_info.is_enum() {
            return Ok(());
        }
        writer.write_all("use serde::{Serializer, Deserializer};".as_bytes())?;
        writer.write_all("\n\n".as_bytes())?;
        Ok(())
    }

    fn pre_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serializer, Deserializer)]\n".as_bytes())?;
        writer.write_all(format!("pub struct {} {{\n", type_info.type_name()).as_bytes())?;
        Ok(())
    }

    fn write_field(
        &self,
        writer: &mut Box<dyn Write>,
        field: &Field,
        type_info: &TypeInfo,
        context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        let type_to_write = match &field.field_type {
            FieldType::Simple(t) => t.to_string(),
            FieldType::Custom { name } => {
                // check if the field type and current type is in same cyclic ref group
                let ref_type = context
                    .type_dict
                    .get(name)
                    .ok_or_else(|| anyhow!("Cannot find field type: {}", name))?;
                let full_type_name = self.get_fully_qualified_type_name(type_info, context);
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
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}\n".as_bytes())?;
        Ok(())
    }

    fn pre_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        writer.write_all("#[derive(Debug, Clone, Serializer, Deserializer)]\n".as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", type_info.type_name()).as_bytes())?;
        Ok(())
    }

    fn write_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &str,
        _type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        writer.write_all(format!("  {},\n", value).as_bytes())?;
        Ok(())
    }

    fn post_write_enum(
        &self,
        writer: &mut dyn Write,
        _type_info: &TypeInfo,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        writer.write_all("}".as_bytes())?;
        Ok(())
    }
}
