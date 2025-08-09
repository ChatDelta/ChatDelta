//! Logging module for ChatDelta conversations
//!
//! Saves all conversations, responses, and delta analyses to JSON files in ~/.chatdelta/logs/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationLog {
    pub session_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub conversations: Vec<ConversationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub responses: HashMap<String, ProviderResponse>,
    pub delta_analysis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub text: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

pub struct Logger {
    log: ConversationLog,
    current_conversation: Option<ConversationEntry>,
    response_timers: HashMap<String, Instant>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            log: ConversationLog {
                session_id: Uuid::new_v4(),
                start_time: Utc::now(),
                end_time: None,
                conversations: Vec::new(),
            },
            current_conversation: None,
            response_timers: HashMap::new(),
        }
    }

    pub fn log_prompt(&mut self, prompt: &str) {
        let entry = ConversationEntry {
            timestamp: Utc::now(),
            prompt: prompt.to_string(),
            responses: HashMap::new(),
            delta_analysis: None,
        };
        self.current_conversation = Some(entry);
        self.response_timers.clear();
    }

    pub fn start_provider_timer(&mut self, provider: &str) {
        self.response_timers.insert(provider.to_string(), Instant::now());
    }

    pub fn log_provider_response(&mut self, provider: &str, response: &str, is_error: bool) {
        if let Some(ref mut conversation) = self.current_conversation {
            let latency_ms = self.response_timers
                .get(provider)
                .map(|start| start.elapsed().as_millis() as u64);

            let provider_response = if is_error {
                ProviderResponse {
                    text: String::new(),
                    latency_ms,
                    error: Some(response.to_string()),
                }
            } else {
                ProviderResponse {
                    text: response.to_string(),
                    latency_ms,
                    error: None,
                }
            };

            conversation.responses.insert(provider.to_string(), provider_response);
        }
    }

    pub fn log_delta_analysis(&mut self, delta: &str) {
        if let Some(ref mut conversation) = self.current_conversation {
            conversation.delta_analysis = Some(delta.to_string());
        }
        
        // Move the completed conversation to the log
        if let Some(conversation) = self.current_conversation.take() {
            self.log.conversations.push(conversation);
        }
    }

    pub fn finalize_conversation(&mut self) {
        // If there's a conversation without delta analysis, still save it
        if let Some(conversation) = self.current_conversation.take() {
            self.log.conversations.push(conversation);
        }
    }

    pub fn save(&mut self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.log.end_time = Some(Utc::now());
        
        // Create log directory structure
        let log_dir = self.get_log_directory()?;
        fs::create_dir_all(&log_dir)?;
        
        // Generate filename with timestamp and session ID
        let filename = format!(
            "session_{}_{}.json",
            self.log.start_time.format("%Y%m%d_%H%M%S"),
            &self.log.session_id.to_string()[..8] // First 8 chars of UUID
        );
        
        let file_path = log_dir.join(filename);
        
        // Write JSON to file
        let json = serde_json::to_string_pretty(&self.log)?;
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(file_path)
    }

    fn get_log_directory(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir()
            .ok_or("Could not determine home directory")?;
        
        let date_str = self.log.start_time.format("%Y-%m-%d").to_string();
        let log_dir = home_dir
            .join(".chatdelta")
            .join("logs")
            .join(date_str);
        
        Ok(log_dir)
    }

    pub fn session_id(&self) -> &Uuid {
        &self.log.session_id
    }

    pub fn start_time(&self) -> &DateTime<Utc> {
        &self.log.start_time
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new();
        assert_eq!(logger.log.conversations.len(), 0);
        assert!(logger.log.end_time.is_none());
    }

    #[test]
    fn test_log_conversation_flow() {
        let mut logger = Logger::new();
        
        // Log a prompt
        logger.log_prompt("What is Rust?");
        assert!(logger.current_conversation.is_some());
        
        // Log provider responses
        logger.start_provider_timer("ChatGPT");
        logger.log_provider_response("ChatGPT", "Rust is a systems programming language...", false);
        
        logger.start_provider_timer("Gemini");
        logger.log_provider_response("Gemini", "Rust is a modern programming language...", false);
        
        // Log delta analysis
        logger.log_delta_analysis("Both responses explain Rust as a programming language...");
        
        // Check that conversation was moved to log
        assert_eq!(logger.log.conversations.len(), 1);
        assert!(logger.current_conversation.is_none());
        
        let conversation = &logger.log.conversations[0];
        assert_eq!(conversation.prompt, "What is Rust?");
        assert_eq!(conversation.responses.len(), 2);
        assert!(conversation.delta_analysis.is_some());
    }

    #[test]
    fn test_error_response_logging() {
        let mut logger = Logger::new();
        
        logger.log_prompt("Test prompt");
        logger.start_provider_timer("ChatGPT");
        logger.log_provider_response("ChatGPT", "API key invalid", true);
        
        let conversation = logger.current_conversation.as_ref().unwrap();
        let response = conversation.responses.get("ChatGPT").unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap(), "API key invalid");
        assert_eq!(response.text, "");
    }
}