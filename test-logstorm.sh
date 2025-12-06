#!/bin/bash
# Comprehensive stress test script for tinywatcher using logstorm

set -e

echo "=========================================="
echo "Logstorm + Tinywatcher Stress Test Suite"
echo "=========================================="
echo ""

# Clean up old test files
echo "Cleaning up old test files..."
rm -f /tmp/stress-*.log /tmp/complex-*.log

echo ""
echo "Test 1: High throughput - 50k lines/sec for 5 seconds"
echo "Expected: ~250k lines generated"
./target/release/logstorm \
    --rate 50000 \
    --duration 5 \
    --output /tmp/stress-50k.log \
    --line-size variable \
    --stats

echo ""
echo "Test 2: Complex patterns with stack traces, SQL, URLs, IPs"
echo "Expected: Complex regex patterns for testing"
./target/release/logstorm \
    --rate 10000 \
    --duration 3 \
    --output /tmp/complex-patterns.log \
    --line-size xl \
    --complex-patterns \
    --error-rate 0.15 \
    --stats

echo ""
echo "Test 3: Burst mode - simulating traffic spikes"
./target/release/logstorm \
    --rate 1000 \
    --duration 10 \
    --output /tmp/stress-burst.log \
    --burst \
    --burst-interval 3 \
    --burst-multiplier 5 \
    --stats &

BURST_PID=$!
sleep 11
kill $BURST_PID 2>/dev/null || true

echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo ""

echo "File sizes:"
ls -lh /tmp/stress-*.log /tmp/complex-*.log | awk '{print $9, $5}'

echo ""
echo "Line counts:"
wc -l /tmp/stress-*.log /tmp/complex-*.log

echo ""
echo "Sample complex pattern (first error):"
grep -m 1 "ERROR" /tmp/complex-patterns.log || echo "No errors found"

echo ""
echo "=========================================="
echo "To test with tinywatcher, run:"
echo "cargo run --bin tinywatcher -- --config examples/logstorm-stress-test.yaml"
echo "=========================================="
