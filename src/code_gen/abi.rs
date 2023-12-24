use std::{collections::HashMap, io::Write};

use crate::definitions::{Definition, SimpleType};

mod type_info;
pub use type_info::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FieldType {
    // unrecognized fields when deserializing the field
    UnknownFields,
    Simple(SimpleType),
    Custom { name: String },
}

const UNKNOWN_FIELDS_TYPE_NAME: &'static str = "UnknownFields";

impl FieldType {
    pub fn is_custom_type(&self) -> bool {
        match self {
            FieldType::UnknownFields => false,
            FieldType::Simple(_) => false,
            FieldType::Custom { name: _ } => true,
        }
    }

    pub fn get_field_type(field_type: &str) -> FieldType {
        let opt_simple_type = SimpleType::all_values()
            .into_iter()
            .find(|t| t.to_string() == field_type);
        match opt_simple_type {
            Some(t) => FieldType::Simple(t),
            None if field_type == UNKNOWN_FIELDS_TYPE_NAME => FieldType::UnknownFields,
            None => FieldType::Custom {
                name: field_type.to_owned(),
            },
        }
    }

    fn is_unknown_field(&self) -> bool {
        matches!(self, FieldType::UnknownFields)
    }
}

pub trait CodeGenProvider<C: CodeGenContext> {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<C>>;
    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<C>>>;
    fn get_type_writer(&self) -> Box<dyn CustomTypeWriter<C>>;
    fn get_object_writer(&self) -> Box<dyn ObjectWriter<C>>;
    fn get_enum_writer(&self) -> Box<dyn EnumWriter<C>>;
    fn get_object_enum_writer(&self) -> Box<dyn ObjectEnumWriter<C>>;
}

pub trait CodeGenContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo>;
}

pub trait PreProcessor<C: CodeGenContext> {
    fn process(&self, definitions: HashMap<String, TypeInfo>) -> anyhow::Result<Box<C>>;
    // TODO: move this to extra type info within context?
    fn get_package_name(&self, definition: &Definition) -> anyhow::Result<String>;
}

pub trait PackageWriter<C: CodeGenContext> {
    fn write_package(
        &self,
        package: &str,
        types: &Vec<&TypeInfo>,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait ObjectEnumWriter<C: CodeGenContext> {
    fn pre_write_object_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectEnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn write_object_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &ObjectEnumValue,
        type_info: &ObjectEnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn post_write_object_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectEnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait EnumWriter<C: CodeGenContext> {
    fn pre_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &EnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn write_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &str,
        type_info: &EnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn post_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &EnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait CustomTypeWriter<C: CodeGenContext> {
    fn get_writer_for_type(
        &self,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<Box<dyn Write>>;

    fn pre_write_type(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait ObjectWriter<C: CodeGenContext> {
    fn pre_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &ObjectTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn write_field(
        &self,
        writer: &mut dyn Write,
        field: &ObjectField,
        type_info: &ObjectTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn post_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &ObjectTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}
