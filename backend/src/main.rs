use backend::error::AppResult;
use backend::signal::SignalClient;
use backend::{
    build_app, db, llm::AnthropicClient, signal::SignalCliClient, worker::start_signal_worker,
    AppState,
};
use dotenvy::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // Startup diagnostics
    info!("🚀 Starting Senator Budd Signal Chatbot");
    info!("📋 Environment check:");

    // Check required environment variables
    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) => {
            info!("✅ ANTHROPIC_API_KEY found (length: {})", key.len());
            key
        }
        Err(_) => {
            error!("❌ ANTHROPIC_API_KEY not set - required for LLM functionality");
            std::process::exit(1);
        }
    };

    let signal_phone = match std::env::var("SIGNAL_PHONE_NUMBER") {
        Ok(phone) => {
            info!("✅ SIGNAL_PHONE_NUMBER found: {}", phone);
            phone
        }
        Err(_) => {
            error!("❌ SIGNAL_PHONE_NUMBER not set - required for Signal integration");
            std::process::exit(1);
        }
    };

    let mut database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        // Use /tmp for Railway/container deployment - always writable
        if std::env::var("RAILWAY_ENVIRONMENT").is_ok() || std::env::var("RAILWAY_PROJECT_ID").is_ok() {
            "sqlite:/tmp/chat_history.db".to_string()
        } else {
            "sqlite:chat_history.db".to_string()  // Local development
        }
    });

    // For container deployment, ensure database directory exists or fallback to /tmp
    let db_path = database_url.replace("sqlite:", "");
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if !parent.exists() {
            info!("📁 Creating database directory: {:?}", parent);
            if let Err(e) = std::fs::create_dir_all(parent) {
                // If we can't create the directory (container permissions), fallback to /tmp
                info!("⚠️  Failed to create directory: {} - falling back to /tmp", e);
                database_url = "sqlite:/tmp/chat_history.db".to_string();
                info!("📁 Using fallback database path: {}", database_url);
            }
        }
    }
    
    info!("📁 Database: {}", database_url);

    let pool = db::init_pool(&database_url).await.map_err(|e| {
        error!("Failed to connect to database: {}", e);
        backend::error::AppError::config(format!("Database connection failed: {e}"))
    })?;
    info!("✅ Database connected successfully");

    let llm_client = Arc::new(AnthropicClient::new(api_key));
    info!("✅ LLM client initialized");

    let signal_client = Arc::new(SignalCliClient::new(signal_phone.clone()));
    info!("✅ Signal client initialized");

    // Test Signal CLI availability at startup
    info!("🔍 Testing Signal CLI availability...");
    match signal_client
        .send_message(&signal_phone, "Startup test - ignore")
        .await
    {
        Ok(_) => info!("✅ Signal CLI test successful"),
        Err(e) => {
            error!("❌ Signal CLI test failed: {}", e);
            error!("💡 Make sure signal-cli is installed and registered");
            error!("💡 Try: brew install signal-cli (macOS) or apt install signal-cli (Linux)");
            error!("💡 Register with: signal-cli -a {} register", &signal_phone);
        }
    }

    let state = AppState {
        pool,
        llm: llm_client,
        signal: signal_client,
    };

    // Start background Signal worker
    info!("🔄 Starting background Signal worker...");
    let worker_state = state.clone();
    tokio::spawn(async move {
        start_signal_worker(worker_state).await;
    });

    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🌐 Server listening on {}", addr);
    info!("📱 Ready to receive Signal messages!");

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        backend::error::AppError::internal(format!("Failed to bind to address {addr}: {e}"))
    })?;

    axum::serve(listener, app).await.map_err(|e| {
        backend::error::AppError::internal(format!("Server error: {e}"))
    })?;

    Ok(())
}
