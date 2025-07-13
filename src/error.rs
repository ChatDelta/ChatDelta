//! Error types for the ChatDelta AI client library

use std::fmt;

/// Errors that can occur when using AI clients
#[derive(Debug)]
pub enum ClientError {
    /// Network-related errors (timeouts, connection failures, etc.)
    Network(String),
    /// API-specific errors (invalid responses, rate limits, etc.)
    Api(String),
    /// Authentication errors (invalid API keys, etc.)
    Authentication(String),
    /// Configuration errors (invalid parameters, etc.)
    Configuration(String),
    /// Response parsing errors
    Parse(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Network(msg) => write!(f, "Network error: {}", msg),
            ClientError::Api(msg) => write!(f, "API error: {}", msg),
            ClientError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            ClientError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            ClientError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ClientError::Network("Request timeout".to_string())
        } else if err.is_connect() {
            ClientError::Network("Connection failed".to_string())
        } else if err.status().is_some() {
            let status = err.status().unwrap();
            if status.as_u16() == 401 {
                ClientError::Authentication("Invalid API key".to_string())
            } else if status.as_u16() == 429 {
                ClientError::Api("Rate limit exceeded".to_string())
            } else {
                ClientError::Api(format!("HTTP {}: {}", status, err))
            }
        } else {
            ClientError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::Parse(format!("JSON parsing failed: {}", err))
    }
}