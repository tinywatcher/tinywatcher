# Logstorm

A high-performance log generator for stress testing `tinywatcher`.

## Features

- **High throughput**: Generate thousands of log lines per second
- **Multiple patterns**: Various log formats (JSON, plain text, structured)
- **Error injection**: Simulate error conditions and anomalies
- **Burst mode**: Generate sudden traffic spikes
- **Multiple outputs**: Write to files, stdout, or multiple destinations
- **Realistic logs**: Generates logs that mimic real-world applications

## Usage

### Basic usage - generate logs to stdout
```bash
cargo run --bin logstorm
```

### Generate logs to a file
```bash
cargo run --bin logstorm -- --output /tmp/test.log
```

### High-volume stress test (50,000 logs/sec)
```bash
cargo run --release --bin logstorm -- --rate 50000 --output /tmp/stress-50k.log --stats
```

### Generate logs with variable sizes for regex testing
```bash
cargo run --release --bin logstorm -- --rate 10000 --line-size variable --output /tmp/variable.log
```

### Generate complex patterns (stack traces, SQL, URLs, IPs)
```bash
cargo run --release --bin logstorm -- --rate 10000 --complex-patterns --line-size xl --error-rate 0.15 --output /tmp/complex.log
```

### Generate logs with error patterns
```bash
cargo run --bin logstorm -- --error-rate 0.05 --output /tmp/errors.log
```

### Burst mode - periodic traffic spikes
```bash
cargo run --bin logstorm -- --burst --burst-interval 30 --output /tmp/burst.log
```

### JSON format logs
```bash
cargo run --bin logstorm -- --format json --output /tmp/json.log
```

## CLI Options

- `--rate <NUM>`: Logs per second (default: 100, tested up to 50,000+)
- `--duration <SECS>`: How long to run (default: infinite)
- `--output <PATH>`: Output file path (default: stdout)
- `--format <FORMAT>`: Log format: text, json, apache, nginx (default: text)
- `--line-size <SIZE>`: short, medium, long, xl, variable (default: medium)
- `--batch-size <NUM>`: Batch size for writes, higher = better performance (default: 100)
- `--error-rate <RATE>`: Probability of error logs 0.0-1.0 (default: 0.01)
- `--complex-patterns`: Generate complex patterns like stack traces, SQL, URLs
- `--burst`: Enable burst mode
- `--burst-interval <SECS>`: Seconds between bursts (default: 60)
- `--burst-multiplier <NUM>`: Traffic multiplier during bursts (default: 10)
- `--stats`: Show real-time statistics

## Use Cases

### Test Pattern Matching
```bash
# Generate logs with specific error patterns
cargo run --bin logstorm -- --error-rate 0.1 --output /tmp/pattern-test.log
```

### Test Threshold Alerts
```bash
# Generate high-volume logs to trigger rate thresholds
cargo run --bin logstorm -- --rate 5000 --duration 60 --output /tmp/threshold-test.log
```

### Test File Rotation
```bash
# Generate large volumes to test file rotation handling
cargo run --bin logstorm -- --rate 10000 --duration 300 --output /tmp/rotation-test.log
```

### Test Multiple Sources
```bash
# Start multiple logstorm instances
cargo run --bin logstorm -- --output /tmp/app1.log --rate 100 &
cargo run --bin logstorm -- --output /tmp/app2.log --rate 200 &
cargo run --bin logstorm -- --output /tmp/app3.log --rate 150 &
```

## Example with tinywatcher

1. Start logstorm:
```bash
cargo run --bin logstorm -- --rate 1000 --error-rate 0.05 --output /tmp/stress-test.log
```

2. Configure tinywatcher to monitor the file:
```yaml
sources:
  - path: /tmp/stress-test.log
    patterns:
      - pattern: "ERROR"
        message: "Error detected in stress test"
      - pattern: "CRITICAL"
        message: "Critical error in stress test"

alerts:
  - type: stdout
```

3. Run tinywatcher:
```bash
cargo run --bin tinywatcher -- --config stress-test-config.yaml
```
