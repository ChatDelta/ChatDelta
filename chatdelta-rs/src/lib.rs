use async_trait::async_trait;
use std::error::Error;
use serde::{Deserialize, Serialize};

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

pub fn create_client(provider: &str, api_key: &str, model: &str, _config: ClientConfig) -> Result<Box<dyn AiClient>, Box<dyn Error + Send + Sync>> {
    match provider {
        "openai" => Ok(Box::new(OpenAIClient::new(api_key, model))),
        "gemini" => Ok(Box::new(GeminiClient::new(api_key, model))),
        "claude" => Ok(Box::new(ClaudeClient::new(api_key, model))),
        _ => Err(format!("Unknown provider: {}", provider).into()),
    }
}

// OpenAI Client
struct OpenAIClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[async_trait]
impl AiClient for OpenAIClient {
    async fn send_prompt(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 1000,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("OpenAI API error: {}", response.status()).into());
        }

        let openai_response: OpenAIResponse = response.json().await?;
        let content = openai_response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "No response".to_string());

        Ok(content)
    }
}

// Gemini Client
struct GeminiClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiClient {
    fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize, Clone)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize, Clone)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize, Clone)]
struct GeminiResponsePart {
    text: String,
}

#[async_trait]
impl AiClient for GeminiClient {
    async fn send_prompt(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Gemini API error: {}", response.status()).into());
        }

        let gemini_response: GeminiResponse = response.json().await?;
        let content = gemini_response.candidates
            .and_then(|candidates| candidates.first().cloned())
            .and_then(|candidate| candidate.content.parts.first().cloned())
            .map(|part| part.text)
            .unwrap_or_else(|| "No response".to_string());

        Ok(content)
    }
}

// Claude Client
struct ClaudeClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeClient {
    fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[async_trait]
impl AiClient for ClaudeClient {
    async fn send_prompt(&self, prompt: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1000,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Claude API error: {} - {}", status, error_text).into());
        }

        let claude_response: ClaudeResponse = response.json().await?;
        let content = claude_response.content
            .first()
            .map(|content| content.text.clone())
            .unwrap_or_else(|| "No response".to_string());

        Ok(content)
    }
}
