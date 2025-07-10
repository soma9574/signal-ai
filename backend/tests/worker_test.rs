use backend::worker::start_signal_worker;
use backend::{AppState, llm::LlmClient, signal::{SignalClient, SignalMessage}};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

struct MockSignalWithMessages {
    messages: Arc<Mutex<Vec<SignalMessage>>>,
    sent_messages: Arc<Mutex<Vec<String>>>,
}

impl MockSignalWithMessages {
    fn new(messages: Vec<SignalMessage>) -> Self {
        Self {
            messages: Arc::new(Mutex::new(messages)),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_sent_messages(&self) -> Vec<String> {
        self.sent_messages.lock().unwrap().clone()
    }
}

#[async_trait]
impl SignalClient for MockSignalWithMessages {
    async fn send_message(&self, _to: &str, content: &str) -> anyhow::Result<()> {
        self.sent_messages.lock().unwrap().push(content.to_string());
        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>> {
        let mut messages = self.messages.lock().unwrap();
        let result = messages.drain(..).collect();
        Ok(result)
    }
}

struct MockLlm;

#[async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, _prompt: &str) -> anyhow::Result<String> {
        Ok("Mock Senator Budd response".to_string())
    }
}

#[tokio::test]
async fn worker_processes_signal_messages() {
    // Use in-memory SQLite for testing  
    let database_url = "sqlite::memory:";

    let pool = backend::db::init_pool(&database_url)
        .await
        .expect("Failed to init pool");

    let test_message = SignalMessage {
        from: "+1234567890".to_string(),
        to: "+0987654321".to_string(),
        content: "What are your thoughts on military preparedness?".to_string(),
    };

    let signal_client = Arc::new(MockSignalWithMessages::new(vec![test_message]));
    let signal_clone = signal_client.clone();

    let state = AppState {
        pool,
        llm: Arc::new(MockLlm),
        signal: signal_clone,
    };

    // Run worker for a short time
    let worker_task = tokio::spawn(async move {
        start_signal_worker(state).await;
    });

    // Give worker time to process
    tokio::time::sleep(Duration::from_millis(100)).await;
    worker_task.abort();

    // Check that a response was sent
    let sent = signal_client.get_sent_messages();
    assert!(!sent.is_empty(), "Worker should have sent at least one response");
    assert_eq!(sent[0], "Mock Senator Budd response");
} 