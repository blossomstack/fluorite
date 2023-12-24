use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
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
        let output_path = format!("{}/{}/", context.options.output_dir, package);
        create_dir_all(output_path.as_str())?;
        let package_file = format!("{}/mod.rs", output_path);
        let file = File::create(package_file)?;
        let mut writer = BufWriter::new(file);
        for type_info in types.into_iter().filter(|t| !t.is_object_enum_value()) {
            let mod_name = context.options.type_to_file_name(type_info.type_name());
            writer.write_all(format!("mod {};\n", mod_name).as_bytes())?;
            writer
                .write_all(format!("pub use crate::{}::{}::*;\n", package, mod_name).as_bytes())?;
        }
        Ok(())
    }
}
