use std::{
    io::{Write},
};

use crate::code_gen::abi::{PackageWriter, TypeInfo};

use super::RustContext;

pub struct RustPackageWriter {}

impl PackageWriter<RustContext> for RustPackageWriter {
    fn write_package(
        &self,
        package: &str,
        types: &Vec<&TypeInfo>,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let mut writer = context.write_to_mod_file(package, false)?;
        if !context.options.single_file {
            for type_info in types.iter().filter(|t| !t.is_object_enum_value()) {
                let mod_name = context.options.type_to_file_name(type_info.type_name());
                writer.write_all(format!("mod {};\n", mod_name).as_bytes())?;
                writer.write_all(
                    format!("pub use crate::{}::{}::*;\n", package, mod_name).as_bytes(),
                )?;
            }
        }
        Ok(())
    }
}
