# Email Alerter for TinyWatcher

The email alerter allows you to send alert notifications via email using your system's email capabilities.

## Features

- **Cross-platform support**: 
  - On macOS/Linux: Uses `sendmail` (system email)
  - On Windows: Uses SMTP (requires configuration)
- **Multiple recipients**: Send alerts to multiple email addresses
- **Simple configuration**: Minimal setup required on Unix systems

## Configuration

### Basic Configuration (macOS/Linux)

On Unix systems (macOS and Linux), the email alerter uses the system's `sendmail` command, which is typically available by default:

```yaml
alerts:
  my-email:
    type: email
    from: "tinywatcher@yourdomain.com"
    to:
      - "admin@example.com"
      - "team@example.com"
```

### Windows Configuration

On Windows, you need to specify an SMTP server:

```yaml
alerts:
  my-email:
    type: email
    from: "tinywatcher@yourdomain.com"
    to:
      - "admin@example.com"
    smtp_server: "localhost"  # or your SMTP server address
```

## Example Usage

### Single Recipient

```yaml
alerts:
  critical-email:
    type: email
    from: "alerts@myapp.com"
    to:
      - "oncall@myteam.com"

rules:
  - name: "Database Error"
    pattern: "(?i)database.*error"
    alert: critical-email
    cooldown: 300
```

### Multiple Recipients

```yaml
alerts:
  team-email:
    type: email
    from: "monitoring@myapp.com"
    to:
      - "devops@myteam.com"
      - "engineering@myteam.com"
      - "oncall@myteam.com"

rules:
  - name: "Service Down"
    pattern: "(?i)service.*unavailable"
    alert: team-email
    cooldown: 600
```

### Combined with Other Alerts

```yaml
alerts:
  email-alert:
    type: email
    from: "alerts@myapp.com"
    to:
      - "admin@myteam.com"
  
  console:
    type: stdout

rules:
  - name: "Critical Error"
    pattern: "(?i)critical|fatal"
    alert:
      - email-alert
      - console
    cooldown: 300
```

## System Requirements

### macOS
- No additional configuration needed
- Uses the built-in `sendmail` command
- Emails are typically sent through system mail or configured SMTP

### Linux
- Requires `sendmail` or compatible MTA (Mail Transfer Agent)
- Common options:
  - `postfix`
  - `sendmail`
  - `exim`
  
To install on Debian/Ubuntu:
```bash
sudo apt-get install sendmail
# or
sudo apt-get install postfix
```

To install on RedHat/CentOS:
```bash
sudo yum install sendmail
# or
sudo yum install postfix
```

### Windows
- Requires SMTP server configuration
- Can use:
  - Local SMTP server
  - External SMTP service (Gmail, SendGrid, etc.)
  - Corporate mail server

## Email Format

Alerts are sent in plain text format with the following structure:

```
Subject: 🚨 TinyWatcher Alert: [Rule Name]

TinyWatcher Alert
=================

Rule: [Rule Name]
Time: [Timestamp]

Message:
[Alert Message/Matched Log Line]
```

## Troubleshooting

### macOS/Linux: "Failed to send email via sendmail"

1. Check if sendmail is installed:
   ```bash
   which sendmail
   ```

2. Test sendmail manually:
   ```bash
   echo "Test email" | sendmail -v your@email.com
   ```

3. Check mail logs:
   ```bash
   tail -f /var/log/mail.log  # Debian/Ubuntu
   tail -f /var/log/maillog   # RedHat/CentOS
   ```

### Windows: "SMTP server must be configured"

Make sure you've specified the `smtp_server` option in your configuration:

```yaml
alerts:
  my-email:
    type: email
    from: "alerts@example.com"
    to:
      - "recipient@example.com"
    smtp_server: "smtp.gmail.com"  # or your SMTP server
```

### Email not received

1. Check spam/junk folder
2. Verify email addresses are correct
3. Check TinyWatcher logs for error messages
4. Verify system mail configuration
5. Test with a simple test alert

## Security Notes

- Email addresses in configuration files should be valid
- Consider using application-specific passwords for SMTP authentication
- Email content may contain sensitive log information
- Use appropriate cooldown periods to avoid email flooding
- Consider firewall rules if using external SMTP servers

## Performance Considerations

- Email sending is non-blocking and doesn't affect log monitoring
- Configure appropriate cooldown periods to prevent alert fatigue
- Multiple recipients are processed sequentially
- Failed email sends are logged but don't stop the monitoring process

## Dependencies

The email alerter uses the `lettre` crate (v0.11), which provides:
- Cross-platform email support
- Both sendmail and SMTP transports
- Async/await support
- Secure email transmission
