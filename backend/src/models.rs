use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,    // Changed from Uuid to String to match database TEXT type
    pub role: String,  // "user" | "assistant"
    pub content: String,
    pub created_at: DateTime<Utc>,
} 