# Production-Ready Features for Fluorite

**Date:** 2026-02-01
**Status:** Implementation Plan Ready
**Implementation Plan:** [2026-02-01-production-ready-features-impl-plan.md](./2026-02-01-production-ready-features-impl-plan.md)
**Target:** API contracts and event/message schemas

## Overview

This design adds the data types, serde features, and documentation support needed to make Fluorite production-ready for API contracts and event/message schema code generation.

## Design Principles

1. **Portable-first** - Core schema features work across all target languages (Rust, TypeScript, Swift)
2. **Language-specific extensions** - Non-portable features live under language namespaces
3. **Backward compatible** - Existing schemas continue to work

---

## 1. New Primitive Types

### Time/Date Family

| Type | Rust Mapping | Description |
|------|--------------|-------------|
| `Timestamp` | `i64` | Unix epoch seconds |
| `TimestampMillis` | `i64` | Unix epoch milliseconds |
| `DateTimeUtc` | `chrono::DateTime<Utc>` | UTC instant, ISO 8601 |
| `DateTimeTz` | `chrono::DateTime<FixedOffset>` | Preserves timezone offset |
| `DateTime` | `chrono::NaiveDateTime` | No timezone (backward compat) |
| `Date` | `chrono::NaiveDate` | Date only |
| `Time` | `chrono::NaiveTime` | Time only |
| `Duration` | `chrono::Duration` | Time span |

### Other Primitives

| Type | Rust Mapping | Description |
|------|--------------|-------------|
| `UUID` | `uuid::Uuid` | Universally unique identifier |
| `Decimal` | `rust_decimal::Decimal` | Precise decimal (money, etc.) |
| `Bytes` | `Vec<u8>` | Binary data (base64 in JSON) |
| `Url` | `url::Url` | URL/URI |

### Cross-Language Mapping

| Fluorite Type | Rust | TypeScript | Swift |
|---------------|------|------------|-------|
| `UUID` | `uuid::Uuid` | `string` | `UUID` |
| `Decimal` | `rust_decimal::Decimal` | `string` | `Decimal` |
| `Bytes` | `Vec<u8>` | `string` (base64) | `Data` |
| `Timestamp` | `i64` | `number` | `Int64` |
| `DateTimeUtc` | `chrono::DateTime<Utc>` | `string` | `Date` |
| `Url` | `url::Url` | `string` | `URL` |

### Required Crate Dependencies

Users must add the relevant crates to their `Cargo.toml`:

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde"] }
rust_decimal = { version = "1.0", features = ["serde"] }
url = { version = "2.0", features = ["serde"] }
```

---

## 2. Serde Features

### Portable Features (all languages)

#### Field-Level

| Feature | YAML Syntax | Rust Output |
|---------|-------------|-------------|
| Rename | `rename: "jsonName"` | `#[serde(rename = "jsonName")]` |
| Alias | `alias: ["old", "legacy"]` | `#[serde(alias = "old", alias = "legacy")]` |
| Default | `default: "value"` | `#[serde(default = "...")]` |

#### Type-Level

| Feature | YAML Syntax | Rust Output |
|---------|-------------|-------------|
| Rename all | `rename_all: camelCase` | `#[serde(rename_all = "camelCase")]` |

Supported `rename_all` values: `camelCase`, `snake_case`, `PascalCase`, `SCREAMING_SNAKE_CASE`, `kebab-case`

### Rust-Specific Features

These live under a `rust:` namespace in the schema.

#### Field-Level

| Feature | YAML Syntax | Rust Output |
|---------|-------------|-------------|
| Skip if none | `rust.skip_if_none: true` | `#[serde(skip_serializing_if = "Option::is_none")]` |
| Skip if default | `rust.skip_if_default: true` | `#[serde(skip_serializing_if = "is_default")]` |
| Flatten | `rust.flatten: true` | `#[serde(flatten)]` |

#### Type-Level

| Feature | YAML Syntax | Rust Output |
|---------|-------------|-------------|
| Deny unknown | `rust.deny_unknown_fields: true` | `#[serde(deny_unknown_fields)]` |

---

## 3. Documentation Support

### Field and Type Descriptions

```yaml
- name: Order
  type: Object
  description: Represents a customer order
  fields:
    - name: id
      type: UUID
      description: Unique order identifier
```

**Generated Rust:**
```rust
/// Represents a customer order
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: Uuid,
}
```

**Generated TypeScript:**
```typescript
/** Represents a customer order */
export interface Order {
  /** Unique order identifier */
  id: string;
}
```

### Deprecation

```yaml
- name: legacy_field
  type: String
  deprecated: true
  description: Use new_field instead
```

**Generated Rust:**
```rust
#[deprecated(note = "Use new_field instead")]
pub legacy_field: String,
```

---

## 4. Schema Examples

### Full Example

```yaml
configs:
  rust_package: "api.orders"

types:
  - name: CreateOrderRequest
    type: Object
    description: Request payload for creating a new order
    config:
      rename_all: camelCase
      rust:
        deny_unknown_fields: true
    fields:
      - name: order_id
        type: UUID
        description: Client-generated order ID for idempotency
      - name: amount
        type: Decimal
        description: Order total in customer's currency
      - name: currency
        type: String
        default: "USD"
      - name: created_at
        type: DateTimeUtc
      - name: ttl
        type: Duration
        optional: true
        rust:
          skip_if_none: true
      - name: metadata
        type: OrderMetadata
        optional: true
        rust:
          flatten: true

  - name: OrderMetadata
    type: Object
    fields:
      - name: source
        type: String
        alias: ["origin", "src"]
      - name: correlation_id
        type: UUID
        optional: true

  - name: OrderEvent
    type: Object
    description: Event emitted when order state changes
    fields:
      - name: event_id
        type: UUID
      - name: timestamp
        type: Timestamp
        description: Unix epoch seconds when event occurred
      - name: payload
        type: Bytes
        optional: true
        description: Optional binary payload
```

---

## 5. Implementation Plan

### Phase 1: New Primitive Types
1. Add new variants to `SimpleType` enum
2. Update `RustOptions` type mappings
3. Add tests for each new type
4. Document required crate dependencies

### Phase 2: Portable Serde Features
1. Extend `FieldConfig` with `default`, `alias`
2. Extend `TypeConfig` with `rename_all`
3. Update Rust codegen to emit attributes
4. Add tests

### Phase 3: Rust-Specific Features
1. Add `RustFieldConfig` for `skip_if_none`, `skip_if_default`, `flatten`
2. Add `RustTypeConfig` for `deny_unknown_fields`
3. Update codegen to handle nested config
4. Add tests

### Phase 4: Documentation Support
1. Add `description` field to types and fields
2. Add `deprecated` field
3. Generate doc comments in Rust
4. Add tests

### Phase 5: TypeScript Support (Future)
1. Implement TypeScript codegen
2. Map portable features
3. Add TS-specific config namespace if needed

---

## 6. Open Questions

1. **Duration format** - Serialize as seconds (number) or ISO 8601 duration string (`PT1H30M`)?
2. **Bytes encoding** - Always base64, or allow hex option?
3. **Decimal precision** - Allow configuring precision/scale in schema?
4. **Generated helper functions** - Generate `is_default()` helpers for `skip_if_default`?

---

## 7. Out of Scope

- Validation constraints (min, max, pattern) - Consider for future
- Code generation for other languages beyond Rust/TypeScript - Future phases
- JSON Schema export - Future consideration
