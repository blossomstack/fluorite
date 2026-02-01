# Fluorite

[![Crates.io](https://img.shields.io/crates/v/fluorite)](https://crates.io/crates/fluorite)
[![docs.rs](https://img.shields.io/docsrs/fluorite)](https://docs.rs/fluorite/latest)
[![CI](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml/badge.svg)](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml)

Generate rust/typescript codes from schemas specified by Yaml/JSON.

## Using `fluorite` in a Cargo Project
> Please check the [demo project](./examples/demo) for details.
Add following dependencies first:
```toml
[dependencies]
serde = "<serde-version>"
fluorite = "0.1"
derive-new = "0.6"

[build-dependencies]
fluorite_codegen = "0.1"
```
Using `fluorite` in the `build.rs` to generate codes during the Cargo build process:
```rust
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fluorite::compile(&["fluorite/demo.yaml"], out_dir.as_str()).unwrap();
}
```
Instruct your project to include the generated codes, e.g. in your lib or main file:
```rust
mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}
```
## Using `fluorite` as a cli
```shell
$ fluorite --help
Generate rust/typescript codes from schemas specified by Yaml/JSON.

Usage: fluorite [OPTIONS] --output <OUTPUT>

Options:
  -i, --inputs <INPUTS>  Input definition files
  -o, --output <OUTPUT>  Output directory
  -t, --target <TARGET>  Target language [default: rust] [possible values: rust]
  -h, --help             Print help
  -V, --version          Print version
```
## Features
- [x] Supports Yaml schema definition, see [example here](examples/orders.yml)
- [x] A codegen binary program
- [ ] Language Support
  - [x] Rust codegen used in Cargo `build.rs` script
  - [ ] Typescript codegen
  - [x] [CodeGen API](./codegen/src/code_gen/abi.rs) to add more language supports to `fluorite`
- [ ] Support for JSON schema definition (no plan)

## Schema Definition Features
More details can be found in [definitions.rs](codegen/src/definitions/mod.rs).
- User defined types:
  - Object
  - Enum
  - Union: to support polymorphic types during serialization/deserialization
- Collection types:
  - List
  - Map
- Primitive types
  - String
  - Bool
  - (WIP) DateTime
  - UIntX
  - IntX
  - Float
- Optional fields support
- Any type fields support

## Development

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

