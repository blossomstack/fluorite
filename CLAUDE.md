# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fluorite is a Rust code generation tool that generates Rust (and planned TypeScript) code from YAML schema definitions. It's an IDL/schema-based code generator focused on serialization/deserialization patterns with serde.

## Build Commands

```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run tests for specific package
cargo test --package fluorite_codegen    # codegen library and CLI
cargo test --package fluorite            # runtime library

# Run the CLI
cargo run --package fluorite_codegen --bin fluorite -- rust --inputs file.yaml --output ./src --single-file true
```

## Workspace Structure

This is a Cargo workspace with three members:

- **codegen/** (`fluorite_codegen`): The code generation library and `fluorite` CLI binary
- **runtime/** (`fluorite`): Runtime support library providing the `Any` type for dynamic values
- **examples/demo/**: Example project demonstrating build.rs integration

## Architecture

### Plugin-Based Code Generation

The code generator uses a trait-based plugin system defined in `codegen/src/code_gen/abi.rs`:

```
CodeGenProvider (trait)
├── PreProcessor        # Parse definitions → type metadata
├── PackageWriter       # Write package module files
├── ObjectWriter        # Write struct definitions
├── EnumWriter          # Write enum definitions
├── UnionWriter    # Write polymorphic tagged unions
├── ListWriter          # Write list/vector types
└── MapWriter           # Write map types
```

The **RustProvider** (`codegen/src/code_gen/rust/`) implements all traits for Rust code generation. TypeScript generation exists as a stub in `codegen/src/code_gen/ts/`.

### Key Files

- `codegen/src/main.rs` - CLI entry point
- `codegen/src/code_gen/generator.rs` - Main code generation orchestrator
- `codegen/src/code_gen/rust/provider.rs` - Rust-specific code generation
- `codegen/src/definitions/` - Schema definition types (auto-generated from bootstrap)
- `runtime/src/lib.rs` - The `Any` enum for dynamic JSON-like values

### Schema Format

YAML schemas define types with this structure:
```yaml
configs:
  rust_package: "package.name"
types:
  - name: TypeName
    type: Object|Enum|Union|List|Map
    fields: [...]      # for Object
    values: [...]      # for Enum
    type_tag: "field"  # for Union (tagged union discriminator)
    config:
      rust_type_wrapper: Box
      union_style: Inline|Extern
      rename: "json_name"
```

### Supported Types

- **Primitives:** String, Bool, DateTime, UInt32, UInt64, Int32, Int64, Float32, Float64
- **Collections:** List, Map
- **Custom:** Object (struct), Enum, Union (polymorphic tagged union), Any
- **Modifiers:** Optional fields, field renaming, type wrappers (Box)

## Build.rs Integration

Projects use fluorite via build scripts:
```rust
use fluorite_codegen::code_gen::rust::RustOptions;

let out_dir = std::env::var("OUT_DIR").unwrap();
let options = RustOptions::new(out_dir).with_any_type("serde_json::Value");
fluorite_codegen::compile_with_options(options, &["schema.yaml"]).unwrap();
```

Then include generated code:
```rust
mod generated {
    include!(concat!(env!("OUT_DIR"), "/schema/mod.rs"));
}
```

## Testing Examples

The `examples/orders.yml` and `examples/users.yml` files are used by tests in `codegen/tests/rust_code_gen.rs` to verify code generation output.
