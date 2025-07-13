//! OpenAI ChatGPT client implementation

use crate::{AiClient, ClientConfig, ClientError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for OpenAI's ChatGPT models
pub struct ChatGpt {
    http: Client,
    key: String,
    model: String,
    temperature: Option<f32>,
    retries: u32,
}

impl ChatGpt {
    /// Create a new ChatGPT client
    pub fn new(http: Client, key: String, model: String, config: ClientConfig) -> Self {
        Self {
            http,
            key,
            model,
            temperature: config.temperature,
            retries: config.retries,
        }
    }
}

#[async_trait]
impl AiClient for ChatGpt {
    async fn send_prompt(&self, prompt: &str) -> Result<String, ClientError> {
        #[derive(Serialize)]
        struct Message<'a> {
            role: &'a str,
            content: &'a str,
        }
        
        #[derive(Serialize)]
        struct Request<'a> {
            model: &'a str,
            messages: Vec<Message<'a>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
        }
        
        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }
        
        #[derive(Deserialize)]
        struct Choice {
            message: RespMessage,
        }
        
        #[derive(Deserialize)]
        struct RespMessage {
            content: String,
        }

        let body = Request {
            model: &self.model,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
            temperature: self.temperature,
        };

        let mut last_error = None;
        for attempt in 0..=self.retries {
            match self
                .http
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(&self.key)
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    match response.json::<Response>().await {
                        Ok(resp) => {
                            return Ok(resp
                                .choices
                                .first()
                                .map(|c| c.message.content.clone())
                                .unwrap_or_else(|| "No response from ChatGPT".to_string()));
                        }
                        Err(e) => last_error = Some(ClientError::from(e)),
                    }
                }
                Err(e) => last_error = Some(ClientError::from(e)),
            }
            
            if attempt < self.retries {
                tokio::time::sleep(Duration::from_millis(1000 * (attempt + 1) as u64)).await;
            }
        }

        Err(last_error.unwrap())
    }

    fn name(&self) -> &str {
        "ChatGPT"
    }

    fn model(&self) -> &str {
        &self.model
    }
}