use std::{collections::HashMap, io::Write};

use crate::definitions::{CustomType, Definition, Field};

use super::utils::get_type_name;

pub trait CodeGenProvider<C: CodeGenContext> {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<C>>;
    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<C>>>;
    fn get_type_writer(&self) -> Box<dyn CustomTypeWriter<C>>;
}

pub struct TypeInfo {
    pub type_def: CustomType,
    pub package: String,
}

impl TypeInfo {
    pub fn new(type_def: CustomType, package: String) -> Self {
        Self { type_def, package }
    }

    pub fn type_name(&self) -> String {
        get_type_name(&self.type_def)
    }

    pub fn is_enum(&self) -> bool {
        match &self.type_def {
            CustomType::Object { name: _, fields: _ } => false,
            CustomType::Enum { name: _, values: _ } => true,
        }
    }
}

pub trait CodeGenContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo>;
}

pub trait PreProcessor<C: CodeGenContext> {
    fn process(&self, definitions: &Vec<Definition>) -> anyhow::Result<Box<C>>;
}

pub trait PackageWriter<C: CodeGenContext> {
    fn write_package(
        &self,
        package: &str,
        types: &Vec<&TypeInfo>,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait CustomTypeWriter<C: CodeGenContext> {
    fn pre_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn write_enum_value(
        &self,
        writer: &mut dyn Write,
        value: &str,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn post_write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

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

    fn pre_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn write_field(
        &self,
        writer: &mut Box<dyn Write>,
        field: &Field,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;

    fn post_write_object(
        &self,
        writer: &mut Box<dyn Write>,
        type_info: &TypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}
