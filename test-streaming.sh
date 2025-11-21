#!/bin/bash

# Simple test script for TinyWatcher streaming
# This creates a TCP server that sends log messages

echo "🧪 TinyWatcher Stream Test"
echo "=========================="
echo ""
echo "This script simulates a TCP log stream on localhost:9999"
echo ""

# Create a simple config
cat > /tmp/stream-test.yaml <<'EOF'
inputs:
  streams:
    - name: test_tcp_stream
      type: tcp
      url: "localhost:9999"
      reconnect_delay: 2

alerts:
  stdout:
    type: stdout

rules:
  - name: error_detection
    pattern: "ERROR|FATAL"
    alert: stdout
    cooldown: 5
    
  - name: warning_detection
    pattern: "WARN"
    alert: stdout
    cooldown: 10
EOF

echo "✅ Created test config at /tmp/stream-test.yaml"
echo ""
echo "To test:"
echo "  1. In terminal 1, start the TCP log generator:"
echo "     $0 server"
echo ""
echo "  2. In terminal 2, start TinyWatcher:"
echo "     cargo run -- watch --config /tmp/stream-test.yaml"
echo ""
echo "You should see TinyWatcher detect ERROR and WARN messages!"
echo ""

if [ "$1" = "server" ]; then
    echo "🚀 Starting TCP log server on port 9999..."
    echo "   Press Ctrl+C to stop"
    echo ""
    
    # Use netcat to create a simple TCP server (macOS compatible)
    while true; do
        {
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: Application started"
            sleep 2
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: Processing request..."
            sleep 3
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARN: High memory usage detected"
            sleep 2
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: Request completed"
            sleep 4
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: Database connection timeout"
            sleep 2
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] FATAL: Application crashed"
            sleep 5
        } | nc -l 9999
        
        echo "Connection closed, restarting server..."
        sleep 1
    done
fi
