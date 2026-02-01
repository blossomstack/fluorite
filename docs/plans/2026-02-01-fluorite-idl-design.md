# Fluorite IDL Design

## Overview

This document describes the design for a custom IDL (Interface Definition Language) for Fluorite, replacing the current YAML schema format. The IDL uses Rust-like syntax for familiarity with the primary code generation target.

## Motivation

- **Better syntax**: YAML is verbose and hard to read for schema definitions
- **Type safety**: Enable compile-time validation and better tooling support
- **Industry standard feel**: Familiar syntax like protobuf/smithy but tailored to Fluorite's needs

## IDL Syntax

### Package Declaration

Every `.fl` file must start with a package declaration:

```rust
package orders;
```

### Imports

Import types from other packages using Rust's `use` syntax:

```rust
use users::User;
use common::types::{Address, PhoneNumber};
```

### Struct Definitions

Objects are defined using `struct`:

```rust
/// Documentation comment for the struct
#[rename_all = "camelCase"]
struct Order {
    id: u64,
    item: String,
    user: User,
    #[box]
    shipping: Option<Shipping>,
    #[rename = "order_type"]
    type_field: String,
}
```

### Enum Definitions

Simple enumerations:

```rust
enum Gender {
    Male,
    Female,
}

enum Status {
    #[rename = "ACTIVE"]
    Active,
    #[deprecated]
    Legacy,
}
```

### Union Definitions (Tagged Unions)

Discriminated unions for polymorphic types:

```rust
#[type_tag = "type"]
union Address {
    Empty,
    PostCode,
    AddressInfo,
}
```

### Type Aliases

List and Map types:

```rust
type OrderList = Vec<Order>;
type OrderMap = Map<String, Order>;
```

## Supported Types

### Primitives

| IDL Type | Description |
|----------|-------------|
| `String` | UTF-8 string |
| `bool` | Boolean |
| `i32`, `i64` | Signed integers |
| `u32`, `u64` | Unsigned integers |
| `f32`, `f64` | Floating point |

### Extended Primitives

| IDL Type | Description |
|----------|-------------|
| `Uuid` | UUID string |
| `Decimal` | Decimal number |
| `Bytes` | Binary data |
| `Url` | URL string |

### Temporal Types

| IDL Type | Description |
|----------|-------------|
| `DateTime` | Date and time |
| `DateTimeUtc` | UTC datetime |
| `DateTimeTz` | Datetime with timezone |
| `Date` | Date only |
| `Time` | Time only |
| `Duration` | Time duration |
| `Timestamp` | Unix timestamp (seconds) |
| `TimestampMillis` | Unix timestamp (milliseconds) |

### Collections

| IDL Type | Description |
|----------|-------------|
| `Option<T>` | Optional value |
| `Vec<T>` | List/array |
| `Map<K, V>` | Key-value map |

### Special Types

| IDL Type | Description |
|----------|-------------|
| `Any` | Dynamic value (serde_json::Value) |

## Attributes

### Type-Level Attributes

```rust
#[rename_all = "camelCase"]     // Rename all fields
#[deny_unknown_fields]          // Fail on unknown JSON fields
#[type_tag = "field_name"]      // Union discriminator field
#[union_style = "inline"]       // inline | extern
```

### Field-Level Attributes

```rust
#[rename = "jsonName"]          // JSON field name
#[alias = "oldName"]            // Alternate name for deserialize
#[default]                      // Use Default::default()
#[default = "value"]            // Specific default value
#[skip_if_none]                 // Don't serialize if None
#[skip_if_default]              // Don't serialize if default
#[flatten]                      // Flatten nested struct
#[deprecated]                   // Mark as deprecated
#[box]                          // Wrap in Box<T>
```

## Example: Complete Schema

Current YAML (`orders.yml`):

```yaml
configs:
  rust_package: "protocols.orders"
types:
  - name: Order
    type: Object
    fields:
      - name: id
        type: UInt64
      - name: item
        type: String
      - name: shipping
        type: Shipping
        optional: true
        configs:
          rust_type_wrapper: Box
```

Equivalent IDL (`orders.fl`):

```rust
package protocols::orders;

use protocols::users::User;

struct Order {
    id: u64,
    item: String,
    user: User,
    #[box]
    shipping: Option<Shipping>,
    #[rename = "order_type"]
    type_field: String,
}

struct Shipping {
    id: String,
    order: Order,
    address: Address,
}

#[type_tag = "type"]
union Address {
    Empty,
    PostCode,
    AddressInfo,
}

struct AddressInfo {
    first_line: String,
    second_line: String,
}

struct PostCode {
    code: String,
    order: Order,
    instruction: Any,
}

type OrderList = Vec<Order>;
type OrderMap = Map<String, Order>;

struct UserOrders {
    user: User,
    orders: OrderList,
}
```

## Architecture

### Parser Pipeline

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  .fl file   │────▶│   Lexer     │────▶│   Parser    │────▶│     AST      │
│  (source)   │     │  (logos)    │     │  (chumsky)  │     │              │
└─────────────┘     └─────────────┘     └─────────────┘     └──────┬───────┘
                                                                    │
                                                                    ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  Generated  │◀────│  Template   │◀────│     IR      │◀────│  AST → IR    │
│    Code     │     │  Generator  │     │  (existing) │     │  Converter   │
└─────────────┘     └─────────────┘     └─────────────┘     └──────────────┘
```

### Module Structure

```
codegen/src/
├── idl/
│   ├── mod.rs           # Public API: parse_file(), parse_string()
│   ├── lexer.rs         # Token definitions using logos
│   ├── parser.rs        # Grammar rules using chumsky
│   ├── ast.rs           # AST type definitions
│   └── ast_to_ir.rs     # Convert AST to existing IR types
```

### Libraries

- **logos**: Lexer generator - fast, derive-macro based tokenization
- **chumsky**: Parser combinator library - good error messages, Rust-idiomatic

### Integration Points

The IDL parser produces the same IR types (`codegen/src/code_gen/ir/`) that the YAML parser produces. This means:

1. No changes to template generators
2. No changes to code generation logic
3. Both Rust and TypeScript output work automatically

### CLI Integration

```bash
# Auto-detect format by extension
fluorite rust --inputs schema.fl --output ./src

# TypeScript generation
fluorite ts --inputs schema.fl --output ./src/generated
```

### build.rs Integration

```rust
// Works seamlessly
fluorite_codegen::compile(&["schema.fl"]).unwrap();
```

## Migration Strategy

1. Implement `.fl` parser with full feature parity
2. Create `.fl` versions of all example/test schemas
3. Verify identical output for both formats
4. Deprecate YAML support
5. Remove YAML parser in future version

## Testing Strategy

### Unit Tests

- **Lexer tests**: Each token type tokenizes correctly
- **Parser tests**: Valid syntax produces correct AST
- **Error tests**: Invalid syntax produces helpful error messages with line/column
- **AST→IR tests**: Conversion produces expected IR

### Integration Tests

Create `.fl` equivalents of existing YAML test files:

```
examples/
├── orders.fl       # Convert from orders.yml
├── users.fl        # Convert from users.yml
```

### E2E Acceptance Criteria

1. `examples/orders.fl` parses and produces same IR as `orders.yml`
2. `examples/users.fl` parses and produces same IR as `users.yml`
3. Rust code generation from `.fl` produces identical output
4. TypeScript code generation from `.fl` produces identical output
5. Syntax errors include accurate line/column information
6. All YAML test schemas have working `.fl` equivalents

## Error Messages

Good error messages are critical for developer experience:

```
error: expected type after field name
  --> schema.fl:15:12
   |
15 |     name: ,
   |           ^ expected type, found ','
```

The parser will track source spans for all AST nodes to enable precise error reporting.
