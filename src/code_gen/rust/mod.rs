mod pre_processor;
pub use pre_processor::*;
mod package_writer;
pub use package_writer::*;
mod type_writer;
pub use type_writer::*;
mod options;
pub use options::*;
mod context;
pub use context::*;

use super::abi::{CodeGenProvider, CustomTypeWriter, PackageWriter, PreProcessor};

pub struct RustProvider {
    options: RustOptions,
}

impl RustProvider {
    pub fn new(options: RustOptions) -> Self {
        Self { options }
    }
}
impl CodeGenProvider<RustContext> for RustProvider {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<RustContext>> {
        Box::new(RustPreProcessor {
            options: self.options.clone(),
        })
    }

    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<RustContext>>> {
        Some(Box::new(RustPackageWriter {}))
    }

    fn get_type_writer(&self) -> Box<dyn CustomTypeWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_object_writer(&self) -> Box<dyn super::abi::ObjectWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_enum_writer(&self) -> Box<dyn super::abi::EnumWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_object_enum_writer(&self) -> Box<dyn super::abi::ObjectEnumWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_list_writer(&self) -> Box<dyn super::abi::ListWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_map_writer(&self) -> Box<dyn super::abi::MapWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }
}
