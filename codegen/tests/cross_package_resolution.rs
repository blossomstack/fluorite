//! Type references must resolve against the referencing file's own package and
//! its `use` imports — never by scanning every package for a matching bare name.
//!
//! Two packages are allowed to declare the same type name. Before resolution
//! was package-aware, a bare reference bound to whichever package a `HashMap`
//! scan reached first, so the same schema generated different code run to run.

use std::collections::HashMap;
use std::sync::Arc;

use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use fluorite_codegen::code_gen::ir::IRSchema;
use fluorite_codegen::code_gen::rust::{RustOptions, RustTemplateGenerator};
use fluorite_codegen::code_gen::ts::{TsTemplateGenerator, TypeScriptOptions};
use fluorite_codegen::idl::parse_strings_to_ir;

fn generate_rust(schema: &IRSchema) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    // One file per type, so assertions can name the file a reference lives in.
    let options = RustOptions::new("/output".to_string()).with_single_file(false);
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator
        .generate_from_schema(schema)
        .expect("Failed to generate");

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

fn generate_ts(schema: &IRSchema) -> HashMap<String, String> {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_string());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator
        .generate_from_schema(schema)
        .expect("Failed to generate");

    fs.files()
        .iter()
        .map(|(path, content)| (path.clone(), String::from_utf8_lossy(content).to_string()))
        .collect()
}

/// The two schemas that collide: each package declares its own `RequestFailed`.
const VENDOR_FL: &str = r#"
    package runtime_vendor;

    struct RequestFailed {
        vendor_code: i32,
    }
"#;

const RUNTIME_FL: &str = r#"
    package runtime;

    struct RequestFailed {
        reason: String,
    }

    union RuntimeEvent {
        Failed(RequestFailed),
    }
"#;

/// A union arm naming a type declared in its own package must bind to that
/// package, even when an unrelated package declares the same name.
#[test]
fn same_package_wins_over_identically_named_type_elsewhere() {
    let schema = parse_strings_to_ir(&[RUNTIME_FL, VENDOR_FL]).expect("Failed to parse");
    let output = generate_rust(&schema);

    let runtime = output
        .iter()
        .find(|(path, _)| path.contains("runtime_event"))
        .map(|(_, content)| content.clone())
        .expect("runtime_event.rs not generated");

    assert!(
        runtime.contains("Failed(crate::runtime::RequestFailed)"),
        "union arm bound to the wrong package:\n{runtime}"
    );
}

/// The same input must produce the same output every run. Resolution used to
/// depend on `HashMap` iteration order, which Rust reseeds per process — but a
/// single process shares one seed, so ordering is varied here by feeding the
/// files in both orders and by building the schema repeatedly.
#[test]
fn resolution_is_deterministic_across_repeated_builds() {
    let mut seen: Vec<String> = Vec::new();

    for _ in 0..25 {
        for inputs in [[RUNTIME_FL, VENDOR_FL], [VENDOR_FL, RUNTIME_FL]] {
            let schema = parse_strings_to_ir(&inputs).expect("Failed to parse");
            let output = generate_rust(&schema);
            let runtime = output
                .iter()
                .find(|(path, _)| path.contains("runtime_event"))
                .map(|(_, content)| content.clone())
                .expect("runtime_event.rs not generated");

            let arm = runtime
                .lines()
                .find(|l| l.contains("Failed("))
                .unwrap_or_default()
                .trim()
                .to_string();
            seen.push(arm);
        }
    }

    seen.sort();
    seen.dedup();
    assert_eq!(
        seen.len(),
        1,
        "same schema resolved differently across runs: {seen:?}"
    );
    assert!(
        seen[0].contains("crate::runtime::RequestFailed"),
        "union arm bound to the wrong package: {}",
        seen[0]
    );
}

/// An explicit `use` decides which package a name comes from when the
/// referencing package does not declare it itself.
#[test]
fn explicit_import_selects_the_package() {
    let a = r#"
        package pkg_a;

        struct Settings {
            a: String,
        }
    "#;
    let b = r#"
        package pkg_b;

        struct Settings {
            b: String,
        }
    "#;
    // Declares no Settings of its own; imports pkg_b's.
    let c = r#"
        package pkg_c;

        use pkg_b.Settings;

        struct Holder {
            settings: Settings,
        }
    "#;

    let schema = parse_strings_to_ir(&[a, b, c]).expect("Failed to parse");
    let output = generate_rust(&schema);

    let holder = output
        .iter()
        .find(|(path, _)| path.contains("holder"))
        .map(|(_, content)| content.clone())
        .expect("holder.rs not generated");

    assert!(
        holder.contains("crate::pkg_b::Settings"),
        "import ignored; field did not bind to pkg_b:\n{holder}"
    );
}

/// Two files importing the same bare name from different packages must each
/// get the package they asked for. This is the shape horsie's schemas already
/// have: `AgentSettings` exists in both `protocol.worker_server` and
/// `resources.worker`, imported separately by two different files.
#[test]
fn two_files_importing_the_same_name_get_different_packages() {
    let worker = r#"
        package resources.worker;

        struct AgentSettings {
            worker_field: String,
        }
    "#;
    let worker_server = r#"
        package protocol.worker_server;

        struct AgentSettings {
            server_field: String,
        }
    "#;
    let agent_server = r#"
        package protocol.agent_server;

        use protocol.worker_server.AgentSettings;

        struct AgentRequest {
            settings: AgentSettings,
        }
    "#;
    let client_server = r#"
        package protocol.client_server;

        use resources.worker.AgentSettings;

        struct ClientRequest {
            settings: AgentSettings,
        }
    "#;

    let schema = parse_strings_to_ir(&[worker, worker_server, agent_server, client_server])
        .expect("Failed to parse");
    let output = generate_rust(&schema);

    let agent = output
        .iter()
        .find(|(path, _)| path.contains("agent_request"))
        .map(|(_, c)| c.clone())
        .expect("agent_request.rs not generated");
    let client = output
        .iter()
        .find(|(path, _)| path.contains("client_request"))
        .map(|(_, c)| c.clone())
        .expect("client_request.rs not generated");

    assert!(
        agent.contains("crate::protocol::worker_server::AgentSettings"),
        "agent_server bound to the wrong package:\n{agent}"
    );
    assert!(
        client.contains("crate::resources::worker::AgentSettings"),
        "client_server bound to the wrong package:\n{client}"
    );
}

/// TypeScript resolves through the same IR, so it must agree with Rust.
#[test]
fn typescript_agrees_with_rust_on_the_owning_package() {
    let schema = parse_strings_to_ir(&[RUNTIME_FL, VENDOR_FL]).expect("Failed to parse");
    let output = generate_ts(&schema);

    let event = output
        .get("/output/runtime/runtimeEvent.ts")
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "runtimeEvent.ts not generated; got {:?}",
                output.keys().collect::<Vec<_>>()
            )
        });

    assert!(
        !event.contains("runtime_vendor"),
        "TypeScript imported RequestFailed from the wrong package:\n{event}"
    );
    assert!(
        event.contains("RequestFailed"),
        "expected a RequestFailed reference:\n{event}"
    );
}

/// A name that is neither declared locally nor imported must fail loudly.
/// Silently binding it to some other package is what caused the original bug.
#[test]
fn unimported_cross_package_reference_is_an_error() {
    let vendor = r#"
        package runtime_vendor;

        struct RequestFailed {
            vendor_code: i32,
        }
    "#;
    // No `use`, and no local RequestFailed.
    let runtime = r#"
        package runtime;

        struct RuntimeEvent {
            failure: RequestFailed,
        }
    "#;

    let err = parse_strings_to_ir(&[vendor, runtime])
        .expect_err("expected an error for an unimported cross-package reference");
    let msg = err.to_string();

    assert!(
        msg.contains("RequestFailed"),
        "error should name the unresolved type: {msg}"
    );
    assert!(
        msg.contains("runtime_vendor"),
        "error should point at the package that declares it: {msg}"
    );
}
