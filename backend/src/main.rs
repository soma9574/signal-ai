use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::info;
use dotenvy::dotenv;

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    reply: String,
}

async fn chat_handler(Json(payload): Json<ChatRequest>) -> Json<ChatResponse> {
    // TODO: integrate with Anthropic Sonnet 4 model
    Json(ChatResponse {
        reply: format!("Echo: {}", payload.message),
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/chat", post(chat_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
} 