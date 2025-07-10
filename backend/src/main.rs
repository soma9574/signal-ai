use axum::{extract::State, routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::sync::Arc;

pub mod db;
pub mod llm;
// mod models; // Removed as per edit hint
use llm::{AnthropicClient, LlmClient};
// use models::Message; // Removed as per edit hint

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub llm: Arc<dyn LlmClient>,
}

async fn chat_handler(State(state): State<AppState>, Json(payload): Json<ChatRequest>) -> Json<ChatResponse> {
    use uuid::Uuid;
    // chrono not currently used directly here
    // use chrono::Utc;

    let mut tx = state.pool.begin().await.unwrap();

    let user_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(user_msg_id)
        .bind("user")
        .bind(&payload.message)
        .execute(&mut *tx)
        .await
        .unwrap();

    let prompt = format!("Respond as Senator Ted Budd to: {}", payload.message);
    let completion = state.llm.complete(&prompt).await.unwrap_or_else(|_| "Sorry, unable to respond now".into());

    let assistant_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(assistant_msg_id)
        .bind("assistant")
        .bind(&completion)
        .execute(&mut *tx)
        .await
        .unwrap();

    tx.commit().await.unwrap();

    Json(ChatResponse { reply: completion })
}

pub fn build_app(state: AppState) -> Router {
    Router::new().route("/chat", post(chat_handler)).with_state(state)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&database_url).await.expect("Failed to connect DB");
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
    let llm_client = Arc::new(AnthropicClient::new(api_key));

    let state = AppState { pool, llm: llm_client };
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
} 