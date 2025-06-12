use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Command line arguments for chatdelta.
#[derive(Parser, Debug)]
#[command(version, about = "Query multiple AIs and connect their responses")]
struct Args {
    /// Prompt to send to the AIs
    #[arg(required = true)]
    prompt: String,

    /// Optional path to log the full interaction
    #[arg(long)]
    log: Option<PathBuf>,
}

impl Args {
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        // Test basic argument parsing
        let args = Args::try_parse_from(["chatdelta", "Hello, world!"])
            .expect("Failed to parse basic arguments");
        assert_eq!(args.prompt, "Hello, world!");
        assert!(args.log.is_none());
        args.validate().expect("Valid arguments should pass validation");

        // Test with log file path
        let args = Args::try_parse_from(["chatdelta", "Hello, world!", "--log", "interaction.log"])
            .expect("Failed to parse arguments with log path");
        assert_eq!(args.prompt, "Hello, world!");
        assert_eq!(args.log.clone().unwrap().to_str().unwrap(), "interaction.log");
        args.validate().expect("Valid arguments should pass validation");

        // Test with empty prompt (should fail)
        let args = Args::try_parse_from(["chatdelta", ""])
            .expect("Empty prompt should be accepted by clap but fail validation");
        assert!(args.validate().is_err());

        // Test with only program name (should fail)
        assert!(Args::try_parse_from(["chatdelta"]).is_err());
    }
}

/// Common trait implemented by all AI clients.
#[async_trait::async_trait]
trait AiClient {
    /// Sends a prompt and returns the textual response.
    async fn send_prompt(&self, prompt: &str) -> Result<String, reqwest::Error>;
}

/// Client for OpenAI's ChatGPT models.
struct ChatGpt {
    http: Client,
    key: String,
}

#[async_trait::async_trait]
impl AiClient for ChatGpt {
    async fn send_prompt(&self, prompt: &str) -> Result<String, reqwest::Error> {
        #[derive(Serialize)]
        struct Message<'a> {
            role: &'a str,
            content: &'a str,
        }
        #[derive(Serialize)]
        struct Request<'a> {
            model: &'a str,
            messages: Vec<Message<'a>>,
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
            model: "gpt-3.5-turbo",
            messages: vec![Message { role: "user", content: prompt }],
        };

        let resp = self
            .http
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

/// Client for Google Gemini models.
struct Gemini {
    http: Client,
    key: String,
}

#[async_trait::async_trait]
impl AiClient for Gemini {
    async fn send_prompt(&self, prompt: &str) -> Result<String, reqwest::Error> {
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
        };

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}", self.key);

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(resp
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default())
    }
}

/// Client for Anthropic Claude models.
struct Claude {
    http: Client,
    key: String,
}

#[async_trait::async_trait]
impl AiClient for Claude {
    async fn send_prompt(&self, prompt: &str) -> Result<String, reqwest::Error> {
        #[derive(Serialize)]
        struct Message<'a> {
            role: &'a str,
            content: &'a str,
        }
        #[derive(Serialize)]
        struct Request<'a> {
            model: &'a str,
            messages: Vec<Message<'a>>,
            max_tokens: u32,
        }
        #[derive(Deserialize)]
        struct Response {
            choices: Vec<Choice>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: Resp,
        }
        #[derive(Deserialize)]
        struct Resp {
            content: String,
        }

        let body = Request {
            model: "claude-3-opus-20240229",
            messages: vec![Message { role: "user", content: prompt }],
            max_tokens: 1024,
        };

        let resp = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
}

/// Runs the chat flow and optionally logs the interaction.
async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Load API keys from environment.
    let chatgpt = ChatGpt { http: client.clone(), key: env::var("OPENAI_API_KEY")? };
    let gemini = Gemini { http: client.clone(), key: env::var("GEMINI_API_KEY")? };
    let claude = Claude { http: client.clone(), key: env::var("ANTHROPIC_API_KEY")? };

    // Query each model with the same prompt.
    let gpt_reply = chatgpt.send_prompt(&args.prompt).await?;
    let gemini_reply = gemini.send_prompt(&args.prompt).await?;
    let claude_reply = claude.send_prompt(&args.prompt).await?;

    // Summarize differences using Gemini.
    let summary_prompt = format!(
        "Given these model replies:\nChatGPT: {}\n---\nGemini: {}\n---\nClaude: {}\nSummarize the key differences.",
        gpt_reply, gemini_reply, claude_reply
    );
    let digest = gemini.send_prompt(&summary_prompt).await?;

    println!("{}", digest);

    if let Some(path) = &args.log {
        let mut file = File::create(path)?;
        writeln!(file, "Prompt:\n{}\n", args.prompt)?;
        writeln!(file, "ChatGPT:\n{}\n", gpt_reply)?;
        writeln!(file, "Gemini:\n{}\n", gemini_reply)?;
        writeln!(file, "Claude:\n{}\n", claude_reply)?;
        writeln!(file, "Digest:\n{}\n", digest)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Err(e) = run(args).await {
        eprintln!("Error: {e}");
    }
    Ok(())
}

