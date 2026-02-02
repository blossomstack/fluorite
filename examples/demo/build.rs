use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    // Compile multiple .fl files with cross-package imports
    // Order matters: dependencies must come before dependents
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(out_dir).with_any_type("serde_json::Value");
    fluorite_codegen::compile_with_options(
        options,
        &[
            "fluorite/common.fl",
            "fluorite/users.fl",
            "fluorite/orders.fl",
            "fluorite/notifications.fl",
        ],
    )
    .unwrap();
}
