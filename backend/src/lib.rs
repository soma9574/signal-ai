pub mod db;
pub mod llm;
pub mod signal;
pub mod worker;

use llm::{AnthropicClient, LlmClient};
use signal::{SignalClient, SignalCliClient};
use axum::{extract::State, routing::post, Router, Json};
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

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/signal/send", post(send_signal_message))
        .with_state(state)
} 