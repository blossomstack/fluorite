fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // let options = fluorite::RustOptions::new();
    // options.compile(&["fluorite/demo.yaml"], out_dir.as_str())
    // fluorite::complile_with_options(options)?
    // or to use default options:
    fluorite_codegen::compile(&["fluorite/demo.yaml"], out_dir.as_str()).unwrap();
}
