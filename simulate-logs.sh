#!/bin/bash

# simulate-logs.sh - Generate realistic log entries for testing TinyWatcher

LOG_FILE="${1:-/tmp/test-app.log}"
INTERVAL="${2:-2}"

echo "Starting log simulator..."
echo "Writing to: $LOG_FILE"
echo "⏱ Interval: ${INTERVAL}s"
echo "Press Ctrl+C to stop"
echo ""

# Create log file if it doesn't exist
touch "$LOG_FILE"

# Array of log levels
LEVELS=("INFO" "DEBUG" "WARNING" "ERROR" "CRITICAL" "FATAL")

# Array of sample messages
INFO_MESSAGES=(
    "User logged in successfully"
    "Request processed in 45ms"
    "Cache hit for user data"
    "Database query executed"
    "API endpoint /api/users called"
    "Session created for user"
    "Email sent successfully"
    "Payment processed"
    "Report generated"
    "Background job completed"
)

WARNING_MESSAGES=(
    "High memory usage detected: 78%"
    "Slow database query: 2.5s"
    "API rate limit approaching"
    "Cache miss rate above threshold"
    "Deprecated API endpoint used"
    "Connection pool nearly exhausted"
    "Disk space running low: 85% used"
    "Large payload detected: 5MB"
)

ERROR_MESSAGES=(
    "Database connection timeout"
    "Failed to connect to Redis"
    "500 Internal Server Error on /api/orders"
    "Authentication failed for user"
    "File not found: /var/data/config.json"
    "Permission denied accessing resource"
    "Invalid JSON in request body"
    "Network timeout after 30s"
    "OperationalError: database is locked"
    "Connection refused to payment gateway"
)

CRITICAL_MESSAGES=(
    "CRITICAL: Out of memory error"
    "CRITICAL: Database connection pool exhausted"
    "CRITICAL: All worker processes crashed"
    "FATAL: Unable to bind to port 8080"
    "FATAL: Configuration file corrupted"
    "CRITICAL: Disk full on /var/log"
)

# Function to get random message by level
get_message() {
    local level=$1
    case $level in
        "INFO"|"DEBUG")
            echo "${INFO_MESSAGES[$RANDOM % ${#INFO_MESSAGES[@]}]}"
            ;;
        "WARNING")
            echo "${WARNING_MESSAGES[$RANDOM % ${#WARNING_MESSAGES[@]}]}"
            ;;
        "ERROR")
            echo "${ERROR_MESSAGES[$RANDOM % ${#ERROR_MESSAGES[@]}]}"
            ;;
        "CRITICAL"|"FATAL")
            echo "${CRITICAL_MESSAGES[$RANDOM % ${#CRITICAL_MESSAGES[@]}]}"
            ;;
    esac
}

# Function to generate a log entry
generate_log() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Weighted random level selection
    # INFO/DEBUG: 60%, WARNING: 20%, ERROR: 15%, CRITICAL: 5%
    local rand=$((RANDOM % 100))
    local level
    
    if [ $rand -lt 60 ]; then
        level=$([ $((RANDOM % 2)) -eq 0 ] && echo "INFO" || echo "DEBUG")
    elif [ $rand -lt 80 ]; then
        level="WARNING"
    elif [ $rand -lt 95 ]; then
        level="ERROR"
    else
        level=$([ $((RANDOM % 2)) -eq 0 ] && echo "CRITICAL" || echo "FATAL")
    fi
    
    local message=$(get_message "$level")
    local log_entry="[$timestamp] $level: $message"
    
    echo "$log_entry" >> "$LOG_FILE"
    echo "$log_entry"
}

# Main loop
counter=0
while true; do
    generate_log
    counter=$((counter + 1))
    
    # Occasionally generate a burst of errors (10% chance)
    if [ $((RANDOM % 100)) -lt 10 ]; then
        echo "💥 Simulating error burst..."
        for i in {1..3}; do
            timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            level="ERROR"
            message=$(get_message "$level")
            log_entry="[$timestamp] $level: $message"
            echo "$log_entry" >> "$LOG_FILE"
            echo "$log_entry"
            sleep 0.5
        done
    fi
    
    sleep "$INTERVAL"
done
