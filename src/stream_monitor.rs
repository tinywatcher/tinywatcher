use crate::alerts::AlertManager;
use crate::config::{Rule, StreamConfig, StreamType};
use anyhow::{Context, Result};
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct StreamMonitor {
    rules: Vec<CompiledRule>,
    alert_manager: Arc<AlertManager>,
}

struct CompiledRule {
    name: String,
    regex: Regex,
    alert_names: Vec<String>,
    cooldown: u64,
}

impl StreamMonitor {
    pub fn new(rules: Vec<Rule>, alert_manager: Arc<AlertManager>) -> Result<Self> {
        let compiled_rules = rules
            .into_iter()
            .map(|rule| {
                let regex = Regex::new(&rule.pattern)
                    .with_context(|| format!("Invalid regex pattern in rule: {}", rule.name))?;
                Ok(CompiledRule {
                    name: rule.name,
                    regex,
                    alert_names: rule.alert,
                    cooldown: rule.cooldown,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            rules: compiled_rules,
            alert_manager,
        })
    }

    pub async fn watch_stream(&self, stream_config: StreamConfig) -> Result<()> {
        let stream_name = stream_config.get_name();
        tracing::info!("📡 Starting stream monitoring: {}", stream_name);

        loop {
            let result = match stream_config.stream_type {
                StreamType::Websocket => self.watch_websocket(&stream_config).await,
                StreamType::Http => self.watch_http(&stream_config).await,
                StreamType::Tcp => self.watch_tcp(&stream_config).await,
            };

            if let Err(e) = result {
                let reconnect_delay = stream_config.get_reconnect_delay();
                tracing::error!(
                    "Stream {} error: {}. Reconnecting in {}s...",
                    stream_name,
                    e,
                    reconnect_delay
                );
                tokio::time::sleep(Duration::from_secs(reconnect_delay)).await;
            }
        }
    }

    async fn watch_websocket(&self, config: &StreamConfig) -> Result<()> {
        use tokio_tungstenite::connect_async;
        use tokio_tungstenite::tungstenite::Message;
        use futures_util::StreamExt;

        tracing::debug!("Connecting to WebSocket: {}", config.url);

        let (ws_stream, _) = connect_async(&config.url)
            .await
            .context("Failed to connect to WebSocket")?;

        tracing::info!("✅ Connected to WebSocket: {}", config.url);

        let (_, mut read) = ws_stream.split();

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    for line in text.lines() {
                        self.process_line(line, &config.get_name()).await;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data) {
                        for line in text.lines() {
                            self.process_line(line, &config.get_name()).await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::warn!("WebSocket closed by server");
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Handled automatically by the library
                }
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
            }
        }

        Err(anyhow::anyhow!("WebSocket stream ended"))
    }

    async fn watch_http(&self, config: &StreamConfig) -> Result<()> {
        use reqwest::Client;

        tracing::debug!("Connecting to HTTP stream: {}", config.url);

        let client = Client::new();
        let mut request = client.get(&config.url);

        // Add custom headers if provided
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        let response = request
            .send()
            .await
            .context("Failed to connect to HTTP stream")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP stream returned error: {}",
                response.status()
            ));
        }

        tracing::info!("✅ Connected to HTTP stream: {}", config.url);

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk) = {
            use futures_util::StreamExt;
            stream.next().await
        } {
            let chunk = chunk.context("Failed to read HTTP stream chunk")?;
            buffer.extend_from_slice(&chunk);

            // Process complete lines
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                if let Ok(line) = String::from_utf8(line_bytes) {
                    let line = line.trim();
                    if !line.is_empty() {
                        self.process_line(line, &config.get_name()).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!("HTTP stream ended"))
    }

    async fn watch_tcp(&self, config: &StreamConfig) -> Result<()> {
        use tokio::net::TcpStream;

        tracing::debug!("Connecting to TCP stream: {}", config.url);

        // Parse host:port from URL (e.g., "tcp://localhost:9000" or "localhost:9000")
        let addr = config
            .url
            .strip_prefix("tcp://")
            .unwrap_or(&config.url)
            .to_string();

        let stream = TcpStream::connect(&addr)
            .await
            .context("Failed to connect to TCP stream")?;

        tracing::info!("✅ Connected to TCP stream: {}", addr);

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            self.process_line(&line, &config.get_name()).await;
        }

        Err(anyhow::anyhow!("TCP stream ended"))
    }

    async fn process_line(&self, line: &str, source: &str) {
        for rule in &self.rules {
            if rule.regex.is_match(line) {
                tracing::info!(
                    "🔔 Rule '{}' matched in stream '{}': {}",
                    rule.name,
                    source,
                    line
                );

                let message = format!(
                    "Rule '{}' triggered\nStream: {}\nLine: {}",
                    rule.name, source, line
                );

                // Send alert to all configured handlers
                if let Err(e) = self
                    .alert_manager
                    .send_alert_multi(&rule.alert_names, &rule.name, &message, rule.cooldown)
                    .await
                {
                    tracing::error!("Failed to send alert for rule '{}': {}", rule.name, e);
                }
            }
        }
    }
}
