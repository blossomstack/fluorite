use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use crate::code_gen::abi::{PackageWriter, TypeInfo};

use super::RustCodeGenContext;

pub struct RustPackageWriter {}

impl PackageWriter<RustCodeGenContext> for RustPackageWriter {
    fn write_package(
        &self,
        package: &str,
        types: &Vec<&TypeInfo>,
        _context: &RustCodeGenContext,
    ) -> anyhow::Result<()> {
        let output_path = format!("/tmp/test_gen/{}/", package);
        create_dir_all(output_path.as_str())?;
        let package_file = format!("{}/mod.rs", output_path);
        let file = File::create(package_file)?;
        let mut writer = BufWriter::new(file);
        for type_info in types {
            writer.write_all(format!("mod {};\n", type_info.type_name()).as_bytes())?;
            writer.write_all(
                format!(
                    "pub use crate::{}::{}::*;\n",
                    package,
                    type_info.type_name()
                )
                .as_bytes(),
            )?;
        }
        Ok(())
    }
}
