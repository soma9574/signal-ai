//! # Signal AI Backend
//! 
//! A high-quality Rust backend service that provides chat functionality using LLM services 
//! and Signal messaging integration. This service acts as Senator Ted Budd to help Vice Admiral 
//! Mitch Bradley prepare for confirmation hearings.
//!
//! ## Features
//! 
//! - **Chat API**: HTTP endpoints for conversing with the Senator Budd AI persona
//! - **Signal Integration**: Bidirectional messaging via Signal CLI
//! - **Database Persistence**: SQLite storage for all conversations
//! - **Background Worker**: Continuous polling for incoming Signal messages
//! - **Health Monitoring**: Health check endpoints for operational visibility
//!
//! ## Usage
//! 
//! ```rust,no_run
//! use backend::{AppState, build_app};
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let state = AppState {
//!     pool: backend::db::init_pool("postgresql://user:pass@localhost/db").await?,
//!     llm: std::sync::Arc::new(backend::llm::AnthropicClient::new("api-key".to_string())),
//!     signal: std::sync::Arc::new(backend::signal::SignalCliClient::new("+1234567890".to_string())),
//! };
//! 
//! let app = build_app(state);
//! # Ok(())
//! # }
//! ```

pub mod db;
pub mod error;
pub mod llm;
pub mod signal;
pub mod worker;

use axum::routing::get;
use axum::{extract::State, routing::post, Json, Router};
use error::{AppError, AppResult};
use llm::LlmClient;
use serde::{Deserialize, Serialize};
use signal::SignalClient;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Application state shared across all handlers
/// 
/// Contains all the dependencies needed by the application handlers,
/// including database connection pool and service clients.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool for PostgreSQL
pub pool: PgPool,
    /// LLM service client for generating responses
    pub llm: Arc<dyn LlmClient>,
    /// Signal messaging client for sending/receiving messages
    pub signal: Arc<dyn SignalClient>,
}

/// Request payload for chat endpoint
/// 
/// Contains the user message to be processed by the Senator Budd AI persona.
#[derive(Deserialize)]
pub struct ChatRequest {
    /// The message from the user (1-4000 characters)
    pub message: String,
}

/// Response payload for chat endpoint
/// 
/// Contains the AI-generated response from Senator Budd.
#[derive(Serialize)]
pub struct ChatResponse {
    /// The generated response from Senator Budd
    pub reply: String,
}

/// Request payload for sending Signal messages
/// 
/// Contains the recipient and message content for Signal messaging.
#[derive(Deserialize)]
pub struct SendSignalRequest {
    /// Phone number to send message to (must start with '+' and country code)
    pub to: String,
    /// Message content to send (1-1000 characters)
    pub message: String,
}

/// Health check response payload
/// 
/// Provides system status information for monitoring and diagnostics.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Overall system status: "healthy" or "degraded"
    pub status: String,
    /// Whether signal-cli is available and working
    pub signal_cli_available: bool,
    /// Whether database connection is working
    pub database_connected: bool,
    /// Configured Signal phone number
    pub phone_number: String,
}

/// Response payload for Signal message sending
/// 
/// Indicates whether the Signal message was sent successfully.
#[derive(Serialize)]
pub struct SendSignalResponse {
    /// Whether the message was sent successfully
    pub success: bool,
    /// Error message if sending failed
    pub error: Option<String>,
}

/// Handle chat requests from users
/// 
/// This endpoint processes chat messages by:
/// 1. Validating the input message (length, content)
/// 2. Storing the user message in the database
/// 3. Generating a response using the LLM service as Senator Ted Budd
/// 4. Storing the assistant response in the database
/// 5. Returning the response to the user
/// 
/// # Arguments
/// 
/// * `state` - Application state containing database and service clients
/// * `payload` - Chat request containing the user's message
/// 
/// # Returns
/// 
/// * `AppResult<Json<ChatResponse>>` - The AI-generated response or an error
/// 
/// # Errors
/// 
/// * `AppError::Validation` - If the message is empty or too long
/// * `AppError::Database` - If database operations fail
/// * Database transaction is automatically rolled back on error
/// 
/// # Example
/// 
/// ```json
/// POST /chat
/// {
///   "message": "What are your thoughts on military readiness?"
/// }
/// 
/// Response:
/// {
///   "reply": "As Senator Ted Budd, I believe maintaining strong military readiness..."
/// }
/// ```
pub async fn chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> AppResult<Json<ChatResponse>> {
    // Validate input
    if payload.message.trim().is_empty() {
        return Err(AppError::validation("message", "Message cannot be empty"));
    }
    if payload.message.len() > 4000 {
        return Err(AppError::validation("message", "Message too long (max 4000 characters)"));
    }

    let mut tx = state.pool.begin().await?;

    let user_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(user_msg_id.to_string())
        .bind("user")
        .bind(&payload.message)
        .execute(&mut *tx)
        .await?;

    let prompt = format!("Respond as Senator Ted Budd to: {}", payload.message);
    let completion = state
        .llm
        .complete(&prompt)
        .await
        .unwrap_or_else(|_| "Sorry, unable to respond now".into());

    let assistant_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(assistant_msg_id.to_string())
        .bind("assistant")
        .bind(&completion)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(ChatResponse { reply: completion }))
}

/// Send a Signal message to a specified phone number
/// 
/// This endpoint allows manual sending of Signal messages through the API.
/// It validates the input and attempts to send the message via Signal CLI.
/// 
/// # Arguments
/// 
/// * `state` - Application state containing the Signal client
/// * `payload` - Send request containing recipient and message
/// 
/// # Returns
/// 
/// * `AppResult<Json<SendSignalResponse>>` - Success/failure status
/// 
/// # Errors
/// 
/// * `AppError::Validation` - If phone number format is invalid or message is too long
/// * Returns success=false in response if Signal sending fails (doesn't return error)
/// 
/// # Example
/// 
/// ```json
/// POST /signal/send
/// {
///   "to": "+1234567890",
///   "message": "Test message from Senator Budd"
/// }
/// 
/// Response:
/// {
///   "success": true,
///   "error": null
/// }
/// ```
pub async fn send_signal_message(
    State(state): State<AppState>,
    Json(payload): Json<SendSignalRequest>,
) -> AppResult<Json<SendSignalResponse>> {
    // Validate input
    if payload.to.trim().is_empty() {
        return Err(AppError::validation("to", "Phone number cannot be empty"));
    }
    if !payload.to.starts_with('+') {
        return Err(AppError::validation("to", "Phone number must start with '+' and include country code"));
    }
    if payload.message.trim().is_empty() {
        return Err(AppError::validation("message", "Message cannot be empty"));
    }
    if payload.message.len() > 1000 {
        return Err(AppError::validation("message", "Message too long (max 1000 characters)"));
    }

    match state
        .signal
        .send_message(&payload.to, &payload.message)
        .await
    {
        Ok(_) => Ok(Json(SendSignalResponse {
            success: true,
            error: None,
        })),
        Err(e) => Ok(Json(SendSignalResponse {
            success: false,
            error: Some(e.to_string()),
        })),
    }
}

/// Health check endpoint for system monitoring
/// 
/// This endpoint provides system health information including:
/// - Overall system status (healthy/degraded)
/// - Signal CLI availability
/// - Database connectivity
/// - Configured phone number
/// 
/// This is typically used by load balancers, monitoring systems, and ops teams
/// to verify the service is functioning correctly.
/// 
/// # Arguments
/// 
/// * `state` - Application state for testing connections
/// 
/// # Returns
/// 
/// * `AppResult<Json<HealthResponse>>` - System health status
/// 
/// # Example
/// 
/// ```json
/// GET /health
/// 
/// Response:
/// {
///   "status": "healthy",
///   "signal_cli_available": true,
///   "database_connected": true,
///   "phone_number": "+1234567890"
/// }
/// ```
pub async fn health_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
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
    let db_connected = sqlx::query("SELECT 1")
        .fetch_optional(&state.pool)
        .await
        .is_ok();

    // Get phone number from environment (safer than unwrap)
    let phone_number = std::env::var("SIGNAL_PHONE_NUMBER").unwrap_or_else(|_| "not configured".to_string());

    let overall_status = if signal_available && db_connected {
        "healthy"
    } else {
        "degraded"
    };

    Ok(Json(HealthResponse {
        status: overall_status.to_string(),
        signal_cli_available: signal_available,
        database_connected: db_connected,
        phone_number,
    }))
}

/// Build the main application router
/// 
/// Creates and configures the Axum router with all API routes and shared state.
/// This function sets up:
/// - `/health` - Health check endpoint (GET)
/// - `/chat` - Chat with Senator Budd (POST)
/// - `/signal/send` - Send Signal messages (POST)
/// 
/// # Arguments
/// 
/// * `state` - Application state to share across all handlers
/// 
/// # Returns
/// 
/// * `Router` - Configured Axum router ready to serve requests
/// 
/// # Example
/// 
/// ```rust,no_run
/// use backend::{AppState, build_app};
/// use std::sync::Arc;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pool = backend::db::init_pool("postgresql://user:pass@localhost/db").await?;
/// let llm = Arc::new(backend::llm::AnthropicClient::new("api-key".to_string()));
/// let signal = Arc::new(backend::signal::SignalCliClient::new("+1234567890".to_string()));
/// 
/// let state = AppState { pool, llm, signal };
/// let app = build_app(state);
/// 
/// // The app can now be served with axum::serve()
/// # Ok(())
/// # }
/// ```
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/chat", post(chat_handler))
        .route("/signal/send", post(send_signal_message))
        .route("/health", get(health_check))
        .with_state(state)
}
