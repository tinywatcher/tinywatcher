use crate::alerts::AlertManager;
use crate::config::{Rule, SourceType, StreamConfig, StreamType};
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
    sources: Option<crate::config::RuleSources>,
}

impl StreamMonitor {
    pub fn new(rules: Vec<Rule>, alert_manager: Arc<AlertManager>) -> Result<Self> {
        let compiled_rules = rules
            .into_iter()
            .map(|rule| {
                let regex = Regex::new(&rule.pattern)
                    .with_context(|| format!("Invalid regex pattern in rule: {}", rule.name))?;
                Ok(CompiledRule {
                    name: rule.name.clone(),
                    regex,
                    alert_names: rule.alert,
                    cooldown: rule.cooldown,
                    sources: rule.sources,
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
        tracing::info!(" Starting stream monitoring: {}", stream_name);

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

        tracing::info!(" Connected to WebSocket: {}", config.url);

        let (_, mut read) = ws_stream.split();

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    let source = SourceType::Stream(config.get_name());
                    for line in text.lines() {
                        self.process_line(line, &source).await;
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data) {
                        let source = SourceType::Stream(config.get_name());
                        for line in text.lines() {
                            self.process_line(line, &source).await;
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
                        let source = SourceType::Stream(config.get_name());
                        self.process_line(line, &source).await;
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

        let source = SourceType::Stream(config.get_name());
        while let Some(line) = lines.next_line().await? {
            self.process_line(&line, &source).await;
        }

        Err(anyhow::anyhow!("TCP stream ended"))
    }

    async fn process_line(&self, line: &str, source: &SourceType) {
        for rule in &self.rules {
            // Check if rule applies to this source
            if !self.rule_applies_to_source(rule, source) {
                continue;
            }

            if rule.regex.is_match(line) {
                let source_name = match source {
                    SourceType::Stream(name) => name.clone(),
                    _ => format!("{:?}", source),
                };

                tracing::info!(
                    "🔔 Rule '{}' matched in stream '{}': {}",
                    rule.name,
                    source_name,
                    line
                );

                let message = format!(
                    "Rule '{}' triggered\nStream: {}\nLine: {}",
                    rule.name, source_name, line
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

    fn rule_applies_to_source(&self, rule: &CompiledRule, source: &SourceType) -> bool {
        // If no sources filter is specified, rule applies to all sources
        let Some(ref sources) = rule.sources else {
            return true;
        };

        match source {
            SourceType::File(path) => {
                if sources.files.is_empty() {
                    return false;
                }
                sources.files.iter().any(|f| f == path)
            }
            SourceType::Container(name) => {
                if sources.containers.is_empty() {
                    return false;
                }
                sources.containers.iter().any(|c| c == name)
            }
            SourceType::Stream(name) => {
                if sources.streams.is_empty() {
                    return false;
                }
                sources.streams.iter().any(|s| s == name)
            }
        }
    }
}
