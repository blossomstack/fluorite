// mod pre_processor;
// mod type_graph;
// pub use pre_processor::*;
// mod package_writer;
// pub use package_writer::*;
// mod type_writer;
// pub use type_writer::*;
// mod options;
// pub use options::*;
// mod context;
// pub use context::*;
//
// use super::abi::{CodeGenProvider, CustomTypeWriter, PackageWriter, PreProcessor};
//
// pub struct TsProvider {
//     options: TsOptions,
// }
//
// impl TsProvider {
//     pub fn new(options: TsOptions) -> Self {
//         Self { options }
//     }
// }
// impl CodeGenProvider<TsContext> for TsProvider {
//     fn get_pre_processor(&self) -> Box<dyn PreProcessor<TsContext>> {
//         Box::new(TsPreProcessor {
//             options: self.options.clone(),
//         })
//     }
//
//     fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<TsContext>>> {
//         Some(Box::new(TsPackageWriter {}))
//     }
//
//     fn get_type_writer(&self) -> Box<dyn CustomTypeWriter<TsContext>> {
//         Box::new(TsTypeWriter {})
//     }
// }
