//! ChatDelta CLI application
//!
//! A command-line tool for querying multiple AI APIs and summarizing their responses.

use chatdelta_base::tui::{run_tui, ProviderState};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect provider API keys
    let mut provider_states = HashMap::new();
    provider_states.insert("ChatGPT", if std::env::var("CHATGPT_API_KEY").is_ok() { ProviderState::Enabled } else { ProviderState::Disabled });
    provider_states.insert("Gemini", if std::env::var("GEMINI_API_KEY").is_ok() { ProviderState::Enabled } else { ProviderState::Disabled });
    provider_states.insert("Claude", if std::env::var("CLAUDE_API_KEY").is_ok() { ProviderState::Enabled } else { ProviderState::Disabled });

    run_tui(provider_states).await?;
    Ok(())
}
