# Dotted Package Syntax Design

## Overview

This document describes the design for adding dotted package names and import paths to the Fluorite IDL parser. This enables hierarchical package organization using dot-separated paths (e.g., `package com.example.users;`).

## Motivation

- **Hierarchical organization**: Allow packages to be organized in a namespace hierarchy
- **Consistency**: Use familiar dot notation common in Java, Python, and other languages
- **Cross-package imports**: Enable clear import paths like `use com.example.users.User;`

## Syntax Changes

### Package Declaration

Before (current):
```rust
package users;
```

After (new syntax):
```rust
package com.example.users;
```

Both simple names and dotted paths are supported. A simple name is just a single-segment path.

### Import Statements

Before (current - using `::`):
```rust
use users::User;
```

After (new syntax - using `.`):
```rust
use com.example.users.User;
```

## Implementation Plan

### 1. Lexer Changes (`codegen/src/idl/lexer.rs`)

Add `Dot` token and remove `DoubleColon`:

```rust
// Add new token
#[token(".")]
Dot,

// Remove DoubleColon token (no longer needed)
// #[token("::")]
// DoubleColon,  // REMOVED
```

Update the `Display` impl and tests accordingly.

### 2. Parser Changes (`codegen/src/idl/parser.rs`)

Create shared `dotted_path()` parser for both package and use statements:

```rust
/// Parser for dotted path: `foo.bar.baz`
fn dotted_path() -> impl Parser<Token, Vec<Spanned<String>>, Error = ParseError> {
    ident()
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect()
}

/// Parser for package statement: `package com.example.users;`
fn package_stmt() -> impl Parser<Token, Vec<Spanned<String>>, Error = ParseError> {
    just(Token::Package)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
}

/// Parser for use statement: `use com.example.users.User;`
fn use_stmt() -> impl Parser<Token, AstUse, Error = ParseError> {
    just(Token::Use)
        .ignore_then(dotted_path())
        .then_ignore(just(Token::Semi))
        .map_with_span(|path, span| AstUse { path, span })
}
```

### 3. AST Changes (`codegen/src/idl/ast.rs`)

Change package from single string to path segments:

```rust
/// A complete .fl file
#[derive(Debug, Clone)]
pub struct AstFile {
    pub package: Vec<Spanned<String>>,  // Changed from Spanned<String>
    pub uses: Vec<AstUse>,
    pub items: Vec<AstItem>,
}
```

`AstUse` already uses `Vec<Spanned<String>>` for its path, so no change needed.

### 4. AST-to-IR Conversion (`codegen/src/idl/ast_to_ir.rs`)

Join path segments with `.` for the IR package name:

```rust
// In convert_files()
let package_name = file.package
    .iter()
    .map(|s| s.value.as_str())
    .collect::<Vec<_>>()
    .join(".");
```

### 5. Updated Demo Files

**examples/users.fl:**
```rust
/// User management types
package com.example.users;

/// Represents a user in the system
struct User {
    /// Unique identifier for the user
    id: Uuid,
    /// User's full name
    name: String,
    /// User's email address
    email: String,
    /// Optional age of the user
    age: Option<u32>,
    /// User's account status
    status: UserStatus,
    /// When the user was created
    created_at: DateTime,
}

/// Possible statuses for a user account
enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

/// List of users
type UserList = Vec<User>;
```

**examples/orders.fl:**
```rust
/// Order management types
package com.example.orders;

use com.example.users.User;
use com.example.users.UserStatus;

/// Represents a customer order
struct Order {
    /// Unique order identifier
    id: Uuid,
    /// Reference to the user who placed the order
    user_id: Uuid,
    /// Items in this order
    items: Vec<OrderItem>,
    /// Total order amount
    total: Decimal,
    /// Order status
    status: OrderStatus,
    /// Shipping address
    shipping_address: Address,
    /// When the order was placed
    created_at: DateTime,
    /// Optional tracking number
    tracking_number: Option<String>,
}

/// An item within an order
struct OrderItem {
    /// Product identifier
    product_id: Uuid,
    /// Product name
    name: String,
    /// Quantity ordered
    quantity: u32,
    /// Price per unit
    unit_price: Decimal,
}

/// Shipping address
struct Address {
    /// Street address line 1
    street1: String,
    /// Street address line 2 (optional)
    street2: Option<String>,
    /// City
    city: String,
    /// State or province
    state: String,
    /// Postal code
    postal_code: String,
    /// Country code
    country: String,
}

/// Possible order statuses
enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

/// Event types for order lifecycle
union OrderEvent {
    Created(Order),
    Updated(Order),
    Cancelled(Order),
}

/// Map of order IDs to orders
type OrderMap = Map<String, Order>;
```

## Test Plan

### Unit Tests

**Lexer tests (`lexer.rs`):**
- `test_dot_token` - Verify `.` tokenizes as `Token::Dot`
- `test_dotted_identifier` - Verify `foo.bar` produces `[Ident, Dot, Ident]`

**Parser tests (`parser.rs`):**
- `test_parse_dotted_package` - Parse `package com.example.users;`
- `test_parse_simple_package` - Parse `package users;` (backwards compatible)
- `test_parse_dotted_use` - Parse `use com.example.users.User;`
- `test_parse_deep_dotted_path` - Parse `a.b.c.d.e.f`

**AST-to-IR tests (`ast_to_ir.rs`):**
- `test_convert_dotted_package` - Verify `["com", "example", "users"]` becomes `"com.example.users"`
- `test_convert_simple_package` - Verify `["users"]` becomes `"users"`

### Integration Tests

**New file: `codegen/tests/idl_dotted_paths.rs`**

```rust
#[test]
fn test_parse_demo_users_fl() {
    // Parse examples/users.fl with dotted package
    // Verify: package = "com.example.users"
    // Verify: 3 types (User struct, UserStatus enum, UserList alias)
}

#[test]
fn test_parse_demo_orders_fl() {
    // Parse examples/orders.fl with dotted imports
    // Verify: package = "com.example.orders"
    // Verify: imports from com.example.users
}

#[test]
fn test_multi_file_cross_package_imports() {
    // Parse both users.fl and orders.fl together
    // Verify: orders can reference User type from users package
    // Verify: IR schema has both packages
}

#[test]
fn test_rust_codegen_dotted_packages() {
    // Generate Rust code from demo .fl files
    // Verify: output directory structure matches package path
    // Verify: use statements in generated code are correct
}

#[test]
fn test_ts_codegen_dotted_packages() {
    // Generate TypeScript code from demo .fl files
    // Verify: output matches package structure
    // Verify: import statements are correct
}

#[test]
fn test_deeply_nested_package() {
    // Test edge case: very deep nesting
    // package a.b.c.d.e.f.types;
    // Verify: all segments preserved
}

#[test]
fn test_single_segment_package_still_works() {
    // Backwards compatibility: simple package names
    // package users;
    // Verify: works as before (single-element path)
}
```

### E2E Tests

**New file: `codegen/tests/e2e_dotted_packages.rs`**

```rust
#[test]
fn test_e2e_rust_serialization_roundtrip() {
    // 1. Parse demo .fl files with dotted packages
    // 2. Generate Rust code
    // 3. Create test instances
    // 4. Serialize to JSON
    // 5. Deserialize back
    // 6. Verify equality
}

#[test]
fn test_e2e_typescript_type_compatibility() {
    // 1. Parse demo .fl files
    // 2. Generate TypeScript
    // 3. Verify TypeScript compiles (tsc --noEmit)
    // 4. Verify types match expected structure
}

#[test]
fn test_e2e_cross_language_json_compatibility() {
    // 1. Generate both Rust and TypeScript from same .fl
    // 2. Create JSON from Rust side
    // 3. Verify TypeScript can parse it
}
```

## Documentation Updates

1. **docs/plans/2026-02-01-fluorite-idl-design.md** - Change all `::` to `.` in examples
2. **CLAUDE.md (root)** - Update IDL syntax examples
3. **CLAUDE.md (worktree)** - Update IDL syntax examples

## Implementation Order

1. Add `Dot` token to lexer, remove `DoubleColon`
2. Update parser to use dotted paths
3. Update AST `package` field type
4. Update AST-to-IR converter
5. Add unit tests for lexer/parser changes
6. Update demo .fl files
7. Add integration tests
8. Add E2E tests
9. Update documentation
10. Run `make all` to verify

## E2E Acceptance Criteria

1. `examples/users.fl` parses with package `com.example.users`
2. `examples/orders.fl` parses with package `com.example.orders` and imports from `com.example.users`
3. Rust code generation from `.fl` produces correct module structure
4. TypeScript code generation from `.fl` produces correct imports
5. All existing tests continue to pass
6. `make all` passes (format, lint, test)
