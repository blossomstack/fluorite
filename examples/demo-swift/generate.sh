#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Build the fluorite CLI if needed
echo "Building fluorite CLI..."
cargo build --package fluorite_codegen --release --manifest-path "$PROJECT_ROOT/Cargo.toml"

# Generate Swift code
echo "Generating Swift code..."
"$PROJECT_ROOT/target/release/fluorite" swift \
    --inputs \
        "$PROJECT_ROOT/examples/demo/fluorite/common.fl" \
        "$PROJECT_ROOT/examples/demo/fluorite/users.fl" \
        "$PROJECT_ROOT/examples/demo/fluorite/orders.fl" \
        "$PROJECT_ROOT/examples/demo/fluorite/notifications.fl" \
    --output "$SCRIPT_DIR/Sources/Generated" \
    --visibility public

echo "Swift code generated at Sources/Generated/"
