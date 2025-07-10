use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};
use crate::AppState;

pub async fn start_signal_worker(state: AppState) {
    info!("🔄 Signal worker started - polling every 10 seconds");
    
    loop {
        match process_signal_messages(&state).await {
            Ok(processed) => {
                if processed > 0 {
                    info!("✅ Processed {} Signal messages", processed);
                } else {
                    debug!("📭 No new messages to process");
                }
            }
            Err(e) => {
                error!("❌ Error processing Signal messages: {}", e);
                error!("🔄 Will retry in 10 seconds...");
            }
        }
        
        // Poll every 10 seconds
        sleep(Duration::from_secs(10)).await;
    }
}

async fn process_signal_messages(state: &AppState) -> anyhow::Result<usize> {
    debug!("🔍 Checking for new Signal messages...");
    let messages = state.signal.receive_messages().await?;
    let mut processed = 0;
    
    for message in messages {
        info!("📨 Processing Signal message from {}: {}", message.from, message.content);
        
        // Create Senator Budd persona prompt
        let senator_prompt = format!(
            "You are Senator Ted Budd of North Carolina. Respond to this message as the Senator would, \
            keeping in mind you're helping prepare Vice Admiral Mitch Bradley for his confirmation hearing \
            for Admiral and Commander of SOCOM. Be professional, knowledgeable about military affairs, \
            and supportive.\n\nMessage: {}",
            message.content
        );
        
        debug!("🤖 Generating LLM response...");
        match state.llm.complete(&senator_prompt).await {
            Ok(response) => {
                info!("✅ Generated response: {}", response);
                
                // Store the conversation in DB
                debug!("💾 Storing conversation in database...");
                if let Err(e) = store_signal_conversation(state, &message, &response).await {
                    warn!("⚠️  Failed to store Signal conversation: {}", e);
                } else {
                    debug!("✅ Conversation stored successfully");
                }
                
                // Send response back via Signal
                debug!("📤 Sending response via Signal...");
                if let Err(e) = state.signal.send_message(&message.from, &response).await {
                    error!("❌ Failed to send Signal response to {}: {}", message.from, e);
                } else {
                    info!("✅ Sent Signal response to {}", message.from);
                    processed += 1;
                }
            }
            Err(e) => {
                error!("❌ Failed to generate LLM response: {}", e);
            }
        }
    }
    
    Ok(processed)
}

async fn store_signal_conversation(
    state: &AppState, 
    incoming: &crate::signal::SignalMessage, 
    response: &str
) -> anyhow::Result<()> {
    use uuid::Uuid;
    
    let mut tx = state.pool.begin().await?;
    
    // Store incoming message
    let user_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(user_msg_id.to_string())
        .bind("user")
        .bind(&incoming.content)
        .execute(&mut *tx)
        .await?;
    
    // Store response
    let assistant_msg_id = Uuid::new_v4();
    sqlx::query("INSERT INTO messages (id, role, content) VALUES ($1, $2, $3)")
        .bind(assistant_msg_id.to_string())
        .bind("assistant")
        .bind(response)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
} 