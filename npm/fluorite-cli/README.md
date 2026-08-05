# @blossomstack/fluorite-cli

Generate Rust and TypeScript code from Fluorite IDL (`.fl`) schema definitions.

## Installation

```bash
npm install -D @blossomstack/fluorite-cli
```

## Usage

### Generate TypeScript

```bash
npx fluorite ts --inputs ./schemas/*.fl --output ./src/generated
```

### Generate Rust

```bash
npx fluorite rust --inputs ./schemas/*.fl --output ./src/generated
```

## Options

### TypeScript (`ts`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input `.fl` files |
| `--output` | required | Output directory |
| `--single-file` | false | Generate all types in a single file |
| `--any-type` | unknown | Type to use for `Any` fields |
| `--readonly` | false | Generate readonly properties |
| `--package-name` | | Override output directory name |

### Rust (`rust`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input `.fl` files |
| `--output` | required | Output directory |
| `--single-file` | true | Generate all types in a single file |
| `--any-type` | fluorite::Any | Type to use for `Any` fields |
| `--derives` | | Custom derives (comma-separated, replaces defaults) |
| `--extra-derives` | | Additional derives (comma-separated) |
| `--generate-new` | true | Generate derive_new |
| `--visibility` | public | Type visibility |

## Example package.json

```json
{
  "scripts": {
    "generate": "fluorite ts --inputs ./schemas/*.fl --output ./src/generated",
    "build": "npm run generate && tsc"
  },
  "devDependencies": {
    "@blossomstack/fluorite-cli": "^0.2.0"
  }
}
```

## Building from Source

If pre-built binaries are not available for your platform:

```bash
git clone https://github.com/blossomstack/fluorite.git
cd fluorite
cargo build --release --package fluorite_codegen
```

The binary will be at `target/release/fluorite`.
