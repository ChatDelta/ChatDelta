//! Google Gemini client implementation

use crate::{AiClient, ClientConfig, ClientError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for Google Gemini models
pub struct Gemini {
    http: Client,
    key: String,
    model: String,
    temperature: Option<f32>,
    retries: u32,
}

impl Gemini {
    /// Create a new Gemini client
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
impl AiClient for Gemini {
    async fn send_prompt(&self, prompt: &str) -> Result<String, ClientError> {
        #[derive(Serialize)]
        struct Part<'a> {
            text: &'a str,
        }
        
        #[derive(Serialize)]
        struct Content<'a> {
            role: &'a str,
            parts: Vec<Part<'a>>,
        }
        
        #[derive(Serialize)]
        struct Request<'a> {
            contents: Vec<Content<'a>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_config: Option<GenerationConfig>,
        }
        
        #[derive(Serialize)]
        struct GenerationConfig {
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
        }
        
        #[derive(Deserialize)]
        struct Response {
            candidates: Vec<Candidate>,
        }
        
        #[derive(Deserialize)]
        struct Candidate {
            content: CandContent,
        }
        
        #[derive(Deserialize)]
        struct CandContent {
            parts: Vec<CandPart>,
        }
        
        #[derive(Deserialize)]
        struct CandPart {
            text: String,
        }

        let body = Request {
            contents: vec![Content {
                role: "user",
                parts: vec![Part { text: prompt }],
            }],
            generation_config: self.temperature.map(|temp| GenerationConfig {
                temperature: Some(temp),
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.key
        );

        let mut last_error = None;
        for attempt in 0..=self.retries {
            match self
                .http
                .post(&url)
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    match response.json::<Response>().await {
                        Ok(resp) => {
                            return Ok(resp
                                .candidates
                                .first()
                                .and_then(|c| c.content.parts.first())
                                .map(|p| p.text.clone())
                                .unwrap_or_else(|| "No response from Gemini".to_string()));
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
        "Gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }
}