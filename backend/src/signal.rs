use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn, error, debug};

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
        info!("🔄 Attempting to send Signal message via signal-cli");
        debug!("Signal CLI send - To: {}, Content length: {} chars", to, content.len());
        debug!("Using phone number: {}", self.phone_number);

        // Check if signal-cli is available
        let version_check = Command::new("signal-cli")
            .arg("--version")
            .output()
            .await;
        
        match version_check {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                debug!("✅ signal-cli found, version: {}", version.trim());
            }
            Ok(output) => {
                error!("❌ signal-cli command failed with status: {}", output.status);
                error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                anyhow::bail!("signal-cli version check failed");
            }
            Err(e) => {
                error!("❌ signal-cli not found or not executable: {}", e);
                anyhow::bail!("signal-cli not available: {}", e);
            }
        }

        let output = Command::new("signal-cli")
            .arg("-a")
            .arg(&self.phone_number)
            .arg("send")
            .arg(to)
            .arg("-m")
            .arg(content)
            .arg("--verbose")  // Add verbose flag for better debugging
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("❌ signal-cli send failed with status: {}", output.status);
            error!("📤 Command: signal-cli -a {} send {} -m [message]", self.phone_number, to);
            error!("📋 stderr: {}", stderr);
            error!("📋 stdout: {}", stdout);
            anyhow::bail!("signal-cli send failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        info!("✅ Signal message sent successfully to {}", to);
        if !stdout.trim().is_empty() {
            debug!("📤 signal-cli output: {}", stdout);
        }
        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>> {
        debug!("🔄 Polling for Signal messages via signal-cli");
        debug!("Using phone number: {}", self.phone_number);

        let output = Command::new("signal-cli")
            .arg("-a")
            .arg(&self.phone_number)
            .arg("receive")
            .arg("--json")
            .arg("--timeout")
            .arg("5")  // 5 second timeout to avoid hanging
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("⚠️  signal-cli receive failed: {}", stderr);
            debug!("📤 Command: signal-cli -a {} receive --json --timeout 5", self.phone_number);
            debug!("📋 Exit status: {}", output.status);
            return Ok(vec![]);
        }

        let response_text = String::from_utf8(output.stdout)?;
        if response_text.trim().is_empty() {
            debug!("📭 No new Signal messages");
            return Ok(vec![]);
        }

        debug!("📬 Raw signal-cli receive output: {}", response_text);

        // Parse signal-cli JSON output (simplified - real parsing would be more robust)
        let lines: Vec<&str> = response_text.lines().collect();
        let mut messages = Vec::new();

        for line in lines {
            debug!("🔍 Parsing line: {}", line);
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
                if let (Some(envelope), Some(source)) = (msg.get("envelope"), msg.get("envelope").and_then(|e| e.get("source"))) {
                    if let Some(data_message) = envelope.get("dataMessage") {
                        if let Some(message_text) = data_message.get("message") {
                            let from = source.as_str().unwrap_or("unknown").to_string();
                            let content = message_text.as_str().unwrap_or("").to_string();
                            info!("📨 Received Signal message from {}: {}", from, content);
                            messages.push(SignalMessage {
                                from,
                                to: self.phone_number.clone(),
                                content,
                            });
                        }
                    }
                }
            } else {
                debug!("⚠️  Could not parse JSON line: {}", line);
            }
        }

        if messages.is_empty() {
            debug!("📭 No parseable messages found");
        } else {
            info!("📬 Successfully parsed {} Signal messages", messages.len());
        }
        Ok(messages)
    }
} 