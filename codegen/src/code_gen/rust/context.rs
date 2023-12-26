use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{BufWriter, Write},
};

use crate::code_gen::abi::{CodeGenContext, TypeInfo, TypeName};

use super::RustOptions;
use anyhow::anyhow;

pub struct RustContext {
    pub types_dict: HashMap<String, TypeInfo>,
    pub options: RustOptions,
}

impl CodeGenContext for RustContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo> {
        &self.types_dict
    }

    fn get_writer_for_type(&self, type_info: &TypeInfo) -> anyhow::Result<Box<dyn Write>> {
        match self.options.single_file {
            true => self.write_to_mod_file(type_info.package(), true),
            _ => self.write_to_type_file(type_info),
        }
    }
}

impl RustContext {
    pub fn write_to_type_file(&self, type_info: &TypeInfo) -> anyhow::Result<Box<dyn Write>> {
        let type_name = type_info.type_name();
        let output_path = format!("{}/{}", self.options.output_dir, type_info.package());
        let output_file_name = format!(
            "{}/{}.rs",
            output_path,
            self.options.type_to_file_name(type_name)
        );
        let file = File::create(output_file_name)?;
        let writer = BufWriter::new(file);
        Ok(Box::new(writer))
    }
    pub fn write_to_mod_file(
        &self,
        package: &str,
        append_only: bool,
    ) -> anyhow::Result<Box<dyn Write>> {
        let output_path = format!("{}/{}/", self.options.output_dir, package);
        create_dir_all(output_path.as_str())?;
        let package_file = format!("{}/mod.rs", output_path);
        let file = match append_only {
            true => OpenOptions::new()
                .append(true)
                .create(true)
                .open(package_file)?,
            _ => File::create(package_file)?,
        };
        let writer = BufWriter::new(file);
        Ok(Box::new(writer))
    }
    pub fn get_fully_qualified_type_name(&self, type_name: &TypeName) -> anyhow::Result<String> {
        let full_type_name = match type_name {
            TypeName::Simple(t) => self.options.get_simple_type(t),
            TypeName::CustomType(name) => {
                let type_info = self
                    .types_dict
                    .get(name)
                    .ok_or_else(|| anyhow!("Cannot find custom type: {}", name))?;
                let package = type_info
                    .package()
                    .split('.')
                    .collect::<Vec<_>>()
                    .join("::");
                format!("crate::{}::{}", package, type_info.type_name())
            }
            TypeName::Any => "fluorite::Any".to_owned(),
        };
        Ok(full_type_name)
    }
}
