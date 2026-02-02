# Fluorite

[![Crates.io](https://img.shields.io/crates/v/fluorite)](https://crates.io/crates/fluorite)
[![docs.rs](https://img.shields.io/docsrs/fluorite)](https://docs.rs/fluorite/latest)
[![CI](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml/badge.svg)](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml)

Fluorite is a code generation tool that generates Rust and TypeScript code from .fl (Fluorite IDL) schema definitions. It's an IDL/schema-based code generator focused on serialization/deserialization patterns.

## Features

- **Dual Language Support**: Generate both Rust and TypeScript code from the same schema
- **Fluorite IDL**: Native Rust-like syntax for schema definitions (.fl files)
- **Rich Type System**: Objects, enums, tagged unions, lists, maps, and primitives
- **Serde Integration**: Built-in support for serialization/deserialization
- **Cargo Integration**: Use via `build.rs` for seamless Rust project integration
- **CLI Tool**: Command-line interface for code generation

## Schema Definition

Fluorite uses `.fl` files with Rust-like syntax:

```rust
/// Package declaration
package users;

/// Import from other .fl files
use orders::Order;

/// Struct definition
struct User {
    /// Unique identifier
    id: Uuid,
    /// User's name
    name: String,
    /// Optional email
    email: Option<String>,
    /// Account status
    status: UserStatus,
}

/// Enum definition
enum UserStatus {
    Active,
    Inactive,
}

/// Tagged union for polymorphic types
union UserEvent {
    Created(User),
    Deleted(Uuid),
}

/// Type alias
type UserList = Vec<User>;
```

See [examples/users.fl](examples/users.fl) and [examples/orders.fl](examples/orders.fl) for complete examples.

## Using `fluorite` as a CLI

### Installation

```bash
cargo install fluorite_codegen
```

### Generate Rust Code

```bash
# Generate Rust from .fl files
fluorite rust --inputs examples/users.fl examples/orders.fl --output ./src/generated

# Single file output
fluorite rust --inputs examples/users.fl --output ./src/generated --single-file

# With cargo
 cargo run --package fluorite_codegen --bin fluorite -- rust \
  --inputs examples/users.fl examples/orders.fl \
  --output ./src/generated
```

### Generate TypeScript Code

```bash
# Generate TypeScript from .fl files
fluorite ts --inputs examples/users.fl examples/orders.fl --output ./src/generated

# Single file output
fluorite ts --inputs examples/users.fl --output ./src/generated --single-file

# With cargo
cargo run --package fluorite_codegen --bin fluorite -- ts \
  --inputs examples/users.fl \
  --output ./src/generated
```

### CLI Help

```bash
$ fluorite --help
Generate Rust and TypeScript code from Fluorite IDL schemas.

Usage: fluorite <COMMAND>

Commands:
  rust  Generate Rust code
  ts    Generate TypeScript code
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Using `fluorite` in a Cargo Project

> See the [demo project](examples/demo) for a complete working example.

### 1. Add Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["serde_derive"] }
fluorite = "0.1"
derive-new = "0.7"

[build-dependencies]
fluorite_codegen = "0.1"
```

### 2. Create a `build.rs` File

```rust
use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(&out_dir)
        .with_any_type("serde_json::Value")
        .with_single_file(true);
    fluorite_codegen::compile_with_options(options, &["schemas/demo.fl"]).unwrap();
}
```

### 3. Include Generated Code

In your `lib.rs` or `main.rs`:

```rust
mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}

// Use the generated types
use demo::User;
```

### Rust Configuration Options

```rust
RustOptions::new(output_dir)
    .with_single_file(true)              // All types in mod.rs
    .with_any_type("serde_json::Value")  // Custom Any type
    .with_derives(vec!["Debug", "Clone"]) // Custom derives
    .with_additional_derives(vec!["PartialEq"]) // Extra derives
    .with_generate_new(true)             // Generate derive_new::new
    .with_visibility(Visibility::Public) // Visibility level
```

## TypeScript Code Generation

### Using Programmatically

```rust
use fluorite_codegen::code_gen::ts::TypeScriptOptions;

let options = TypeScriptOptions::new("./src/generated")
    .with_single_file(true)
    .with_readonly(true);

fluorite_codegen::compile_ts_with_options(options, &["schemas/users.fl"]).unwrap();
```

### TypeScript Configuration Options

```rust
TypeScriptOptions::new(output_dir)
    .with_single_file(true)        // All types in index.ts
    .with_any_type("any")          // Custom Any type mapping
    .with_readonly(true)           // Generate readonly properties
    .with_package_name("custom")   // Override output package directory
```

### Type Mapping (Fluorite → TypeScript)

| Fluorite Type | TypeScript |
|---------------|------------|
| String, DateTime, DateTimeUtc, DateTimeTz, Date, Time, Duration | `string` |
| Bool | `boolean` |
| Int32, Int64, UInt32, UInt64, Float32, Float64, Timestamp, TimestampMillis | `number` |
| UUID, Decimal, Bytes, Url | `string` |
| Any | `unknown` |
| List<T> | `T[]` |
| Map<K, V> | `Record<K, V>` |
| Optional field | `field?: Type` |

## Supported Types

### Basic Primitives
- `String`, `Bool`
- `Int32`, `Int64`, `UInt32`, `UInt64`
- `Float32`, `Float64`

### Extended Primitives
- `Uuid`, `Decimal`, `Bytes`, `Url`
- `DateTime`, `DateTimeUtc`, `DateTimeTz`
- `Date`, `Time`, `Duration`
- `Timestamp`, `TimestampMillis`

### Collections
- `Vec<T>` / `List<T>`
- `Map<K, V>`

### Custom Types
- `struct` - Object definitions
- `enum` - Enum definitions
- `union` - Polymorphic tagged unions
- `type` - Type aliases
- `Option<T>` - Optional fields
- `Any` - Dynamic JSON-like values

### Attributes
- `#[rename = "value"]` - Field/type renaming
- `#[rename_all = "camelCase"]` - Case conversion
- `#[alias = "alt_name"]` - Deserialization aliases
- `#[default]` - Default values
- `#[skip_if_none]`, `#[skip_if_default]` - Conditional serialization
- `#[flatten]` - Flatten nested structures
- `#[deprecated]` - Deprecation notices

## Development

### Build Commands

```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run tests for specific package
cargo test --package fluorite_codegen    # codegen library and CLI
cargo test --package fluorite            # runtime library
```

### Make Commands

| Command | Description |
|---------|-------------|
| `make all` | Run format check, lint, and tests |
| `make build` | Build the project |
| `make release` | Build in release mode |
| `make test` | Run all tests |
| `make fmt` | Format code |
| `make fmt-check` | Check code formatting |
| `make lint` | Run clippy lints |
| `make check` | Run cargo check |
| `make clean` | Clean build artifacts |

## Architecture

Fluorite uses a plugin-based code generation system:

```
CodeGenProvider (trait)
├── PreProcessor        # Parse definitions → type metadata
├── PackageWriter       # Write package module files
├── ObjectWriter        # Write struct definitions
├── EnumWriter          # Write enum definitions
├── UnionWriter         # Write polymorphic tagged unions
├── ListWriter          # Write list/vector types
└── MapWriter           # Write map types
```

The **RustProvider** implements all traits for Rust code generation. The **TsTemplateGenerator** provides full TypeScript code generation using the same intermediate representation (IR) layer.

### Key Components

1. **IDL Parser** (`codegen/src/idl/`) - Lexer and parser for .fl files using `logos` and `chumsky`
2. **IR Layer** (`codegen/src/code_gen/ir/`) - Language-agnostic type representation
3. **Validation** (`codegen/src/code_gen/validation/`) - Schema validation
4. **Templates** (`codegen/templates/`) - Askama templates for code generation
5. **FileSystem Abstraction** (`codegen/src/code_gen/fs/`) - Testable I/O operations

## License

This project is licensed under the MIT License.
