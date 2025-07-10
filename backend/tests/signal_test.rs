use backend::signal::{SignalClient, SignalMessage};
use async_trait::async_trait;

struct MockSignalClient {
    should_fail: bool,
}

impl MockSignalClient {
    fn new(should_fail: bool) -> Self {
        Self { should_fail }
    }
}

#[async_trait]
impl SignalClient for MockSignalClient {
    async fn send_message(&self, _to: &str, _content: &str) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("Mock signal send failure");
        }
        Ok(())
    }

    async fn receive_messages(&self) -> anyhow::Result<Vec<SignalMessage>> {
        Ok(vec![SignalMessage {
            from: "+1234567890".to_string(),
            to: "+0987654321".to_string(),
            content: "Test message".to_string(),
        }])
    }
}

#[tokio::test]
async fn signal_client_send_success() {
    let client = MockSignalClient::new(false);
    let result = client.send_message("+1234567890", "Hello").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn signal_client_send_failure() {
    let client = MockSignalClient::new(true);
    let result = client.send_message("+1234567890", "Hello").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn signal_client_receive_messages() {
    let client = MockSignalClient::new(false);
    let messages = client.receive_messages().await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test message");
} 