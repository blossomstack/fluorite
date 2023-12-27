use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    // use with explicit options:
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(out_dir).with_any_type("serde_json::Value");
    fluorite_codegen::compile_with_options(options, &["fluorite/demo.yaml"]).unwrap();

    // or to use default options:
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // fluorite_codegen::compile(&["fluorite/demo.yaml"], out_dir.as_str()).unwrap();
}
