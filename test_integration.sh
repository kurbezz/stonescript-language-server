#!/usr/bin/env bash
# Integration test: Parse all StoneScript files in test_scripts directory

set -e

cd "$(dirname "$0")"

echo "Building parser..."
cargo build --release -p stonescript-parser --quiet

echo ""
echo "Testing parser against all StoneScript files in test_scripts/"
echo "=============================================================="

passed=0
failed=0
total=0

# Create temporary Rust program to test parsing
cat > /tmp/test_parser.rs << 'RUST_EOF'
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        process::exit(1);
    }
    
    let file_path = &args[1];
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", file_path, e);
            process::exit(1);
        }
    };
    
    // Use the parser from the workspace
    match stonescript_parser::parse_source(&content) {
        Ok(_program) => {
            println!("✓ {}", file_path);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("✗ {} - Parse error: {}", file_path, e);
            process::exit(1);
        }
    }
}
RUST_EOF

# Test a few sample files first
echo "Testing sample files..."
for file in test_scripts/*.txt; do
    [ -f "$file" ] || continue
    total=$((total + 1))
    
    if cargo run --release -p stonescript-parser --example parse_file "$file" >/dev/null 2>&1; then
        echo "✓ $(basename "$file")"
        passed=$((passed + 1))
    else
        # Try direct parsing
        if cargo run --quiet --release -p stonescript-parser --bin test-parse "$file" 2>/dev/null; then
            echo "✓ $(basename "$file")"
            passed=$((passed + 1))
        else
            echo "✗ $(basename "$file") - Parse failed"
            failed=$((failed + 1))
        fi
    fi
done

echo ""
echo "=============================================================="
echo "Results:"
echo "  Total:  $total files"
echo "  Passed: $passed"
echo "  Failed: $failed"
echo ""

if [ $failed -eq 0 ]; then
    echo "✓ All tests passed!"
    exit 0
else
    echo "✗ Some tests failed"
    exit 1
fi
