//! # ChatDelta AI Client Library
//! 
//! A Rust library for connecting to multiple AI APIs (OpenAI, Google Gemini, Anthropic Claude)
//! with a unified interface. Supports parallel execution, retry logic, and configurable parameters.
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use chatdelta::{AiClient, ClientConfig, create_client};
//! use std::time::Duration;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig {
//!         timeout: Duration::from_secs(30),
//!         retries: 3,
//!         temperature: Some(0.7),
//!         max_tokens: Some(1024),
//!     };
//!     
//!     let client = create_client("openai", "your-api-key", "gpt-4o", config)?;
//!     let response = client.send_prompt("Hello, world!").await?;
//!     println!("{}", response);
//!     
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

pub mod clients;
pub mod error;

pub use clients::*;
pub use error::*;

/// Configuration for AI clients
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Timeout for HTTP requests
    pub timeout: Duration,
    /// Number of retry attempts for failed requests
    pub retries: u32,
    /// Temperature for AI responses (0.0-2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens for responses (Claude only)
    pub max_tokens: Option<u32>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retries: 0,
            temperature: None,
            max_tokens: Some(1024),
        }
    }
}

/// Common trait implemented by all AI clients
#[async_trait]
pub trait AiClient: Send + Sync {
    /// Sends a prompt and returns the textual response
    async fn send_prompt(&self, prompt: &str) -> Result<String, ClientError>;
    
    /// Returns the name/identifier of this AI client
    fn name(&self) -> &str;
    
    /// Returns the model being used by this client
    fn model(&self) -> &str;
}

/// Factory function to create AI clients
pub fn create_client(
    provider: &str,
    api_key: &str,
    model: &str,
    config: ClientConfig,
) -> Result<Box<dyn AiClient>, ClientError> {
    let http_client = Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| ClientError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

    match provider.to_lowercase().as_str() {
        "openai" | "gpt" | "chatgpt" => Ok(Box::new(ChatGpt::new(
            http_client,
            api_key.to_string(),
            model.to_string(),
            config,
        ))),
        "google" | "gemini" => Ok(Box::new(Gemini::new(
            http_client,
            api_key.to_string(),
            model.to_string(),
            config,
        ))),
        "anthropic" | "claude" => Ok(Box::new(Claude::new(
            http_client,
            api_key.to_string(),
            model.to_string(),
            config,
        ))),
        _ => Err(ClientError::Configuration(format!(
            "Unknown provider: {}. Supported providers: openai, google, anthropic",
            provider
        ))),
    }
}

/// Execute multiple AI clients in parallel and return all results
pub async fn execute_parallel(
    clients: Vec<Box<dyn AiClient>>,
    prompt: &str,
) -> Vec<(String, Result<String, ClientError>)> {
    use futures::future;
    
    let futures: Vec<_> = clients.iter().map(|client| {
        let name = client.name().to_string();
        let prompt = prompt.to_string();
        async move {
            let result = client.send_prompt(&prompt).await;
            (name, result)
        }
    }).collect();
    
    future::join_all(futures).await
}

/// Generate a summary using one of the provided clients
pub async fn generate_summary(
    client: &dyn AiClient,
    responses: &[(String, String)],
) -> Result<String, ClientError> {
    let mut summary_prompt = "Given these AI model responses:\n".to_string();
    for (name, response) in responses {
        summary_prompt.push_str(&format!("{}:\n{}\n---\n", name, response));
    }
    summary_prompt.push_str("Summarize the key differences and commonalities.");
    
    client.send_prompt(&summary_prompt).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    // Mock client for testing
    pub struct MockClient {
        pub name: String,
        pub model: String,
        pub responses: Arc<Mutex<VecDeque<Result<String, ClientError>>>>,
    }

    impl MockClient {
        pub fn new(name: &str, responses: Vec<Result<String, ClientError>>) -> Self {
            Self {
                name: name.to_string(),
                model: "mock-model".to_string(),
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            }
        }
    }

    #[async_trait]
    impl AiClient for MockClient {
        async fn send_prompt(&self, _prompt: &str) -> Result<String, ClientError> {
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Ok("mock response".to_string()))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn model(&self) -> &str {
            &self.model
        }
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retries, 0);
        assert_eq!(config.temperature, None);
        assert_eq!(config.max_tokens, Some(1024));
    }

    #[tokio::test]
    async fn test_execute_parallel() {
        let clients: Vec<Box<dyn AiClient>> = vec![
            Box::new(MockClient::new("client1", vec![Ok("response1".to_string())])),
            Box::new(MockClient::new("client2", vec![Ok("response2".to_string())])),
        ];

        let results = execute_parallel(clients, "test prompt").await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "client1");
        assert!(results[0].1.is_ok());
        assert_eq!(results[1].0, "client2");
        assert!(results[1].1.is_ok());
    }

    #[tokio::test]
    async fn test_generate_summary() {
        let client = MockClient::new("summarizer", vec![Ok("summary response".to_string())]);
        let responses = vec![
            ("AI1".to_string(), "response1".to_string()),
            ("AI2".to_string(), "response2".to_string()),
        ];

        let summary = generate_summary(&client, &responses).await;
        assert!(summary.is_ok());
        assert_eq!(summary.unwrap(), "summary response");
    }
}