#!/bin/bash

# Run the test and capture output
cd "$(dirname "$0")/../crates/stonescript-parser"
output=$(cargo test --test validate_scripts 2>&1)

# Extract error lines
echo "$output" | grep "Parse error" | sed 's/Parse error in "//' | sed 's/"//' > /tmp/failing_scripts.txt

# Count
total=$(find ../../test_scripts -name "*.txt" | wc -l | tr -d ' ')
failing=$(wc -l < /tmp/failing_scripts.txt | tr -d ' ')
passing=$((total - failing))

echo "Total scripts: $total"
echo "Passing scripts: $passing"
echo "Failing scripts: $failing"
echo ""
echo "Failing scripts saved to: /tmp/failing_scripts.txt"
