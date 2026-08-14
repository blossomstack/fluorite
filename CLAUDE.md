# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fluorite is a code generation tool that generates Rust and TypeScript code from YAML schema definitions. It's an IDL/schema-based code generator focused on serialization/deserialization patterns with serde.

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

## Code Quality Verification

**IMPORTANT:** Always verify code changes by running the same checks as GitHub Actions before considering work complete. Use the Makefile targets:

```bash
# Run all CI checks (format check, lint, test)
make all

# Or run individual checks:
make fmt-check    # Format check
make lint         # Clippy linting
make test         # Run all tests
```

All checks must pass before submitting changes.

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

The **RustProvider** (`codegen/src/code_gen/rust/`) implements all traits for Rust code generation. The **TsTemplateGenerator** (`codegen/src/code_gen/ts/`) provides full TypeScript code generation using the same IR layer.

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
    configs:
      rust_type_wrapper: Box
      rename: "json_name"
```

### Supported Types

- **Basic Primitives:** String, Bool, DateTime, UInt32, UInt64, Int32, Int64, Float32, Float64
- **Extended Primitives:** UUID, Decimal, Bytes, Url, Timestamp, TimestampMillis, DateTimeUtc, DateTimeTz, Date, Time, Duration
- **Collections:** List, Map
- **Custom:** Object (struct), Enum, Union (polymorphic tagged union), Any
- **Modifiers:** Optional fields, field renaming, type wrappers (Box)
- **Serde Features:** rename_all, alias, default, skip_if_none, skip_if_default, flatten, deny_unknown_fields
- **Documentation:** description (doc comments), deprecated annotation

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

The demo projects in `examples/demo/` (Rust) and `examples/demo-ts/` (TypeScript) demonstrate multi-package .fl files with cross-package imports. These are used by the e2e tests to verify code generation output.

## New Template-Based Architecture (v2)

The code generator now uses a template-based approach with askama:

### Key Components

1. **IR (Intermediate Representation)** - `codegen/src/code_gen/ir/`
   - Language-agnostic representation of types
   - `IRBuilder` converts YAML definitions to IR
   - Separates parsing from code generation

2. **Validation** - `codegen/src/code_gen/validation/`
   - Validates schemas before generation
   - Detects: unknown types, duplicate types, empty types

3. **Templates** - `codegen/templates/rust/`
   - Askama templates for Rust code output
   - Compile-time checked for correctness

4. **FileSystem Abstraction** - `codegen/src/code_gen/fs/`
   - `FileSystem` trait for I/O operations
   - `MemoryFileSystem` for testing
   - `RealFileSystem` for production

5. **RustTemplateGenerator** - `codegen/src/code_gen/rust/template_generator.rs`
   - Main entry point for Rust code generation
   - Uses IR + Validation + Templates

### Configuration Options

```rust
RustOptions::new(output_dir)
    .with_single_file(true)           // All types in mod.rs
    .with_any_type("serde_json::Value") // Custom Any type
    .with_derives(vec!["Debug", ...])  // Custom derives
    .with_additional_derives(vec![...]) // Extra derives
    .with_generate_new(true)           // derive_new::new
    .with_visibility(Visibility::Public)
```

### Adding a New Language

1. Create `codegen/templates/<lang>/` with templates
2. Create `codegen/src/code_gen/<lang>/template_generator.rs`
3. Implement type formatting and FQN resolution for the language
4. Add CLI subcommand

## Fluorite IDL (.fl files)

Fluorite now supports a native IDL format with Rust-like syntax as an alternative to YAML.

### IDL Syntax Example

```rust
/// User management types
package users;

use orders::Order;

/// Represents a user in the system
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

enum UserStatus {
    Active,
    Inactive,
}

/// User lifecycle events (adjacently tagged union)
#[type_tag = "type"]
union UserEvent {
    Created(User),
    Updated(User),
    Deleted,
    StatusChanged(UserStatusChange),
}

type UserList = Vec<User>;
```

### IDL Features

- **Package declaration**: `package name;`
- **Imports**: `use path::Type;` (supports dotted paths: `use demo.common.Address;`). Required for any cross-package reference: a bare name resolves to the file's own package, then to that file's imports, and is otherwise an error. Two packages may declare the same type name.
- **Structs**: `struct Name { fields }`
- **Enums**: `enum Name { Variants }`
- **Unions**: Adjacently tagged unions (see Union Design below)
- **Type aliases**: `type Name = Vec<T>;`
- **Doc comments**: `/// Description`
- **Attributes**: `#[rename = "value"]`, `#[type_tag = "type"]`, `#[content_tag = "value"]`
- **Generic types**: `Option<T>`, `Vec<T>`, `Map<K, V>`

### Union Design (Adjacently Tagged)

Fluorite uses **adjacently tagged** union format for both Rust and TypeScript, providing a simple and consistent serialization format.

#### IDL Syntax
```rust
/// Event types (adjacently tagged: {type: "...", value: ...})
#[type_tag = "type"]           // Tag field name (default: "type")
#[content_tag = "value"]       // Content field name (default: "value")
union UserEvent {
    Created(User),             // Newtype variant: {type: "Created", value: User}
    Updated(User),             // Newtype variant: {type: "Updated", value: User}
    Deleted,                   // Unit variant: {type: "Deleted"}
    StatusChanged(StatusChange),
}
```

#### Rust Output
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum UserEvent {
    Created(User),
    Updated(User),
    Deleted,
    StatusChanged(StatusChange),
}
```

#### TypeScript Output
```typescript
export type UserEvent =
  | { type: "Created"; value: User }
  | { type: "Updated"; value: User }
  | { type: "Deleted" }
  | { type: "StatusChanged"; value: StatusChange };
```

#### JSON Serialization Examples
```json
// Unit variant (no data)
{"type": "Deleted"}

// Newtype variant (wraps a value)
{"type": "Created", "value": {"id": "user-001", "name": "John"}}

// Primitive values work too
{"type": "PlainText", "value": "Hello world!"}
```

### Type Mapping (IDL → IR)

| IDL Type | IR Primitive |
|----------|-------------|
| `String` | `String` |
| `bool` | `Bool` |
| `i32` | `Int32` |
| `i64` | `Int64` |
| `u32` | `UInt32` |
| `u64` | `UInt64` |
| `f32` | `Float32` |
| `f64` | `Float64` |
| `Uuid` | `UUID` |
| `Decimal` | `Decimal` |
| `Bytes` | `Bytes` |
| `Url` | `Url` |
| `DateTime` | `DateTime` |
| `DateTimeUtc` | `DateTimeUtc` |
| `DateTimeTz` | `DateTimeTz` |
| `Date` | `Date` |
| `Time` | `Time` |
| `Duration` | `Duration` |
| `Timestamp` | `Timestamp` |
| `TimestampMillis` | `TimestampMillis` |
| `Any` | `Any` |

### Using .fl Files with CLI

```bash
# Generate Rust from .fl files
cargo run --package fluorite_codegen --bin fluorite -- rust \
  --inputs examples/users.fl examples/orders.fl \
  --output ./src/generated

# Generate TypeScript from .fl files
cargo run --package fluorite_codegen --bin fluorite -- ts \
  --inputs examples/users.fl \
  --output ./src/generated
```

### IDL Module Structure

- `codegen/src/idl/lexer.rs` - Tokenizer using `logos`
- `codegen/src/idl/parser.rs` - Parser using `chumsky`
- `codegen/src/idl/ast.rs` - AST type definitions
- `codegen/src/idl/ast_to_ir.rs` - AST to IR converter
- `codegen/src/idl/mod.rs` - Public API (`parse_string`, `parse_file`, `parse_to_ir`)

### Example Files

The demo project (`examples/demo/fluorite/`) contains multi-file .fl examples:
- `common.fl` - Shared types (Address, ApiResponse, Pagination)
- `users.fl` - User management types, imports from common
- `orders.fl` - Order management types, imports from common and users
- `notifications.fl` - Notification types with adjacently tagged unions

## TypeScript Code Generation

TypeScript generation is available via the `ts` subcommand:

```bash
# Generate TypeScript from YAML schemas
cargo run --package fluorite_codegen --bin fluorite -- ts \
  --inputs schemas/orders.yaml schemas/users.yaml \
  --output ./src/generated \
  --single-file false

# Or via npm package (after publishing)
npx @blossomstack/fluorite-cli ts --inputs ./schemas/*.yaml --output ./src/generated
```

### TypeScript-Specific Files

- `codegen/src/code_gen/ts/` - TypeScript generator implementation
- `codegen/templates/ts/` - Askama templates for TypeScript output
- `npm/fluorite-cli/` - npm package for easy integration

### Type Mapping (YAML → TypeScript)

| YAML Type | TypeScript |
|-----------|-----------|
| String, DateTime, DateTimeUtc, DateTimeTz, Date, Time, Duration | `string` |
| Bool | `boolean` |
| Int32, Int64, UInt32, UInt64, Float32, Float64, Timestamp, TimestampMillis | `number` |
| UUID, Decimal, Bytes, Url | `string` |
| Any | `unknown` |
| List<T> | `T[]` |
| Map<K, V> | `Record<K, V>` |
| Optional field | `field?: Type` |

### TypeScript Configuration Options

```rust
TypeScriptOptions::new(output_dir)
    .with_single_file(true)           // All types in index.ts
    .with_any_type("any")             // Custom Any type mapping
    .with_readonly(true)              // Generate readonly properties
    .with_package_name("custom")      // Override output package directory
```

### Design Document

See `docs/plans/2026-02-01-production-ready-features-design.md` for full design details.
