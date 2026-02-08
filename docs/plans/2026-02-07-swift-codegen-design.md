# Swift Code Generation Design

## Overview

Add Swift code generation support to Fluorite, enabling generation of Swift Codable types from `.fl` schema definitions for iOS/macOS client applications.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Target platform | iOS/macOS client apps | Use Foundation types (Date, URL, UUID, Data) |
| Optionals | Swift Optional (`T?`) | Idiomatic Swift |
| Mutability | Immutable (`let`) | API response models shouldn't need mutation |
| Protocols | `Codable`, `Equatable`, `Sendable` | Serialization, equality, concurrency safety |
| Visibility | `public` (configurable) | Shared models across modules |
| Any type | `AnyCodable` from runtime package | Consistent with Rust runtime pattern |
| File organization | Both modes via `--single-file` flag | Matches Rust/TypeScript behavior |
| Union encoding | Inline `init(from:)`/`encode(to:)` | Self-contained, no extra abstractions |
| Package.swift generation | No | Users manage their own project structure |
| Runtime location | `swift-runtime/` in main repo | Consistent with `runtime/` for Rust |

## Type Mapping

### Primitives

| IR Primitive | Swift Type | Notes |
|--------------|------------|-------|
| String | `String` | |
| Bool | `Bool` | |
| Int32 | `Int32` | |
| Int64 | `Int64` | |
| UInt32 | `UInt32` | |
| UInt64 | `UInt64` | |
| Float32 | `Float` | |
| Float64 | `Double` | |
| UUID | `UUID` | Foundation |
| Decimal | `Decimal` | Foundation |
| DateTime | `Date` | Foundation |
| DateTimeUtc | `Date` | Foundation |
| DateTimeTz | `Date` | Foundation |
| Date | `String` | ISO8601 date string |
| Time | `String` | ISO8601 time string |
| Duration | `TimeInterval` | Foundation (Double alias) |
| Timestamp | `Date` | Unix seconds, custom decoder |
| TimestampMillis | `Date` | Unix millis, custom decoder |
| Bytes | `Data` | Foundation |
| Url | `URL` | Foundation |
| Any | `AnyCodable` | From FluoriteRuntime |

### Collections

| IR Type | Swift Type |
|---------|------------|
| `List<T>` | `[T]` |
| `Map<K, V>` | `[K: V]` |
| `Optional<T>` | `T?` |

## Generated Code Examples

### Struct (from IRStruct)

```swift
import Foundation

/// Represents a user in the system
public struct User: Codable, Equatable, Sendable {
    public let id: UUID
    public let name: String
    public let email: String?

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case email = "email_address"  // when rename is specified
    }
}
```

### Enum (from IREnum)

```swift
public enum UserStatus: String, Codable, Equatable, Sendable {
    case active = "Active"
    case inactive = "Inactive"
}
```

### Union (from IRUnion) - Adjacently Tagged

```swift
/// User lifecycle events
public enum UserEvent: Codable, Equatable, Sendable {
    case created(User)
    case updated(User)
    case deleted

    enum CodingKeys: String, CodingKey {
        case type
        case value
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "Created":
            self = .created(try container.decode(User.self, forKey: .value))
        case "Updated":
            self = .updated(try container.decode(User.self, forKey: .value))
        case "Deleted":
            self = .deleted
        default:
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: decoder.codingPath,
                    debugDescription: "Unknown type: \(type)"
                )
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .created(let value):
            try container.encode("Created", forKey: .type)
            try container.encode(value, forKey: .value)
        case .updated(let value):
            try container.encode("Updated", forKey: .type)
            try container.encode(value, forKey: .value)
        case .deleted:
            try container.encode("Deleted", forKey: .type)
        }
    }
}
```

### Type Alias (from IRTypeAlias)

```swift
public typealias UserList = [User]
public typealias UserMap = [String: User]
```

## CLI Interface

### Command

```bash
fluorite swift \
  --inputs examples/users.fl examples/orders.fl \
  --output ./Sources/Generated \
  --single-file false \
  --any-type AnyCodable \
  --visibility public
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--inputs` | required | Input .fl or .yaml files |
| `--output` | required | Output directory |
| `--single-file` | `false` | All types in one file vs separate files |
| `--any-type` | `AnyCodable` | Type to use for Any (allows override) |
| `--visibility` | `public` | Access level: public, internal, package |

## File Structure

### Multi-file Output (default)

```
Sources/Generated/
├── users/
│   ├── User.swift
│   ├── UserStatus.swift
│   ├── UserEvent.swift
│   └── Users.swift        # barrel file with imports
└── orders/
    ├── Order.swift
    └── Orders.swift
```

### Single-file Output

```
Sources/Generated/
├── users/
│   └── Users.swift        # all types in one file
└── orders/
    └── Orders.swift
```

## Runtime Package

### Location

```
fluorite/
├── runtime/              # Existing Rust runtime
├── swift-runtime/        # New Swift runtime
│   ├── Package.swift
│   ├── Sources/
│   │   └── FluoriteRuntime/
│   │       └── AnyCodable.swift
│   └── Tests/
│       └── FluoriteRuntimeTests/
│           └── AnyCodableTests.swift
└── codegen/
```

### User Integration

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/blossomstack/fluorite", from: "0.5.0"),
],
targets: [
    .target(name: "MyApp", dependencies: [
        .product(name: "FluoriteRuntime", package: "fluorite")
    ])
]
```

### AnyCodable Implementation

```swift
public enum AnyCodable: Codable, Equatable, Sendable {
    case null
    case bool(Bool)
    case int(Int64)
    case double(Double)
    case string(String)
    case array([AnyCodable])
    case object([String: AnyCodable])

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self = .null
        } else if let bool = try? container.decode(Bool.self) {
            self = .bool(bool)
        } else if let int = try? container.decode(Int64.self) {
            self = .int(int)
        } else if let double = try? container.decode(Double.self) {
            self = .double(double)
        } else if let string = try? container.decode(String.self) {
            self = .string(string)
        } else if let array = try? container.decode([AnyCodable].self) {
            self = .array(array)
        } else if let object = try? container.decode([String: AnyCodable].self) {
            self = .object(object)
        } else {
            throw DecodingError.dataCorrupted(
                DecodingError.Context(
                    codingPath: decoder.codingPath,
                    debugDescription: "Unable to decode AnyCodable"
                )
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .null:
            try container.encodeNil()
        case .bool(let value):
            try container.encode(value)
        case .int(let value):
            try container.encode(value)
        case .double(let value):
            try container.encode(value)
        case .string(let value):
            try container.encode(value)
        case .array(let value):
            try container.encode(value)
        case .object(let value):
            try container.encode(value)
        }
    }
}
```

## Implementation Structure

### New Files

```
codegen/
├── src/code_gen/
│   ├── swift/
│   │   ├── mod.rs              # Module exports
│   │   ├── options.rs          # SwiftOptions struct
│   │   ├── template_generator.rs  # Main generator
│   │   └── templates.rs        # Template data structs
│   └── mod.rs                  # Add: pub mod swift;
├── templates/
│   └── swift/
│       ├── struct.swift.j2     # Struct template
│       ├── enum.swift.j2       # Enum template
│       ├── union.swift.j2      # Union with Codable impl
│       ├── type_alias.swift.j2 # typealias template
│       └── barrel.swift.j2     # Re-export file
└── src/main.rs                 # Add Swift subcommand

swift-runtime/
├── Package.swift
├── Sources/
│   └── FluoriteRuntime/
│       └── AnyCodable.swift
└── Tests/
    └── FluoriteRuntimeTests/
        └── AnyCodableTests.swift
```

### Changes to Existing Files

- `codegen/src/code_gen/mod.rs` - add `pub mod swift;`
- `codegen/src/main.rs` - add `Swift` CLI subcommand

## Testing Strategy

1. Unit tests using `MemoryFileSystem` to verify generated output
2. Integration tests with example `.fl` files
3. Swift compilation tests to ensure generated code compiles
4. Serialization round-trip tests in Swift runtime

## Implementation Order

1. Create `swift-runtime/` package with `AnyCodable`
2. Create `codegen/src/code_gen/swift/options.rs`
3. Create `codegen/templates/swift/*.j2` templates
4. Create `codegen/src/code_gen/swift/templates.rs`
5. Create `codegen/src/code_gen/swift/template_generator.rs`
6. Add CLI subcommand in `main.rs`
7. Add tests
