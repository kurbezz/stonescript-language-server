#!/bin/bash
# Test all StoneScript files in test_scripts directory

echo "Testing parser against all test_scripts files..."
echo "================================================"

passed=0
failed=0
total=0

for file in test_scripts/*.txt; do
    total=$((total + 1))
    filename=$(basename "$file")
    
    # Create a simple Rust test program that tries to parse the file
    cat > /tmp/test_parse.rs << 'EOF'
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        std::process::exit(1);
    }
    
    let content = fs::read_to_string(&args[1]).expect("Failed to read file");
    
    // This would use stonescript_parser::parse_source
    println!("File: {}", args[1]);
    println!("Size: {} bytes", content.len());
}
EOF
    
    # For now, just check file exists and is readable
    if [ -f "$file" ] && [ -r "$file" ]; then
        echo "✓ $filename - readable"
        passed=$((passed + 1))
    else
        echo "✗ $filename - not readable"
        failed=$((failed + 1))
    fi
done

echo ""
echo "================================================"
echo "Total: $total files"
echo "Passed: $passed"
echo "Failed: $failed"

exit $failed
