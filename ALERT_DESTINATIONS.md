# Alert Destinations

TinyWatcher supports multiple alert destinations, allowing you to send notifications to various platforms and services.

## Table of Contents

1. [Discord](#discord)
2. [Telegram](#telegram)
3. [PagerDuty](#pagerduty)
4. [Ntfy.sh](#ntfysh)
5. [Slack](#slack)
6. [Webhook](#webhook)
7. [Email](#email)
8. [Stdout](#stdout)

---

## Discord

Send alerts to Discord channels using webhooks. Perfect for team collaboration and real-time notifications.

### Setup

1. Open your Discord server settings
2. Navigate to **Integrations** → **Webhooks**
3. Click **New Webhook** or edit an existing one
4. Copy the webhook URL
5. Add it to your configuration

### Configuration

```yaml
alerts:
  my-discord:
    type: discord
    url: "https://discord.com/api/webhooks/123456789/your-webhook-token"

rules:
  - name: error-alerts
    pattern: "ERROR"
    alert: my-discord
    cooldown: 300
```

### Features

- **Rich embeds** with color-coded messages
- **Formatted fields** showing host and rule details
- **Timestamps** for each alert
- **Easy setup** - just need a webhook URL

---

## Telegram

Send notifications via Telegram Bot API. Ideal for personal alerts and mobile-first notifications.

### Setup

1. Talk to [@BotFather](https://t.me/BotFather) on Telegram
2. Create a new bot: `/newbot`
3. Copy the bot token
4. Get your chat ID:
   - Start a chat with your bot
   - Send any message
   - Visit: `https://api.telegram.org/bot<BOT_TOKEN>/getUpdates`
   - Find `"chat":{"id": YOUR_CHAT_ID}` in the response
5. For group chats, add the bot to your group and use the group's chat ID

### Configuration

```yaml
alerts:
  telegram-alerts:
    type: telegram
    bot_token: "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
    chat_id: "987654321"  # Or group ID (can be negative)

rules:
  - name: critical-errors
    pattern: "CRITICAL|FATAL"
    alert: telegram-alerts
    cooldown: 300
```

### Features

- **Mobile-first** notifications
- **Markdown formatting** support
- **Works anywhere** with internet access
- **Group chat support** for team notifications
- **Free and fast** delivery

---

## PagerDuty

Enterprise-grade incident management integration using Events API v2. Perfect for on-call teams and production systems.

### Setup

1. Log in to your PagerDuty account
2. Go to **Services** → **Service Directory**
3. Select a service or create a new one
4. Go to the **Integrations** tab
5. Click **Add an integration**
6. Select **Events API V2**
7. Copy the **Integration Key** (routing key)

### Configuration

```yaml
alerts:
  pagerduty-oncall:
    type: pagerduty
    routing_key: "your-integration-key-here"

rules:
  - name: production-errors
    pattern: "ERROR|Exception"
    alert: pagerduty-oncall
    cooldown: 600
```

### Features

- **Automatic incident creation** with proper severity
- **On-call escalation** policies
- **Rich incident details** including custom fields
- **Integration with alerting rules**
- **Production-ready** and battle-tested

### Alert Payload

Alerts sent to PagerDuty include:
- **Summary**: Alert name and host
- **Severity**: Always set to "error"
- **Source**: Host/identity name
- **Component**: "TinyWatcher"
- **Custom details**: Full message, rule name, alert name

---

## Ntfy.sh

The simplest possible push notification service. No authentication required for basic usage!

### Setup

1. Choose a unique topic name (e.g., `tinywatcher-alerts-xyz123`)
2. Subscribe on your device:
   - **iOS**: [App Store](https://apps.apple.com/us/app/ntfy/id1625396347)
   - **Android**: [Play Store](https://play.google.com/store/apps/details?id=io.heckel.ntfy)
   - **Web**: Visit `https://ntfy.sh/your-topic-name`
3. Start receiving notifications!

### Configuration

**Public ntfy.sh server:**
```yaml
alerts:
  ntfy-public:
    type: ntfy
    topic: "tinywatcher-alerts-xyz123"
```

**Self-hosted server:**
```yaml
alerts:
  ntfy-private:
    type: ntfy
    topic: "alerts"
    server: "https://ntfy.example.com"
```

### Features

- **No authentication** required for basic usage
- **Works immediately** - just pick a topic
- **Mobile apps** for iOS and Android
- **Self-hosting** supported
- **High priority** notifications
- **Custom tags** and emojis

### Security Note

Since the public ntfy.sh server requires no authentication, choose a unique, hard-to-guess topic name to prevent others from subscribing to your alerts. For production use, consider using authentication features or self-hosting.

---

## Slack

Send alerts to Slack channels using incoming webhooks.

### Setup

1. Go to your Slack workspace
2. Navigate to **Apps** → **Manage**
3. Search for **Incoming Webhooks** and add to your workspace
4. Choose a channel and create webhook
5. Copy the webhook URL

### Configuration

```yaml
alerts:
  slack-team:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### Features

- **Formatted messages** with emojis
- **Code blocks** for log excerpts
- **Custom username** and icon
- **Channel integration**

---

## Webhook

Generic webhook support for custom integrations.

### Configuration

```yaml
alerts:
  custom-webhook:
    type: webhook
    url: "https://your-service.com/webhook"
```

### Payload Format

```json
{
  "identity": "hostname",
  "rule": "rule-name",
  "message": "log message",
  "timestamp": "2025-11-28T12:00:00Z",
  "alert_name": "custom-webhook"
}
```

---

## Email

Send alerts via email using sendmail (Unix) or SMTP.

### Configuration

**Unix (using sendmail):**
```yaml
alerts:
  email-admin:
    type: email
    from: "alerts@example.com"
    to:
      - "admin@example.com"
```

**Windows/SMTP:**
```yaml
alerts:
  email-admin:
    type: email
    from: "alerts@example.com"
    to:
      - "admin@example.com"
    smtp_server: "smtp.gmail.com:587"
```

---

## Stdout

Output alerts to standard output (console). Useful for testing.

### Configuration

```yaml
alerts:
  console:
    type: stdout
```

---

## Multi-Destination Alerts

You can send a single alert to multiple destinations:

```yaml
alerts:
  pagerduty-oncall:
    type: pagerduty
    routing_key: "key123"
  
  telegram-personal:
    type: telegram
    bot_token: "token123"
    chat_id: "chat123"
  
  discord-team:
    type: discord
    url: "https://discord.com/api/webhooks/..."

rules:
  - name: critical-error
    pattern: "CRITICAL"
    alert:
      - pagerduty-oncall
      - telegram-personal
      - discord-team
    cooldown: 300
```

---

## Environment Variables

All alert configurations support environment variable expansion:

```yaml
alerts:
  telegram-alerts:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    chat_id: "${TELEGRAM_CHAT_ID}"
  
  discord-alerts:
    type: discord
    url: "${DISCORD_WEBHOOK_URL}"
  
  ntfy-alerts:
    type: ntfy
    topic: "tinywatcher-${HOSTNAME}"
```

---

## Best Practices

1. **Use multiple destinations for critical alerts** - Ensure important alerts reach you through redundant channels
2. **Set appropriate cooldowns** - Prevent alert fatigue by spacing out notifications
3. **Choose the right tool for the job**:
   - **PagerDuty**: Production incidents, on-call management
   - **Telegram/Ntfy**: Personal mobile notifications
   - **Discord/Slack**: Team collaboration and awareness
   - **Email**: Formal notifications and audit trails
4. **Test your configuration** - Verify alerts work before relying on them
5. **Secure your credentials** - Use environment variables for sensitive data
6. **Use unique topic names** - Especially for ntfy.sh public server

---

## See Also

- [Example Configurations](./example-all-alerts.yaml)
- [Source Filtering](./SOURCE_FILTERING.md)
- [Health Checks](./HEALTH_CHECKS.md)
