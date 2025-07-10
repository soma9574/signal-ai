use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait LlmClient: Send + Sync + 'static {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String>;
}

pub struct AnthropicClient {
    api_key: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'static str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

#[derive(Deserialize)]
struct Content {
    #[serde(rename = "type")]
    _content_type: String, // Prefixed with _ to indicate intentionally unused
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        let req_body = AnthropicRequest {
            model: "claude-sonnet-4-20250514", // Claude Sonnet 4 (latest)
            max_tokens: 1024,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
        };
        
        let resp = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req_body)
            .send()
            .await?;
            
        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Anthropic API error {}: {}", status, error_text);
        }
        
        let data: AnthropicResponse = resp.json().await?;
        
        // Extract the text from the first content item
        if let Some(content) = data.content.first() {
            Ok(content.text.clone())
        } else {
            anyhow::bail!("No content returned from Anthropic API")
        }
    }
}
