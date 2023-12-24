use crate::code_gen::abi::{
    CodeGenContext, CustomTypeWriter, EnumTypeInfo, EnumWriter, FieldType, ObjectEnumTypeInfo,
    ObjectEnumValue, ObjectEnumWriter, ObjectField, ObjectTypeInfo, ObjectWriter, TypeInfo,
};

use super::RustContext;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::anyhow;

pub struct RustTypeWriter {}

impl RustTypeWriter {
    fn get_fully_qualified_type_name(
        &self,
        type_info: &TypeInfo,
        _context: &RustContext,
    ) -> String {
        format!("crate::{}::{}", type_info.package(), type_info.type_name(),)
    }
}

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
        type_info: &TypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        writer.write_all("use serde::{Serialize, Deserialize};".as_bytes())?;
        writer.write_all("\n\n".as_bytes())?;
        if type_info.has_unknown_fields() {
            todo!()
        }
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
        let ref this = self;
        let field = field;
        let type_to_write = match &field.field_type {
            FieldType::Simple(t) => context.options.get_simple_type(&t),
            FieldType::Custom { name } => {
                // check if the field type and current type is in same cyclic ref group
                let ref_type = context
                    .types_dict
                    .get(name)
                    .ok_or_else(|| anyhow!("Cannot find field type: {}", name))?;
                let full_type_name = this.get_fully_qualified_type_name(ref_type, context);
                if context.is_cyclic_ref(type_info.name.as_str(), ref_type.type_name()) {
                    format!("Box<{}>", full_type_name)
                } else {
                    full_type_name
                }
            }
            FieldType::UnknownFields => "HashMap<String, Value>".to_string(),
        };
        if type_info.is_object_enum_value {
            writer.write_all(format!("    {}: {},\n", field.name, type_to_write).as_bytes())?;
        } else {
            writer.write_all(format!("  pub {}: {},\n", field.name, type_to_write).as_bytes())?;
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
            ObjectEnumValue::ObjectType(type_name) => match context.type_dict().get(type_name) {
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

impl RustTypeWriter {}
