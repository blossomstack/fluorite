# Go Code Generation Design

## Overview

Add Go code generation to Fluorite, so `.fl` schemas produce Go structs that serialize
to the same JSON as the Rust, TypeScript and Swift backends already do.

The motivating consumer is a Terraform provider. Terraform plugins are Go gRPC servers,
so a Go client for a Fluorite-defined API cannot reuse any of the existing three
backends. That consumer also sets two priorities the other backends do not have:
the generated code must depend on nothing outside the Go standard library, and it must
pass `gofmt` and `go vet` unmodified.

Scope is types only. Fluorite generates no HTTP client, no route definitions and no
`go.mod` — the consuming module owns those, exactly as `clients/web` hand-writes its
fetch calls over the generated `clients/ts` types.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Package layout | One flat Go package for all `.fl` packages | Go forbids import cycles; Fluorite resolves imported types by bare name globally, so a per-package layout can emit an unbuildable tree from a schema Rust and TS accept |
| Name collisions | New validation error naming the type and both packages | A flat package makes cross-package duplicates fatal; fail at codegen, not at `go build` |
| Runtime module | None | `any` and `encoding/json` cover what `fluorite::Any` and `AnyCodable` exist for |
| External deps | None, stdlib only | A published Terraform provider is judged on its dependency tree |
| Type names | Passed through unchanged | Already PascalCase and author-chosen |
| Field names | snake_case → PascalCase with Go initialisms | `revive`/`staticcheck` expect `ID`, `APIKey`, `BaseURL`; changing this later is a breaking API change |
| Wire names | Always the exact camelCase name in the JSON tag | The derived Go name is never load-bearing |
| Optionals | `*T` with `,omitempty`, including `*[]T` and `*map[K]V` | Omission is semantically load-bearing in Horsie's `*Input` types: absent means "leave unchanged", which `[]T` cannot express. The Rust backend emits `skip_serializing_if = "Option::is_none"` on every optional field, so `,omitempty` on every optional is what keeps the two wire-compatible |
| Extended primitives | `string` / `int64`, matching the TypeScript backend | Keeps the stdlib-only rule; `Decimal` and `Duration` have no clean stdlib type, and `Timestamp` would need a custom codec |
| Unions | Wrapper struct around a sealed interface | The wrapper is a concrete type, so it marshals correctly as a field, slice element or map value with no cooperation from the parent |
| `go.mod` generation | No | Users manage their own module |
| Unsupported IR features | Fail codegen with a clear message | `flatten`, `default`, `alias` and `deny_unknown_fields` have no clean struct-tag equivalent; silently dropping them produces wrong wire behaviour |

## Type Mapping

### Primitives

| IR Primitive | Go Type | Notes |
|--------------|---------|-------|
| String | `string` | |
| Bool | `bool` | |
| Int32 / Int64 | `int32` / `int64` | |
| UInt32 / UInt64 | `uint32` / `uint64` | |
| Float32 / Float64 | `float32` / `float64` | |
| UUID | `string` | |
| Decimal | `string` | |
| Bytes | `string` | base64, as on the wire |
| Url | `string` | |
| DateTime | `string` | ISO 8601 |
| DateTimeUtc | `string` | ISO 8601 |
| DateTimeTz | `string` | ISO 8601 with offset |
| Date | `string` | ISO 8601 date |
| Time | `string` | ISO 8601 time |
| Duration | `string` | ISO 8601 duration |
| Timestamp | `int64` | Unix epoch seconds |
| TimestampMillis | `int64` | Unix epoch milliseconds |
| Any | `any` | Overridable via `--any-type` |

This deliberately diverges from the Swift backend, which maps `DateTime` to Foundation's
`Date` and `Url` to `URL`. Go's `time.Time` would cover `DateTime`, but `Decimal`,
`Duration` and the two `Timestamp` variants would each need a custom codec or a
third-party type, and the wire format is a string or a number either way. Uniform
`string`/`int64` keeps the package dependency-free and the mapping obvious.

### Collections

| IR Type | Go Type |
|---------|---------|
| `List<T>` | `[]T` |
| `Map<K, V>` | `map[K]V` |
| `Option<T>` | `*T` |

`Option<List<T>>` becomes `*[]T` rather than `[]T` with `omitempty`. It reads as
un-idiomatic Go, and it is the only representation that distinguishes an absent list
from an empty one.

### Naming

Field names convert from snake_case to PascalCase, uppercasing any segment in the
initialism list:

```
ID  URL  URI  API  HTTP  HTTPS  JSON  XML  HTML  SQL  SSE  TLS  TTL  UUID
CPU  RAM  OS   IP   MCP   LLM    CLI   SDK  RPC   ACL  DB   EOF  UID  GID
```

So `id → ID`, `api_key → APIKey`, `base_url → BaseURL`, `model_id → ModelID`,
`mcp_servers → MCPServers`.

Go keywords are all lowercase and every generated identifier is exported, so a field or
type can never collide with a keyword. No escaping logic is needed.

## Generated Code Examples

### Struct

```go
// An agent preset as shown to clients.
type AgentView struct {
	// Slug; the id of record, used in API paths and CLI invocations.
	Name string `json:"name"`
	// Configured model alias.
	Model string `json:"model"`
	// Repositories cloned into the session workspace at provision time.
	Repos []RepoConfig `json:"repos"`
	// Canonical thinking effort; absent → the model's configured default.
	ThinkingEffort *string `json:"thinkingEffort,omitempty"`
}
```

Every optional field gets `,omitempty`, so a nil pointer is omitted rather than
emitted as `null`. This matches the Rust backend, whose struct template applies
`#[serde(skip_serializing_if = "Option::is_none")]` to every optional field regardless
of whether the schema carries `#[skip_if_none]`.

### Enum

```go
type UserStatus string

const (
	UserStatusActive   UserStatus = "Active"
	UserStatusInactive UserStatus = "Inactive"
)
```

Constants are type-prefixed because Go has no per-type namespacing, and the block is
column-aligned in gofmt's alignment runs — a variant carrying a doc comment starts a new
run, because a comment line flushes gofmt's aligner.

### Union

For an adjacently tagged union with `tag_field: "type"` and `content_field: "value"`:

```go
// UserEventVariant is implemented by every variant of UserEvent.
type UserEventVariant interface{ isUserEventVariant() }

type UserEventCreated struct{ Value User }

func (UserEventCreated) isUserEventVariant() {}

type UserEventDeleted struct{}

func (UserEventDeleted) isUserEventVariant() {}

type UserEvent struct{ Variant UserEventVariant }

func (u UserEvent) MarshalJSON() ([]byte, error)
func (u *UserEvent) UnmarshalJSON(data []byte) error
```

The unexported marker method means only generated variants satisfy the interface.
Consumers type-switch on `.Variant`.

The union's own doc comment sits on the `{{ name }}` wrapper rather than at the top of
the file, so godoc attributes it to the type callers actually use instead of to the
sealed interface.

`MarshalJSON` emits the union's own tag and content field names, so
`#[type_tag = "kind"]` is honoured. A nil `Variant` returns an error rather than
emitting a half-formed object. `UnmarshalJSON` reads the tag, then decodes the content
into the matching variant; an unrecognised tag returns an error naming both the tag and
the union.

### Type Alias

```go
type SessionList []SessionSummary
type Headers map[string]string
```

## File Organization

One output directory, one Go package. Every `.fl` package's types land side by side.

- Package name comes from `--package-name`, defaulting to the output directory's basename.
- One file per type by default; `--single-file` collapses everything into `types.go`.
- Every file opens with `// Code generated by fluorite. DO NOT EDIT.`, which Go tooling
  and diff viewers recognise.

## CLI

```
fluorite go --inputs <paths...> --output <dir>
            [--package-name <name>]
            [--any-type any]
            [--single-file]
```

Mirrors `fluorite ts`. `--inputs` accepts files or directories, as all backends now do.

## Implementation Layout

A new backend beside the existing three, touching nothing else — no changes to the
lexer, parser, AST→IR lowering, or the Rust/TS/Swift backends.

```
codegen/src/code_gen/go/mod.rs
codegen/src/code_gen/go/naming.rs        # initialisms + gofmt column alignment
codegen/src/code_gen/go/options.rs
codegen/src/code_gen/go/template_generator.rs
codegen/src/code_gen/go/templates.rs
codegen/templates/go/file_header.go.j2
codegen/templates/go/struct.go.j2
codegen/templates/go/enum.go.j2
codegen/templates/go/union.go.j2
codegen/templates/go/type_alias.go.j2
codegen/src/main.rs                      # + a `Go` subcommand
```

The cross-package duplicate-name check lives in the Go generator rather than the
shared `validation` module. Adding a variant to `ValidationError` would force
`format_validation_errors` changes in all three existing backends — their matches are
exhaustive under `deny(clippy::wildcard_enum_match_arm)` — for a rule only Go has.

## Unsupported IR Features

Some IR attributes have no clean Go struct-tag equivalent — three on fields (`flatten`,
`default`, `alias`) and one on structs (`deny_unknown_fields`). Rather than emit code
that silently disagrees with the Rust side, the Go backend fails generation with a
message naming the type, the field where applicable, and the attribute:

| Attribute | Disposition |
|-----------|-------------|
| `flatten` | Error |
| `default` | Error |
| `alias` | Error |
| `deny_unknown_fields` | Error |
| `rename` | Supported — it is just the JSON tag |
| `skip_if_default` | Supported — `,omitempty` is exactly "skip if zero value" |
| `skip_if_none` | Supported — every optional gets `,omitempty` regardless |
| `is_boxed` | Ignored — Rust-only |
| `deprecated` | Emits a `// Deprecated:` comment, which Go tooling reads |
| doc comments | Emitted verbatim as `//` lines via the shared `doc::doc_lines`, so a multi-line doc survives |

None of the four erroring attributes is used by the motivating consumer's schemas.
Supporting them later means generating a custom `UnmarshalJSON` per affected struct;
erroring now keeps that decision open without shipping wrong behaviour.

## Testing

Three layers, the second being the one that proves wire correctness.

**Backend unit tests** in `codegen/src/code_gen/go/template_generator.rs`, matching how
the Rust, TypeScript and Swift backends test: each IR shape in, expected Go out.

**Interop tests**, extending the existing harness. Add `examples/demo-go` mirroring
`examples/demo-ts`, and `rust_to_go` / `go_to_rust` fixture directories under
`tests/interop/fixtures/`, wired into `run-interop-test.sh` and `make interop-test`.
The same schemas and the same JSON cross the boundary both ways. This is what catches a
wrong tag, a mishandled optional, or a union that does not round-trip.

**Generated-output lint in CI.** Run `gofmt -l` and `go vet` over the generated package
and fail the build on any output. A bad initialism, an unformatted template or a
malformed tag fails here rather than shipping. `ci.yml` gains a Go toolchain step.

## Out of Scope

- HTTP client, route or RPC generation.
- `go.mod` emission.
- A Go runtime module — there is nothing for it to hold.
- Support for `flatten`, `default`, `alias` and `deny_unknown_fields`.
