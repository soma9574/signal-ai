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
struct AnthropicRequest<'a> {
    model: &'static str,
    prompt: &'a str,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    completion: String,
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        let req_body = AnthropicRequest {
            model: "claude-3-sonnet-20240229", // Sonnet 4 (Claude 3 Sonnet) id
            prompt,
            max_tokens: 512,
        };
        let resp = reqwest::Client::new()
            .post("https://api.anthropic.com/v1/complete")
            .header("x-api-key", &self.api_key)
            .json(&req_body)
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!("Anthropic API error: {}", resp.status());
        }
        let data: AnthropicResponse = resp.json().await?;
        Ok(data.completion)
    }
}
