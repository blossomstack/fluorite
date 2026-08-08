#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "Fluorite E2E Interoperability Test"
echo "========================================="
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
RUST_TO_TS_DIR="$FIXTURES_DIR/rust_to_ts"
TS_TO_RUST_DIR="$FIXTURES_DIR/ts_to_rust"
RUST_TO_SWIFT_DIR="$FIXTURES_DIR/rust_to_swift"
SWIFT_TO_RUST_DIR="$FIXTURES_DIR/swift_to_rust"
RUST_TO_GO_DIR="$FIXTURES_DIR/rust_to_go"
GO_TO_RUST_DIR="$FIXTURES_DIR/go_to_rust"

# Create fixture directories
mkdir -p "$RUST_TO_TS_DIR"
mkdir -p "$TS_TO_RUST_DIR"
mkdir -p "$RUST_TO_SWIFT_DIR"
mkdir -p "$SWIFT_TO_RUST_DIR"
mkdir -p "$RUST_TO_GO_DIR"
mkdir -p "$GO_TO_RUST_DIR"

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

info() {
    echo -e "${YELLOW}→ INFO${NC}: $1"
}

# Step 1: Build Rust demo
echo "Step 1: Building Rust demo..."
cd "$SCRIPT_DIR/../../examples/demo"
if cargo build --quiet 2>/dev/null; then
    pass "Rust demo built successfully"
else
    fail "Rust demo build failed"
    exit 1
fi

# Step 2: Build Fluorite CLI for TypeScript generation
echo ""
echo "Step 2: Building Fluorite CLI..."
cd "$SCRIPT_DIR/../../"
if cargo build --bin fluorite --quiet 2>/dev/null; then
    pass "Fluorite CLI built successfully"
else
    fail "Fluorite CLI build failed"
    exit 1
fi

# Step 3: Generate TypeScript types
echo ""
echo "Step 3: Generating TypeScript types..."
cd "$SCRIPT_DIR/../../examples/demo-ts"
if "$SCRIPT_DIR/../../target/debug/fluorite" ts --inputs ../demo/fluorite/common.fl ../demo/fluorite/users.fl ../demo/fluorite/orders.fl ../demo/fluorite/notifications.fl --output ./generated 2>/dev/null; then
    pass "TypeScript types generated successfully"
else
    fail "TypeScript type generation failed"
    exit 1
fi

# Step 4: Install TypeScript dependencies and build
echo ""
echo "Step 4: Installing TypeScript dependencies..."
if [ ! -d "node_modules" ]; then
    # Use npm install with local package
    npm install --silent 2>/dev/null || true
fi

# Clean stale build artifacts
rm -rf dist/

# Build TypeScript - FAIL if compilation errors
info "Building TypeScript..."
if npx tsc; then
    pass "TypeScript built successfully"
else
    fail "TypeScript compilation failed"
    exit 1
fi

# Step 4.5: Build Swift demo (if Swift is available)
echo ""
echo "Step 4.5: Building Swift demo..."
if command -v swift &> /dev/null; then
    cd "$SCRIPT_DIR/../../examples/demo-swift"
    if swift build --quiet 2>/dev/null; then
        pass "Swift demo built successfully"
        SWIFT_AVAILABLE=true
    else
        fail "Swift demo build failed"
        SWIFT_AVAILABLE=false
    fi
else
    info "Swift not available, skipping Swift tests"
    SWIFT_AVAILABLE=false
fi

# Step 4.6: Build Go demo (if Go is available)
echo ""
echo "Step 4.6: Building Go demo..."
if command -v go &> /dev/null; then
    cd "$SCRIPT_DIR/../../examples/demo-go"
    FLUORITE="$SCRIPT_DIR/../../target/debug/fluorite"
    # Clean first: generation overwrites but never deletes, so a renamed type
    # would otherwise leave a stale file behind that still compiles.
    if "$FLUORITE" clean --output ./generated >/dev/null 2>&1 \
        && "$FLUORITE" go --inputs ../demo/fluorite --output ./generated --package-name generated >/dev/null 2>&1; then
        pass "Go types generated successfully"
    else
        fail "Go type generation failed"
        exit 1
    fi
    # Generated Go must already be gofmt-clean. Formatting it here would hide a
    # template bug, so this only checks. Note gofmt -l exits 0 either way.
    GOFMT_OUT="$(gofmt -l ./generated)"
    if [ -z "$GOFMT_OUT" ]; then
        pass "Generated Go is gofmt-clean"
    else
        fail "Generated Go is not gofmt-clean: $GOFMT_OUT"
        exit 1
    fi
    if go vet ./... && go build -o demo-go .; then
        pass "Go demo built successfully"
        GO_AVAILABLE=true
    else
        fail "Go demo build failed"
        GO_AVAILABLE=false
    fi
else
    info "Go not available, skipping Go tests"
    GO_AVAILABLE=false
fi

# Step 5: Test Rust → TypeScript
echo ""
echo "========================================="
echo "Step 5: Testing Rust → TypeScript"
echo "========================================="

# Rust writes JSON files
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to write JSON files..."
cargo run --quiet -- write --output "$RUST_TO_TS_DIR" 2>/dev/null || true

# Verify Rust wrote files
if [ -f "$RUST_TO_TS_DIR/user.json" ] && [ -f "$RUST_TO_TS_DIR/user_minimal.json" ]; then
    pass "Rust wrote JSON files successfully"
else
    fail "Rust failed to write JSON files"
fi

# TypeScript reads and validates Rust's JSON
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to read Rust's JSON files..."
if node dist/src/index.js --read "$RUST_TO_TS_DIR" 2>/dev/null; then
    pass "TypeScript successfully read Rust's JSON files"
else
    # Try with ts-node
    if npx ts-node src/index.ts --read "$RUST_TO_TS_DIR" 2>/dev/null; then
        pass "TypeScript successfully read Rust's JSON files (ts-node)"
    else
        fail "TypeScript failed to read Rust's JSON files"
    fi
fi

# Step 6: Test TypeScript → Rust
echo ""
echo "========================================="
echo "Step 6: Testing TypeScript → Rust"
echo "========================================="

# TypeScript writes JSON files
cd "$SCRIPT_DIR/../../examples/demo-ts"
info "Running TypeScript to write JSON files..."
if node dist/src/index.js --write "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "TypeScript wrote JSON files successfully"
else
    # Try with ts-node
    if npx ts-node src/index.ts --write "$TS_TO_RUST_DIR" 2>/dev/null; then
        pass "TypeScript wrote JSON files successfully (ts-node)"
    else
        fail "TypeScript failed to write JSON files"
    fi
fi

# Verify TypeScript wrote files
if [ -f "$TS_TO_RUST_DIR/user.json" ] && [ -f "$TS_TO_RUST_DIR/user_event_created.json" ]; then
    pass "TypeScript JSON files exist"
else
    fail "TypeScript failed to write JSON files"
fi

# Rust reads and validates TypeScript's JSON
cd "$SCRIPT_DIR/../../examples/demo"
info "Running Rust demo to read TypeScript's JSON files..."
if cargo run --quiet -- read --input "$TS_TO_RUST_DIR" 2>/dev/null; then
    pass "Rust successfully read TypeScript's JSON files"
else
    fail "Rust failed to read TypeScript's JSON files"
fi

# Step 7: Test Rust → Swift (if Swift available)
if [ "$SWIFT_AVAILABLE" = true ]; then
    echo ""
    echo "========================================="
    echo "Step 7: Testing Rust → Swift"
    echo "========================================="

    # Rust writes JSON files for Swift
    cd "$SCRIPT_DIR/../../examples/demo"
    info "Running Rust demo to write JSON files for Swift..."
    cargo run --quiet -- write --output "$RUST_TO_SWIFT_DIR" 2>/dev/null || true

    # Verify Rust wrote files
    if [ -f "$RUST_TO_SWIFT_DIR/user.json" ]; then
        pass "Rust wrote JSON files for Swift"
    else
        fail "Rust failed to write JSON files for Swift"
    fi

    # Swift reads and validates Rust's JSON
    cd "$SCRIPT_DIR/../../examples/demo-swift"
    info "Running Swift demo to read Rust's JSON files..."
    if swift run Demo --read "$RUST_TO_SWIFT_DIR" 2>/dev/null; then
        pass "Swift successfully read Rust's JSON files"
    else
        fail "Swift failed to read Rust's JSON files"
    fi

    # Step 8: Test Swift → Rust
    echo ""
    echo "========================================="
    echo "Step 8: Testing Swift → Rust"
    echo "========================================="

    # Swift writes JSON files
    cd "$SCRIPT_DIR/../../examples/demo-swift"
    info "Running Swift demo to write JSON files..."
    if swift run Demo --write "$SWIFT_TO_RUST_DIR" 2>/dev/null; then
        pass "Swift wrote JSON files successfully"
    else
        fail "Swift failed to write JSON files"
    fi

    # Verify Swift wrote files
    if [ -f "$SWIFT_TO_RUST_DIR/user.json" ]; then
        pass "Swift JSON files exist"
    else
        fail "Swift failed to write JSON files"
    fi

    # Rust reads and validates Swift's JSON
    cd "$SCRIPT_DIR/../../examples/demo"
    info "Running Rust demo to read Swift's JSON files..."
    if cargo run --quiet -- read --input "$SWIFT_TO_RUST_DIR" 2>/dev/null; then
        pass "Rust successfully read Swift's JSON files"
    else
        fail "Rust failed to read Swift's JSON files"
    fi
fi

# Steps 9-10: Go legs (if Go available)
if [ "$GO_AVAILABLE" = true ]; then
    echo ""
    echo "========================================="
    echo "Step 9: Testing Rust → Go"
    echo "========================================="

    cd "$SCRIPT_DIR/../../examples/demo"
    info "Running Rust demo to write JSON files for Go..."
    cargo run --quiet -- write --output "$RUST_TO_GO_DIR" 2>/dev/null || true

    if [ -f "$RUST_TO_GO_DIR/user.json" ]; then
        pass "Rust wrote JSON files for Go"
    else
        fail "Rust failed to write JSON files for Go"
    fi

    cd "$SCRIPT_DIR/../../examples/demo-go"
    info "Running Go demo to read Rust's JSON files..."
    if ./demo-go --read "$RUST_TO_GO_DIR" >/dev/null; then
        pass "Go successfully read Rust's JSON files"
    else
        fail "Go failed to read Rust's JSON files"
    fi

    echo ""
    echo "========================================="
    echo "Step 10: Testing Go → Rust"
    echo "========================================="

    info "Running Go demo to write JSON files..."
    if ./demo-go --write "$GO_TO_RUST_DIR" >/dev/null; then
        pass "Go wrote JSON files successfully"
    else
        fail "Go failed to write JSON files"
    fi

    if [ -f "$GO_TO_RUST_DIR/user.json" ] && [ -f "$GO_TO_RUST_DIR/user_event_created.json" ]; then
        pass "Go JSON files exist"
    else
        fail "Go failed to write JSON files"
    fi

    cd "$SCRIPT_DIR/../../examples/demo"
    info "Running Rust demo to read Go's JSON files..."
    if cargo run --quiet -- read --input "$GO_TO_RUST_DIR" 2>/dev/null; then
        pass "Rust successfully read Go's JSON files"
    else
        fail "Rust failed to read Go's JSON files"
    fi
fi

# Final Summary
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All interoperability tests passed!${NC}"
    echo ""
    echo "Tested scenarios:"
    echo "  ✓ Rust serializes → TypeScript deserializes"
    echo "  ✓ TypeScript serializes → Rust deserializes"
    if [ "$SWIFT_AVAILABLE" = true ]; then
        echo "  ✓ Rust serializes → Swift deserializes"
        echo "  ✓ Swift serializes → Rust deserializes"
    fi
    if [ "$GO_AVAILABLE" = true ]; then
        echo "  ✓ Rust serializes → Go deserializes"
        echo "  ✓ Go serializes → Rust deserializes"
    fi
    echo "  ✓ Multi-package types (common, users, orders, notifications)"
    echo "  ✓ Adjacently tagged unions"
    echo "  ✓ Cross-package type imports"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
