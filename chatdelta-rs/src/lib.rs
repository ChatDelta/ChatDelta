use async_trait::async_trait;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct ClientConfig;

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig
    }
}

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn send_prompt(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
}

pub fn create_client(_provider: &str, _api_key: &str, _model: &str, _config: ClientConfig) -> Result<Box<dyn AiClient>, Box<dyn Error + Send + Sync>> {
    Ok(Box::new(MockClient))
}

struct MockClient;

#[async_trait]
impl AiClient for MockClient {
    async fn send_prompt(&self, _prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("Mock response".to_string())
    }
}
