use async_trait::async_trait;
use axum::body::Body;
use axum::http::StatusCode;
use axum::Router;
use backend::llm::LlmClient;
use backend::signal::SignalClient;
use backend::{build_app, AppState};
use hyper::Request;
use std::sync::Arc;
use tower::util::ServiceExt;

struct DummyLlm;

#[async_trait]
impl LlmClient for DummyLlm {
    async fn complete(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok("dummy reply".into())
    }
}

struct DummySignal;

#[async_trait]
impl SignalClient for DummySignal {
    async fn send_message(&self, _to: &str, _content: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<backend::signal::SignalMessage>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn chat_endpoint_returns_dummy_reply() {
    // Use test PostgreSQL database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/test_db".to_string());

    let pool = backend::db::init_pool(database_url)
        .await
        .expect("Failed to init pool");

    let state = AppState {
        pool,
        llm: Arc::new(DummyLlm),
        signal: Arc::new(DummySignal),
    };

    let app: Router = build_app(state);

    let payload = serde_json::json!({"message": "Hello"});
    let req = Request::builder()
        .method("POST")
        .uri("/chat")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["reply"], "dummy reply");
}
