# Logstorm + TinyWatcher Testing Guide

## Quick Start

### 1. Build Everything
```bash
cargo build --release
```

### 2. Generate Test Logs

**High-volume stress test (50k lines/sec):**
```bash
./target/release/logstorm \
    --rate 50000 \
    --duration 5 \
    --output /tmp/stress-50k.log \
    --line-size variable \
    --stats
```

**Complex patterns for regex testing:**
```bash
./target/release/logstorm \
    --rate 10000 \
    --duration 3 \
    --output /tmp/complex-patterns.log \
    --line-size xl \
    --complex-patterns \
    --error-rate 0.15 \
    --stats
```

### 3. Monitor with TinyWatcher

```bash
./target/release/tinywatcher --config examples/logstorm-stress-test.yaml
```

## Performance Results

### Throughput Test
- **Target**: 50,000 lines/sec
- **Duration**: 5 seconds
- **Lines Generated**: ~250,000
- **File Size**: ~82 MB
- **Avg Line Size**: 342 bytes

### Complex Pattern Test
- **Target**: 10,000 lines/sec
- **Duration**: 3 seconds
- **Lines Generated**: ~30,000
- **File Size**: ~17 MB
- **Avg Line Size**: 545 bytes
- **Error Rate**: 15%

### Pattern Types Generated
- Stack traces (Java-style)
- SQL queries
- HTTP URLs
- IP addresses
- User agents
- JSON payloads
- Connection errors
- Memory errors
- Timeouts

## Testing Scenarios

### 1. Regex Performance Testing
```bash
# Generate varying line sizes
./target/release/logstorm --rate 10000 --line-size variable --output /tmp/regex-test.log

# Test complex patterns
grep -E "Exception|SQLException|TimeoutException" /tmp/regex-test.log | wc -l
```

### 2. Threshold Testing
```bash
# Generate high error rate
./target/release/logstorm --rate 5000 --error-rate 0.2 --duration 60 --output /tmp/threshold-test.log
```

### 3. Burst Testing
```bash
# Simulate traffic spikes
./target/release/logstorm --rate 1000 --burst --burst-interval 30 --burst-multiplier 10 --output /tmp/burst-test.log
```

### 4. Format Testing
```bash
# JSON logs
./target/release/logstorm --rate 1000 --format json --output /tmp/json-test.log

# Apache logs
./target/release/logstorm --rate 1000 --format apache --output /tmp/apache-test.log

# Nginx logs
./target/release/logstorm --rate 1000 --format nginx --output /tmp/nginx-test.log
```

## Architecture Benefits

### Why a Sub-Crate?
1. **Separation of concerns**: Testing tools separate from production code
2. **Optional dependency**: Doesn't bloat the main binary
3. **Workspace benefits**: Shared dependencies, single build
4. **Independent development**: Can evolve separately

### Project Structure
```
tinywatcher/
├── Cargo.toml (workspace root)
├── tinywatcher/ (main application)
│   ├── Cargo.toml
│   └── src/
└── logstorm/ (stress testing tool)
    ├── Cargo.toml
    ├── README.md
    └── src/
```

## CLI Options Reference

### Logstorm
- `--rate <NUM>`: Logs per second (tested up to 50,000+)
- `--duration <SECS>`: How long to run (0 = infinite)
- `--output <PATH>`: Output file (default: stdout)
- `--format <FORMAT>`: text, json, apache, nginx
- `--line-size <SIZE>`: short, medium, long, xl, variable
- `--batch-size <NUM>`: Batch size for writes (default: 100)
- `--error-rate <RATE>`: Error probability 0.0-1.0
- `--complex-patterns`: Generate stack traces, SQL, URLs, etc.
- `--burst`: Enable burst mode
- `--burst-interval <SECS>`: Seconds between bursts
- `--burst-multiplier <NUM>`: Traffic multiplier during bursts
- `--stats`: Show real-time statistics

### Performance Tips
1. Use `--release` builds for accurate performance testing
2. For 50k+ logs/sec, use `--batch-size 200` or higher
3. Write to fast storage (SSD) or tmpfs for maximum throughput
4. Use `--stats` to verify actual throughput
5. For regex testing, use `--complex-patterns` and `--line-size variable`

## Example Workflow

```bash
# 1. Start generating logs
./target/release/logstorm \
    --rate 50000 \
    --output /tmp/test.log \
    --line-size variable \
    --complex-patterns \
    --stats &

LOGSTORM_PID=$!

# 2. Wait a moment for logs to accumulate
sleep 2

# 3. Start monitoring
./target/release/tinywatcher --config examples/logstorm-stress-test.yaml &

TINYWATCHER_PID=$!

# 4. Run for desired duration
sleep 30

# 5. Clean up
kill $LOGSTORM_PID $TINYWATCHER_PID
```

## Troubleshooting

### "Rate is lower than expected"
- Check disk I/O with `iostat`
- Use tmpfs: `--output /tmp/test.log` (often in RAM on Linux)
- Increase `--batch-size`
- Ensure release build

### "Line sizes not varying"
- Verify `--line-size variable` is set
- Check with: `awk '{print length}' /tmp/test.log | sort -n | uniq -c`

### "Not seeing complex patterns"
- Ensure `--complex-patterns` flag is set
- Check with: `grep -E "(Exception|SELECT|https://)" /tmp/test.log`
