# Fluorite

[![Crates.io](https://img.shields.io/crates/v/fluorite)](https://crates.io/crates/fluorite)
[![docs.rs](https://img.shields.io/docsrs/fluorite)](https://docs.rs/fluorite/latest)
[![CI](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml/badge.svg)](https://github.com/blossomstack/fluorite/actions/workflows/ci.yml)

Generate Rust code from schemas specified by YAML.

## Using `fluorite` in a Cargo Project
> Please check the [demo project](./examples/demo) for details.
Add following dependencies first:
```toml
[dependencies]
serde = { version = "1.0", features = ["serde_derive"] }
fluorite = "0.1"
derive-new = "0.7"

[build-dependencies]
fluorite_codegen = "0.1"
```
Using `fluorite_codegen` in the `build.rs` to generate codes during the Cargo build process:
```rust
fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    fluorite_codegen::compile(&["fluorite/demo.yaml"], out_dir.as_str()).unwrap();
}
```
Instruct your project to include the generated codes, e.g. in your lib or main file:
```rust
mod demo {
    include!(concat!(env!("OUT_DIR"), "/demo/mod.rs"));
}
```
## Using `fluorite` as a CLI
```shell
$ fluorite --help
Generate Rust code from schemas specified by YAML.

Usage: fluorite <COMMAND>

Commands:
  rust  Generate Rust code
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

$ fluorite rust --help
Usage: fluorite rust [OPTIONS] --output <OUTPUT>

Options:
  -i, --inputs <INPUTS>  Input definition files
  -o, --output <OUTPUT>  Output directory
  -s, --single-file      Output codes to a single mod file for each package
  -h, --help             Print help
```
## Features
- [x] Supports Yaml schema definition, see [example here](examples/orders.yml)
- [x] A codegen binary program
- [ ] Language Support
  - [x] Rust codegen used in Cargo `build.rs` script
  - [ ] TypeScript codegen
  - [x] [CodeGen API](./codegen/src/code_gen/abi.rs) to add more language supports to `fluorite`

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
  - DateTime
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

