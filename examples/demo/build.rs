use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    // Compile all .fl files under the fluorite/ directory
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(out_dir).with_any_type("serde_json::Value");
    fluorite_codegen::compile_with_options(options, &["fluorite/"]).unwrap();
}
