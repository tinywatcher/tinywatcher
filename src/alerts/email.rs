use super::AlertHandler;
use async_trait::async_trait;
use anyhow::{Result, Context};
use lettre::{
    Message, 
    Transport,
    message::header::ContentType,
};

#[cfg(unix)]
use lettre::SendmailTransport;

#[cfg(not(unix))]
use lettre::SmtpTransport;

pub struct EmailAlert {
    name: String,
    from: String,
    to: Vec<String>,
    #[cfg(not(unix))]
    smtp_server: Option<String>,
}

impl EmailAlert {
    #[cfg(unix)]
    pub fn new(name: String, from: String, to: Vec<String>) -> Self {
        tracing::info!(
            "Created email alert '{}' (sendmail) - from: {}, to: {:?}",
            name, from, to
        );
        Self {
            name,
            from,
            to,
        }
    }

    #[cfg(not(unix))]
    pub fn new(name: String, from: String, to: Vec<String>, smtp_server: Option<String>) -> Self {
        tracing::info!(
            "Created email alert '{}' (SMTP: {:?}) - from: {}, to: {:?}",
            name, smtp_server, from, to
        );
        Self {
            name,
            from,
            to,
            smtp_server,
        }
    }
}

#[async_trait]
impl AlertHandler for EmailAlert {
    async fn send(&self, rule_name: &str, message: &str) -> Result<()> {
        tracing::info!(
            "Email alert '{}' triggered for rule '{}' - sending to {} recipient(s)",
            self.name, rule_name, self.to.len()
        );
        
        let subject = format!("🚨 TinyWatcher Alert: {}", rule_name);
        let body = format!(
            "TinyWatcher Alert\n\
             =================\n\n\
             Rule: {}\n\
             Time: {}\n\n\
             Message:\n\
             {}\n",
            rule_name,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            message
        );

        // Send to each recipient
        for recipient in &self.to {
            tracing::debug!("Building email to: {}", recipient);
            let email = Message::builder()
                .from(self.from.parse().context("Invalid from email address")?)
                .to(recipient.parse().context(format!("Invalid to email address: {}", recipient))?)
                .subject(&subject)
                .header(ContentType::TEXT_PLAIN)
                .body(body.clone())
                .context("Failed to build email message")?;

            // Platform-specific email sending
            #[cfg(unix)]
            {
                // Use sendmail on Unix systems (macOS, Linux)
                tracing::debug!("Using sendmail transport for {}", recipient);
                let sender = SendmailTransport::new();
                match sender.send(&email) {
                    Ok(_) => {
                        tracing::info!("✅ Successfully sent email alert '{}' to {} for rule: {}", self.name, recipient, rule_name);
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to send email via sendmail to {}: {}", recipient, e);
                        return Err(anyhow::anyhow!("Failed to send email via sendmail to {}: {}", recipient, e));
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // Use SMTP on Windows or when specified
                let smtp_server = self.smtp_server.as_ref()
                    .context("SMTP server must be configured on non-Unix systems")?;
                
                tracing::debug!("Using SMTP transport ({}) for {}", smtp_server, recipient);
                let sender = SmtpTransport::relay(smtp_server)
                    .context("Failed to create SMTP transport")?
                    .build();
                
                match sender.send(&email) {
                    Ok(_) => {
                        tracing::info!("✅ Successfully sent email alert '{}' to {} for rule: {}", self.name, recipient, rule_name);
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to send email via SMTP to {}: {}", recipient, e);
                        return Err(anyhow::anyhow!("Failed to send email via SMTP to {}: {}", recipient, e));
                    }
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
