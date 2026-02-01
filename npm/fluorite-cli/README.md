# @zhxiaogg/fluorite-cli

Code generator from YAML schema definitions - generates Rust and TypeScript code.

## Installation

```bash
npm install -D @zhxiaogg/fluorite-cli
```

## Usage

### Generate TypeScript

```bash
npx fluorite ts --inputs ./schemas/*.yaml --output ./src/generated
```

### Generate Rust

```bash
npx fluorite rust --inputs ./schemas/*.yaml --output ./src/generated
```

## Options

### TypeScript (`ts`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input YAML files |
| `--output` | required | Output directory |
| `--single-file` | false | Generate all types in a single file |
| `--any-type` | unknown | Type to use for Any fields |
| `--readonly` | false | Generate readonly properties |

### Rust (`rust`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input YAML files |
| `--output` | required | Output directory |
| `--single-file` | true | Generate all types in a single file |
| `--any-type` | fluorite::Any | Type to use for Any fields |
| `--derives` | | Custom derives (comma-separated) |
| `--extra-derives` | | Additional derives |
| `--generate-new` | true | Generate derive_new |
| `--visibility` | public | Type visibility |

## Example package.json

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

## Building from Source

If pre-built binaries are not available for your platform:

```bash
git clone https://github.com/blossomstack/fluorite.git
cd fluorite
cargo build --release --package fluorite_codegen
```

The binary will be at `target/release/fluorite`.
