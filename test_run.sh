#!/bin/bash

# Test script for LangDB
# This script demonstrates basic functionality

echo "=== LangDB Test Run ==="
echo ""
echo "Building the project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Build successful!"
echo ""
echo "To run LangDB interactively, use:"
echo "  cargo run --release"
echo ""
echo "Or run the compiled binary:"
echo "  ./target/release/langdb"
echo ""
echo "=== Quick Test ==="
echo "Running a quick test with sample SQL commands..."
echo ""

# Create a test input file
cat > /tmp/langdb_test.sql << 'EOF'
CREATE TABLE test (id INTEGER, name TEXT);
INSERT INTO test VALUES (1, 'Test1'), (2, 'Test2');
SELECT * FROM test;
.exit
EOF

# Run the test
./target/release/langdb < /tmp/langdb_test.sql

echo ""
echo "=== Test Complete ==="
echo ""
echo "For more examples, check demo.sql"
echo "To run interactively: cargo run --release"
