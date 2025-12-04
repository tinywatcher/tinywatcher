# SendGrid Integration for TinyWatcher

## Overview
SendGrid has been successfully integrated as a new alert destination in TinyWatcher. This provides reliable, API-based email delivery with advanced features like delivery tracking, analytics, and high deliverability rates.

## What Was Added

### 1. New Files
- `src/alerts/sendgrid.rs` - SendGrid alert handler implementation
- `examples/example-sendgrid.yaml` - Example configuration file

### 2. Modified Files
- `src/alerts/mod.rs` - Added SendGrid module and export
- `src/config.rs` - Added SendGrid alert type and options with environment variable expansion
- `src/main.rs` - Added SendGrid alert initialization and display formatting
- `examples/example-all-alerts.yaml` - Added SendGrid example
- `README.md` - Updated documentation with SendGrid details

## Configuration

```yaml
alerts:
  sendgrid-critical:
    type: sendgrid
    api_key: "${SENDGRID_API_KEY}"  # Can use environment variables
    from: "alerts@yourdomain.com"    # Must be verified in SendGrid
    to:
      - "admin@yourdomain.com"
      - "oncall@yourdomain.com"
```

## Setup Instructions

1. **Sign up for SendGrid** at https://sendgrid.com
2. **Create an API key** in SendGrid dashboard (Settings > API Keys)
3. **Verify your sender email/domain** in SendGrid
4. **Set environment variable**: 
   ```bash
   export SENDGRID_API_KEY="your-api-key-here"
   ```
5. **Configure TinyWatcher** with the SendGrid alert as shown above

## Features

- ✅ Multiple recipients per alert
- ✅ Environment variable support for API keys
- ✅ Proper error handling and logging
- ✅ Consistent message formatting with other alerts
- ✅ Identity and hostname tracking in alerts
- ✅ Timestamp in alert messages

## Differences from Email Alert

The existing `email` alert type uses:
- **Unix**: sendmail (local mail transfer agent)
- **Windows**: SMTP connection

The new `sendgrid` alert type uses:
- **All platforms**: SendGrid REST API (v3)
- Requires API key instead of SMTP credentials
- Better deliverability and tracking
- No local mail server required

## Testing

The integration has been tested and validated:
```bash
./target/release/tinywatcher test --config examples/example-sendgrid.yaml
```

Configuration parses correctly and displays SendGrid alerts properly.

## API Usage

SendGrid alert uses the v3 Mail Send API:
- Endpoint: `https://api.sendgrid.com/v3/mail/send`
- Authentication: Bearer token (API key)
- Format: JSON payload with personalizations for each recipient
- Response: Success (2xx) or detailed error message

## Example Output

When a rule triggers:
```
✅ Successfully sent SendGrid alert 'sendgrid-critical' to 2 recipient(s) for rule: critical-error
```

Alert message format:
```
Subject: 🚨 TinyWatcher Alert: critical-error (production-server)

Body:
TinyWatcher Alert
=================

Host: production-server
Rule: critical-error
Time: 2025-12-04 13:52:53

Message:
[Your log message here]
```
