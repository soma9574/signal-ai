use backend::{AppState, build_app, db, llm::AnthropicClient, signal::SignalCliClient, worker::start_signal_worker};
use std::net::SocketAddr;
use tracing::info;
use dotenvy::dotenv;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:chat_history.db".to_string());
    let pool = db::init_pool(&database_url).await.expect("Failed to connect DB");
    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
    let llm_client = Arc::new(AnthropicClient::new(api_key));

    let signal_phone = std::env::var("SIGNAL_PHONE_NUMBER").expect("SIGNAL_PHONE_NUMBER must be set");
    let signal_client = Arc::new(SignalCliClient::new(signal_phone));

    let state = AppState { 
        pool, 
        llm: llm_client, 
        signal: signal_client 
    };
    
    // Start background Signal worker
    let worker_state = state.clone();
    tokio::spawn(async move {
        start_signal_worker(worker_state).await;
    });

    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
} 