# TypeScript Code Generation Design

**Date:** 2026-02-01
**Status:** Approved

## Overview

Add TypeScript code generation to Fluorite, enabling YAML schema definitions to generate TypeScript interfaces, enums, and discriminated unions. Includes an npm package (`@zhxiaogg/fluorite-cli`) for easy integration into npm/Bun-managed TypeScript projects.

## Goals

1. Generate idiomatic TypeScript code from YAML schemas
2. Seamless build-time integration with npm/Bun/pnpm/yarn projects
3. Reuse existing IR, validation, and filesystem abstractions
4. Type-safe discriminated unions with TypeScript type narrowing

## Non-Goals

- Runtime validation (Zod support deferred to future)
- CommonJS module output (ESM only)
- Browser-specific bundling

## Architecture

```
YAML Schemas → Definitions → IRBuilder → IRSchema
                                           ↓
                                      Validator
                                           ↓
                              TsTemplateGenerator → .ts files
```

The TypeScript generator reuses:
- `IRBuilder` - converts YAML definitions to language-agnostic IR
- `Validator` - validates schema before generation
- `FileSystem` trait - enables testing with `MemoryFileSystem`

### File Structure

```
codegen/
├── src/code_gen/ts/
│   ├── mod.rs               # Module exports
│   ├── options.rs           # TypeScriptOptions configuration
│   ├── template_generator.rs # TsTemplateGenerator (main logic)
│   └── templates.rs         # Askama template wrapper structs
└── templates/ts/
    ├── interface.ts.j2      # Object → interface
    ├── enum.ts.j2           # Enum types
    ├── union.ts.j2          # Discriminated unions
    ├── type_alias.ts.j2     # List/Map type aliases
    └── index.ts.j2          # Barrel exports

npm/fluorite-cli/            # npm package
├── package.json
├── install.js               # Downloads platform-specific binary
├── bin/
│   └── fluorite.js          # Node.js wrapper
└── README.md
```

## Type Mapping

### Primitives

| YAML Type | TypeScript |
|-----------|-----------|
| String | `string` |
| Bool | `boolean` |
| Int32, Int64, UInt32, UInt64 | `number` |
| Float32, Float64 | `number` |
| DateTime | `string` |
| Any | `unknown` |

### Collections & Modifiers

| YAML Construct | TypeScript |
|----------------|-----------|
| `List<T>` | `T[]` |
| `Map<K, V>` | `Record<K, V>` |
| Optional field | `fieldName?: Type` |
| Box wrapper | (no effect in TS) |

## Generated Code Examples

### Object → Interface

```yaml
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
      - name: type
        type: String
        configs:
          rename: order_type
```

```typescript
export interface Order {
  id: number;
  item: string;
  shipping?: Shipping;
  orderType: string;
}
```

### Enum

```yaml
types:
  - name: Gender
    type: Enum
    values: [Male, Female]
```

```typescript
export enum Gender {
  Male = "Male",
  Female = "Female",
}
```

### Union → Discriminated Union

```yaml
types:
  - name: PaymentMethod
    type: Union
    type_tag: type
    values: [CreditCard, BankTransfer, Cash]
    configs:
      union_style: Inline
```

```typescript
export type PaymentMethod =
  | { type: "CreditCard"; cardNumber: string; expiry: string }
  | { type: "BankTransfer"; accountNumber: string }
  | { type: "Cash" };
```

### List/Map Type Aliases

```typescript
export type OrderList = Order[];
export type UserMap = Record<string, User>;
```

## Configuration

### TypeScriptOptions

```rust
pub struct TypeScriptOptions {
    pub output_dir: String,
    pub single_file: bool,           // All types in one file vs separate
    pub package_name: Option<String>, // Override package directory name
    pub use_readonly: bool,          // readonly properties
    pub any_type: String,            // "unknown" (default) or "any"
}
```

### CLI Usage

```bash
fluorite ts \
  --inputs ./schemas/orders.yaml ./schemas/users.yaml \
  --output ./src/generated \
  --single-file false \
  --readonly true \
  --any-type unknown
```

### Output Modes

**Multi-file mode** (default):
```
src/generated/
├── orders/
│   ├── order.ts
│   ├── shipping.ts
│   └── index.ts
├── users/
│   ├── user.ts
│   └── index.ts
└── index.ts
```

**Single-file mode**:
```
src/generated/
├── orders.ts
├── users.ts
└── index.ts
```

## npm Package Integration

### Package: @zhxiaogg/fluorite-cli

```json
{
  "name": "@zhxiaogg/fluorite-cli",
  "version": "0.1.0",
  "bin": {
    "fluorite": "./bin/fluorite.js"
  },
  "scripts": {
    "postinstall": "node install.js"
  },
  "os": ["darwin", "linux", "win32"],
  "cpu": ["x64", "arm64"]
}
```

### Usage in TypeScript Projects

```json
{
  "scripts": {
    "generate": "fluorite ts --inputs ./schemas/*.yaml --output ./src/generated",
    "build": "npm run generate && tsc"
  },
  "devDependencies": {
    "@zhxiaogg/fluorite-cli": "^0.1.0"
  }
}
```

### Binary Distribution

- Build Rust binaries for: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64
- Host binaries on GitHub Releases
- `install.js` downloads correct binary at npm install time

## Implementation Plan

### Phase 1: Core TypeScript Generator

1. Create `codegen/src/code_gen/ts/options.rs` with `TypeScriptOptions`
2. Create Askama templates in `codegen/templates/ts/`
3. Implement `TsTemplateGenerator` in `codegen/src/code_gen/ts/template_generator.rs`
4. Add `ts` subcommand to CLI in `codegen/src/main.rs`
5. Add unit tests with `MemoryFileSystem`

### Phase 2: Integration Tests

1. Create integration tests using `examples/orders.yml` and `examples/users.yml`
2. Verify generated TypeScript matches expected output
3. Test all type variations: Object, Enum, Union, List, Map

### Phase 3: E2E TypeScript Tests

1. Create `tests/ts_e2e/` directory with TypeScript project
2. Generate types from test schema
3. Compile with `tsc --strict` to verify correctness
4. Test type usage in actual TypeScript code

### Phase 4: npm Package

1. Create `npm/fluorite-cli/` package structure
2. Implement `install.js` for binary download
3. Create `bin/fluorite.js` wrapper
4. Set up GitHub Actions for cross-platform binary builds
5. Publish to npm

## Testing Strategy

### Unit Tests (Rust)

- Test `TsTemplateGenerator` with `MemoryFileSystem`
- Verify TypeScript syntax for each IR type
- Test edge cases: nested generics, optional fields, renames

### Integration Tests (Rust)

- Generate TypeScript from example YAML files
- Verify output structure and content
- Test CLI argument parsing

### E2E Tests (TypeScript)

```
tests/ts_e2e/
├── package.json
├── tsconfig.json
├── schemas/test.yaml
├── generated/           # Output from fluorite
└── src/test.ts          # Uses generated types
```

**Test Script:**
```bash
# Generate
cargo run -p fluorite_codegen --bin fluorite -- ts \
  --inputs tests/ts_e2e/schemas/test.yaml \
  --output tests/ts_e2e/generated

# Type-check
cd tests/ts_e2e && npx tsc --noEmit
```

## Acceptance Criteria

1. ✅ Generated TypeScript compiles with `tsc --strict`
2. ✅ Interfaces correctly represent Object types with all field modifiers
3. ✅ Enums have string values matching variant names
4. ✅ Discriminated unions work with TypeScript's type narrowing
5. ✅ List/Map type aliases resolve correctly
6. ✅ Cross-package type references work (imports between packages)
7. ✅ CLI works: `fluorite ts --inputs ... --output ...`
8. ✅ npm package can be installed and used: `npx @zhxiaogg/fluorite-cli ts ...`

## Future Enhancements

- **Zod schemas**: Optional `--with-zod` flag for runtime validation
- **JSDoc comments**: Generate documentation from YAML descriptions
- **Custom type mappings**: Configure DateTime to `Date` object, etc.
- **Watch mode**: Regenerate on schema file changes
