use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMessage {
    pub from: String,
    pub to: String,
    pub content: String,
}

#[async_trait]
pub trait SignalClient: Send + Sync + 'static {
    async fn send_message(&self, to: &str, content: &str) -> anyhow::Result<()>;
    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>>;
}

pub struct SignaldClient {
    socket_path: String,
    phone_number: String,
}

impl SignaldClient {
    pub fn new(socket_path: String, phone_number: String) -> Self {
        Self { socket_path, phone_number }
    }
}

#[derive(Serialize)]
struct SignaldRequest {
    #[serde(rename = "type")]
    request_type: String,
    account: String,
    recipient: String,
    message: String,
}

#[derive(Deserialize)]
struct SignaldResponse {
    #[serde(rename = "type")]
    response_type: String,
    data: Option<serde_json::Value>,
}

#[async_trait]
impl SignalClient for SignaldClient {
    async fn send_message(&self, to: &str, content: &str) -> anyhow::Result<()> {
        let request = SignaldRequest {
            request_type: "send".to_string(),
            account: self.phone_number.clone(),
            recipient: to.to_string(),
            message: content.to_string(),
        };

        let json_request = serde_json::to_string(&request)?;
        info!("Sending Signal message via signald: {}", json_request);

        // Use socat to send JSON to signald Unix socket
        let mut cmd = Command::new("socat")
            .arg("-")
            .arg(format!("UNIX-CONNECT:{}", self.socket_path))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = cmd.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(json_request.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
        }

        let output = cmd.wait_with_output().await?;
        if !output.status.success() {
            anyhow::bail!("signald command failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let response_text = String::from_utf8(output.stdout)?;
        info!("signald response: {}", response_text);

        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>> {
        // For now, return empty - in a real implementation we'd listen to the signald socket
        // or poll for new messages
        warn!("receive_messages not fully implemented yet");
        Ok(vec![])
    }
}

// Alternative implementation using signal-cli instead of signald
pub struct SignalCliClient {
    phone_number: String,
}

impl SignalCliClient {
    pub fn new(phone_number: String) -> Self {
        Self { phone_number }
    }
}

#[async_trait]
impl SignalClient for SignalCliClient {
    async fn send_message(&self, to: &str, content: &str) -> anyhow::Result<()> {
        info!("Sending Signal message via signal-cli to {}: {}", to, content);

        let output = Command::new("signal-cli")
            .arg("-a")
            .arg(&self.phone_number)
            .arg("send")
            .arg(to)
            .arg("-m")
            .arg(content)
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("signal-cli send failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        info!("signal-cli send successful");
        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>> {
        info!("Receiving messages via signal-cli");

        let output = Command::new("signal-cli")
            .arg("-a")
            .arg(&self.phone_number)
            .arg("receive")
            .arg("--json")
            .output()
            .await?;

        if !output.status.success() {
            warn!("signal-cli receive failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(vec![]);
        }

        let response_text = String::from_utf8(output.stdout)?;
        if response_text.trim().is_empty() {
            return Ok(vec![]);
        }

        // Parse signal-cli JSON output (simplified - real parsing would be more robust)
        let lines: Vec<&str> = response_text.lines().collect();
        let mut messages = Vec::new();

        for line in lines {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                if let (Some(envelope), Some(source)) = (msg.get("envelope"), msg.get("envelope").and_then(|e| e.get("source"))) {
                    if let Some(data_message) = envelope.get("dataMessage") {
                        if let Some(message_text) = data_message.get("message") {
                            messages.push(SignalMessage {
                                from: source.as_str().unwrap_or("unknown").to_string(),
                                to: self.phone_number.clone(),
                                content: message_text.as_str().unwrap_or("").to_string(),
                            });
                        }
                    }
                }
            }
        }

        info!("Received {} messages", messages.len());
        Ok(messages)
    }
} 