pub mod db;
pub mod llm;
pub mod signal;
pub mod worker;

use llm::{AnthropicClient, LlmClient};
use signal::{SignalClient, SignalCliClient};
use axum::{extract::State, routing::post, Router, Json};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: Arc<dyn LlmClient>,
    pub signal: Arc<dyn SignalClient>,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Deserialize)]
pub struct SendSignalRequest {
    pub to: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub signal_cli_available: bool,
    pub database_connected: bool,
    pub phone_number: String,
}

#[derive(Serialize)]
pub struct SendSignalResponse {
    pub success: bool,
    pub error: Option<String>,
}

pub async fn chat_handler(State(state): State<AppState>, Json(payload): Json<ChatRequest>) -> Json<ChatResponse> {
    let mut tx = state.pool.begin().await.unwrap();

    let user_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(user_msg_id.to_string())
        .bind("user")
        .bind(&payload.message)
        .execute(&mut *tx)
        .await
        .unwrap();

    let prompt = format!("Respond as Senator Ted Budd to: {}", payload.message);
    let completion = state.llm.complete(&prompt).await.unwrap_or_else(|_| "Sorry, unable to respond now".into());

    let assistant_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(assistant_msg_id.to_string())
        .bind("assistant")
        .bind(&completion)
        .execute(&mut *tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();

    Json(ChatResponse { reply: completion })
}

pub async fn send_signal_message(State(state): State<AppState>, Json(payload): Json<SendSignalRequest>) -> Json<SendSignalResponse> {
    match state.signal.send_message(&payload.to, &payload.message).await {
        Ok(_) => Json(SendSignalResponse { success: true, error: None }),
        Err(e) => Json(SendSignalResponse { success: false, error: Some(e.to_string()) }),
    }
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // Test signal-cli availability
    let signal_available = match tokio::process::Command::new("signal-cli")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    };
    
    // Test database connection
    let db_connected = sqlx::query("SELECT 1").fetch_optional(&state.pool).await.is_ok();
    
    // Get phone number from signal client (we'll need to add a getter)
    let phone_number = std::env::var("SIGNAL_PHONE_NUMBER").unwrap_or("not configured".to_string());
    
    let overall_status = if signal_available && db_connected {
        "healthy"
    } else {
        "degraded"
    };
    
    Json(HealthResponse {
        status: overall_status.to_string(),
        signal_cli_available: signal_available,
        database_connected: db_connected,
        phone_number,
    })
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/signal/send", post(send_signal_message))
        .route("/health", get(health_check))
        .with_state(state)
} 