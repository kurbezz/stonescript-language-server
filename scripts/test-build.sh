#!/bin/bash

# Test build script for stonescript-language-server
# This script simulates the CI build process locally

set -e

echo "🔧 StoneScript LSP Build Test"
echo "=============================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from stone-script-lsp directory"
    exit 1
fi

# Check if tree-sitter-stonescript exists
if [ ! -d "../tree-sitter-stonescript" ]; then
    echo "❌ Error: tree-sitter-stonescript not found in parent directory"
    echo "Expected structure:"
    echo "  parent-dir/"
    echo "    ├── stone-script-lsp/"
    echo "    └── tree-sitter-stonescript/"
    exit 1
fi

echo "✅ Directory structure OK"
echo ""

# Check for required files
echo "🔍 Checking tree-sitter files..."
if [ ! -f "../tree-sitter-stonescript/src/parser.c" ]; then
    echo "❌ Error: parser.c not found"
    exit 1
fi
if [ ! -f "../tree-sitter-stonescript/src/scanner.c" ]; then
    echo "❌ Error: scanner.c not found"
    exit 1
fi
echo "✅ Tree-sitter source files found"
echo ""

# Clean build
echo "🧹 Cleaning previous builds..."
cargo clean
echo "✅ Clean complete"
echo ""

# Build
echo "🔨 Building stonescript-lsp..."
cargo build --release --bin stonescript-lsp
echo "✅ Build complete"
echo ""

# Check binary
if [ -f "target/release/stonescript-lsp" ]; then
    echo "✅ Binary created successfully"
    ls -lh target/release/stonescript-lsp
    echo ""

    # Test run
    echo "🧪 Testing binary..."
    timeout 2 ./target/release/stonescript-lsp --version 2>&1 || true
    echo ""

    echo "✅ Build test completed successfully!"
else
    echo "❌ Error: Binary not found at target/release/stonescript-lsp"
    exit 1
fi

echo ""
echo "📦 To create a release archive:"
echo "  cd target/release"
echo "  tar czf stonescript-lsp.tar.gz stonescript-lsp"
