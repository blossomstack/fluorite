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
                assert!(output.contains("p_73616d706c652e636f6d6d6f6e.decodeChild(item, itemPath)"));
                assert!(output.contains("record[\"note\"] !== undefined"));
            }
        }
    }
}

#[test]
fn decoder_aliases_are_injective_and_empty_structs_need_no_local() {
    let schema = parse_strings_to_ir(&[
        "package a.b; struct Nested { count: u32 }",
        "package a_b; struct Flat { label: String }",
        "package consumer; use a.b.Nested; use a_b.Flat; struct Pair { nested: Nested, flat: Flat } struct Empty {}",
    ]).unwrap();
    for single in [false, true] {
        let fs = Arc::new(MemoryFileSystem::new());
        TsTemplateGenerator::new(
            TypeScriptOptions::new("/out".into())
                .with_single_file(single)
                .with_runtime_decoders(true),
            fs.clone(),
        )
        .generate_from_schema(&schema)
        .unwrap();
        let output = fs.get_string("/out/consumer/decoders.ts").unwrap();
        for alias in ["p_612e62", "p_615f62"] {
            assert!(output.contains(&format!("import * as {alias} from")));
            assert!(output.contains(&format!("{alias}.decode")));
        }
        let empty = output
            .split("export function decodeEmpty")
            .nth(1)
            .unwrap()
            .split("\n}")
            .next()
            .unwrap();
        assert!(empty.contains("Wire.object(value, path)"));
        assert!(!empty.contains("const record"));
    }
}
