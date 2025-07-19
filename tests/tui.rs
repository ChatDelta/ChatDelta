use std::collections::HashMap;
use chatdelta_base::tui::{AppState, ProviderState};

#[tokio::test]
async fn test_app_state_new() {
    let mut states = HashMap::new();
    states.insert("OpenAI", ProviderState::Enabled);
    states.insert("Gemini", ProviderState::Disabled);
    states.insert("Claude", ProviderState::Enabled);

    let app = AppState::new(states);
    assert_eq!(app.providers.len(), 3);
    assert_eq!(app.providers[0].state, ProviderState::Enabled);
    assert_eq!(app.providers[1].state, ProviderState::Disabled);
    assert_eq!(app.providers[2].state, ProviderState::Enabled);
}
