# Fluorite

[![Crates.io](https://img.shields.io/crates/v/fluorite)](https://crates.io/crates/fluorite)
[![docs.rs](https://img.shields.io/docsrs/fluorite)](https://docs.rs/fluorite/latest)
[![CI](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml/badge.svg)](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml)

Fluorite generates **Rust**, **TypeScript**, **Swift**, and **Go** code from a shared schema language. Define your types once in `.fl` files, then generate type-safe, serialization-ready code for all four languages.

All generated code uses **camelCase** as the JSON serialization format, ensuring consistent cross-language interoperability without any configuration.

## Quick Start

### 1. Install

```bash
# via Cargo
cargo install fluorite_codegen

# or via npm
npm install -D @blossomstack/fluorite-cli
```

### 2. Write a Schema

Create `schema.fl`:

```rust
package myapp;

struct User {
    id: String,
    name: String,
    email: Option<String>,
    active: bool,
}

enum Role {
    Admin,
    Member,
    Guest,
}
```

### 3. Generate Code

```bash
# Rust
fluorite rust --inputs schema.fl --output ./src/generated

# TypeScript
fluorite ts --inputs schema.fl --output ./src/generated

# Swift
fluorite swift --inputs schema.fl --output ./Sources/Generated

# Go
fluorite go --inputs schema.fl --output ./generated
```

Generation overwrites files but never deletes them, so a renamed or moved type leaves its old file behind. Clean first to make the output tree a pure function of your schema:

```bash
fluorite clean --output ./src/generated && fluorite ts --inputs schema.fl --output ./src/generated
```

That's it. You now have type-safe structs (Rust), interfaces (TypeScript), Codable types (Swift), and structs with JSON tags (Go), all with full serialization support.

Go output differs from the others in two ways worth knowing up front: every `.fl` package lands in a **single flat Go package** (Go forbids import cycles, and Fluorite already resolves type names globally), and the generated code imports **nothing outside the standard library**.

---

## The Fluorite IDL

Fluorite uses `.fl` files with a Rust-like syntax. Here's what you can express:

### Structs

```rust
/// A customer order
struct Order {
    order_id: String,
    total: f64,
    shipped: bool,
    notes: Option<String>,
}
```

Fields are automatically serialized as **camelCase** in JSON across all languages. No annotation needed &mdash; `order_id` becomes `"orderId"` in JSON.

**Rust output** &mdash; a `#[derive(Serialize, Deserialize)]` struct with `#[serde(rename_all = "camelCase")]`.

**TypeScript output** &mdash; an exported interface with camelCase field names.

**Swift output** &mdash; a `Codable` struct with camelCase properties and `CodingKeys`.

### Enums

```rust
enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}
```

**Rust** &mdash; a standard `enum` with serde derives. **TypeScript** &mdash; a string literal union type. **Swift** &mdash; a `String`-backed enum with `Codable` conformance.

### Tagged Unions

Fluorite uses **adjacently tagged** unions, producing consistent JSON across all languages:

```rust
#[type_tag = "type"]
#[content_tag = "value"]
union OrderEvent {
    Created(Order),
    StatusChanged(StatusChange),
    Cancelled,
}
```

This serializes as:
```json
{"type": "Created", "value": {"orderId": "...", "total": 42.0, ...}}
{"type": "Cancelled"}
```

**Rust output:**
```rust
#[serde(tag = "type", content = "value")]
pub enum OrderEvent {
    Created(Order),
    StatusChanged(StatusChange),
    Cancelled,
}
```

**TypeScript output:**
```typescript
export type OrderEvent =
  | { type: "Created"; value: Order }
  | { type: "StatusChanged"; value: StatusChange }
  | { type: "Cancelled" };
```

**Swift output:**
```swift
public enum OrderEvent: Codable, Equatable, Sendable {
    case created(Order)
    case statusChanged(StatusChange)
    case cancelled
    // Custom Codable implementation for adjacently tagged format
}
```

### Type Aliases

```rust
type OrderList = Vec<Order>;
type OrderMap = Map<String, Order>;
```

### Packages and Imports

Split schemas across files with dotted package names:

```rust
// common.fl
package myapp.common;

struct Address {
    street: String,
    city: String,
    country: String,
}
```

```rust
// users.fl
package myapp.users;

use myapp.common.Address;

struct User {
    name: String,
    home_address: Address,
}
```

### Doc Comments

Lines starting with `///` become doc comments in Rust, JSDoc comments in TypeScript, and documentation comments in Swift:

```rust
/// A user in the system.
/// Created during registration.
struct User {
    /// Unique identifier
    id: String,
}
```

Consecutive `///` lines form a single comment and are emitted one line per line, so a sentence that wraps in the schema stays intact in every generated language. The text is copied verbatim — nothing is escaped.

### Attributes

| Attribute | Applies to | Effect |
|-----------|-----------|--------|
| `#[rename = "name"]` | fields, variants | Rename in JSON |
| `#[alias = "alt"]` | fields | Accept alternate name during deserialization |
| `#[default]` | fields | Use `Default::default()` if missing |
| `#[skip_if_none]` | fields | Omit if `None` |
| `#[skip_if_default]` | fields | Omit if default value |
| `#[flatten]` | fields | Flatten nested struct into parent |
| `#[deprecated]` | types, fields | Mark as deprecated |
| `#[type_tag = "..."]` | unions | Tag field name (default: `"type"`) |
| `#[content_tag = "..."]` | unions | Content field name (default: `"value"`) |

> **Note:** All fields are serialized as camelCase by default. Use `#[rename = "..."]` to override individual fields when needed.

### Type Reference

| Fluorite Type | Rust | TypeScript | Swift |
|---------------|------|------------|-------|
| `String` | `String` | `string` | `String` |
| `bool` | `bool` | `boolean` | `Bool` |
| `i32`, `i64` | `i32`, `i64` | `number` | `Int32`, `Int64` |
| `u32`, `u64` | `u32`, `u64` | `number` | `UInt32`, `UInt64` |
| `f32`, `f64` | `f32`, `f64` | `number` | `Float`, `Double` |
| `Uuid` | `uuid::Uuid` | `string` | `UUID` |
| `Decimal` | `rust_decimal::Decimal` | `string` | `Decimal` |
| `Bytes` | `Vec<u8>` | `string` | `Data` |
| `Url` | `url::Url` | `string` | `URL` |
| `DateTime`, `DateTimeUtc`, `DateTimeTz` | `chrono` types | `string` | `Date` |
| `Date`, `Time`, `Duration` | `chrono` types | `string` | `String` |
| `Timestamp`, `TimestampMillis` | `i64` | `number` | `Date` |
| `Any` | `fluorite::Any` | `unknown` | `AnyCodable` |
| `Option<T>` | `Option<T>` | `T \| undefined` (optional field) | `T?` |
| `Vec<T>` | `Vec<T>` | `T[]` | `[T]` |
| `Map<K, V>` | `HashMap<K, V>` | `Record<K, V>` | `[K: V]` |

---

## Using Fluorite in a Rust Project

For Rust projects, the recommended approach is `build.rs` integration so types are generated at compile time.

> See the [examples/demo](examples/demo) project for a complete working example.

### 1. Add Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["serde_derive"] }
fluorite = "0.2"
derive-new = "0.7"

[build-dependencies]
fluorite_codegen = "0.2"
```

### 2. Create `build.rs`

```rust
use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(&out_dir)
        .with_any_type("serde_json::Value")
        .with_single_file(true);
    fluorite_codegen::compile_with_options(options, &["schemas/myapp.fl"]).unwrap();
}
```

### 3. Include Generated Code

```rust
mod myapp {
    include!(concat!(env!("OUT_DIR"), "/myapp/mod.rs"));
}

use myapp::User;
```

### Rust Options

```rust
RustOptions::new(output_dir)
    .with_single_file(true)              // All types in one mod.rs (default: true)
    .with_any_type("serde_json::Value")  // Map `Any` to this type
    .with_derives(vec!["Debug", "Clone"]) // Replace default derives
    .with_additional_derives(vec!["Hash"]) // Add extra derives
    .with_generate_new(true)             // Add derive_new::new (default: true)
    .with_visibility(Visibility::Public) // Type visibility (default: public)
```

---

## Using Fluorite in a TypeScript Project

### Via npm

```bash
npm install -D @blossomstack/fluorite-cli
```

Add to `package.json`:
```json
{
  "scripts": {
    "generate": "fluorite ts --inputs ./schemas/*.fl --output ./src/generated",
    "build": "npm run generate && tsc"
  }
}
```

> See the [examples/demo-ts](examples/demo-ts) project for a complete working example.

### Via Rust API

```rust
use fluorite_codegen::code_gen::ts::TypeScriptOptions;

let options = TypeScriptOptions::new("./src/generated")
    .with_single_file(true)
    .with_readonly(true);

fluorite_codegen::compile_ts_with_options(options, &["schemas/users.fl"]).unwrap();
```

### TypeScript Options

```rust
TypeScriptOptions::new(output_dir)
    .with_single_file(true)        // All types in index.ts (default: false)
    .with_any_type("any")          // Map `Any` to this type (default: "unknown")
    .with_readonly(true)           // Generate readonly properties (default: false)
    .with_package_name("custom")   // Override output directory name
```

---

## Using Fluorite for Swift

Generate Swift `Codable` types for iOS/macOS projects.

### Via CLI

```bash
fluorite swift --inputs schemas/users.fl --output ./Sources/Generated
```

### Via Rust API

```rust
use fluorite_codegen::code_gen::swift::SwiftOptions;

let options = SwiftOptions::new("./Sources/Generated")
    .with_single_file(false)
    .with_visibility(SwiftVisibility::Public);

fluorite_codegen::compile_swift_with_options(options, &["schemas/users.fl"]).unwrap();
```

### Swift Options

```rust
SwiftOptions::new(output_dir)
    .with_single_file(false)       // Separate file per type (default: false)
    .with_any_type("AnyCodable")   // Map `Any` to this type (default: "AnyCodable")
    .with_visibility(SwiftVisibility::Public)  // public, internal, or package
```

Generated Swift types conform to `Codable`, `Equatable`, and `Sendable`. Unions include a custom `Codable` implementation for adjacently tagged JSON format.

---

## CLI Reference

```
fluorite <COMMAND>

Commands:
  rust    Generate Rust code
  ts      Generate TypeScript code
  swift   Generate Swift code
```

### `fluorite rust`

| Flag | Default | Description |
|------|---------|-------------|
| `--inputs` | *required* | Input `.fl` files |
| `--output` | *required* | Output directory |
| `--single-file` | `true` | Put all types in one `mod.rs` |
| `--any-type` | `fluorite::Any` | Rust type for `Any` |
| `--derives` | | Custom derives (replaces defaults) |
| `--extra-derives` | | Additional derives |
| `--generate-new` | `true` | Generate `derive_new::new` |
| `--visibility` | `public` | Type visibility |

### `fluorite ts`

| Flag | Default | Description |
|------|---------|-------------|
| `--inputs` | *required* | Input `.fl` files |
| `--output` | *required* | Output directory |
| `--single-file` | `false` | Put all types in one `index.ts` |
| `--any-type` | `unknown` | TypeScript type for `Any` |
| `--readonly` | `false` | Generate `readonly` properties |
| `--package-name` | | Override output directory name |

### `fluorite swift`

| Flag | Default | Description |
|------|---------|-------------|
| `--inputs` | *required* | Input `.fl` files |
| `--output` | *required* | Output directory |
| `--single-file` | `false` | Separate file per type or all in one |
| `--any-type` | `AnyCodable` | Swift type for `Any` |
| `--visibility` | `public` | Access level: `public`, `internal`, `package` |

---

## Examples

The [examples/](examples/) directory contains complete projects:

- **[examples/demo](examples/demo)** &mdash; Rust project with `build.rs` integration, multi-package schemas, and cross-package imports
- **[examples/demo-ts](examples/demo-ts)** &mdash; TypeScript project using generated types from the same schemas

The demo schemas in [examples/demo/fluorite/](examples/demo/fluorite/) show real-world patterns:

| File | What it demonstrates |
|------|---------------------|
| `common.fl` | Shared types, `skip_if_none`, `Any` type |
| `users.fl` | Cross-package imports, tagged unions, type aliases |
| `orders.fl` | Multiple imports, enums, complex structs |
| `notifications.fl` | Unions with primitive variants (`PlainText(String)`) |

---

## Development

```bash
# Build
cargo build

# Run all tests
cargo test

# Run all CI checks (format, lint, test)
make all

# Run Rust <-> TypeScript interop tests
make interop-test
```

| Make target | Description |
|-------------|-------------|
| `make all` | Format check + lint + test |
| `make test` | Run all tests |
| `make fmt` | Format code |
| `make lint` | Run clippy |
| `make interop-test` | Rust/TypeScript round-trip tests |

## License

MIT
