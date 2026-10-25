use fluorite_codegen::code_gen::{
    fs::MemoryFileSystem,
    ts::{TsTemplateGenerator, TypeScriptOptions},
};
use fluorite_codegen::idl::parse_strings_to_ir;
use std::sync::Arc;

#[test]
fn decoders_are_opt_in_and_cover_cross_package_nested_types() {
    let schema = parse_strings_to_ir(&[
        "package sample.common; struct Child { count: u32 }",
        "package sample.main; use sample.common.Child; struct Parent { children: Vec<Child>, note: Option<String> }",
    ]).unwrap();
    for single in [false, true] {
        for enabled in [false, true] {
            let fs = Arc::new(MemoryFileSystem::new());
            TsTemplateGenerator::new(
                TypeScriptOptions::new("/out".into())
                    .with_single_file(single)
                    .with_runtime_decoders(enabled),
                fs.clone(),
            )
            .generate_from_schema(&schema)
            .unwrap();
            let output = fs.get_string("/out/sample/main/decoders.ts");
            assert_eq!(output.is_some(), enabled);
            if let Some(output) = output {
                assert!(output.contains("../common/decoders"));
                assert!(output.contains("p_sample_common.decodeChild(item, itemPath)"));
                assert!(output.contains("record[\"note\"] !== undefined"));
            }
        }
    }
}
