//! Terminal UI for ChatDelta
//!
//! Displays a column for each AI provider (OpenAI, Gemini, Claude). If the API key is missing, the column is greyed out.

use std::collections::HashMap;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Terminal;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::execute;
use crossterm::cursor;
use std::io;
use chatdelta::{create_client, AiClient, ClientConfig};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone)]
pub enum ResponseType {
    Provider(usize, String),  // (provider_index, response)
    Delta(String),            // delta analysis
}

pub struct Provider {
    pub name: &'static str,
    pub state: ProviderState,
    pub chat_history: Vec<String>,
    pub client: Option<Box<dyn AiClient>>,
}

pub struct AppState {
    pub providers: Vec<Provider>,
    pub shared_input: String,
    pub selected_column: usize, // 0-2 for providers, 3 for delta field
    pub scroll_positions: Vec<usize>, // index 3 will be for delta field
    pub delta_text: String,
    pub show_delta: bool,
}

impl AppState {
    pub fn new(provider_states: HashMap<&'static str, ProviderState>) -> Self {
        let mut providers = Vec::new();
        let config = ClientConfig::default();
        
        for &name in ["ChatGPT", "Gemini", "Claude"].iter() {
            let state = *provider_states.get(name).unwrap_or(&ProviderState::Disabled);
            let client = if state == ProviderState::Enabled {
                Self::create_provider_client(name, &config)
            } else {
                None
            };
            
            providers.push(Provider {
                name,
                state,
                chat_history: vec![Self::create_welcome_message(name)],
                client,
            });
        }
        let scroll_positions = vec![0; providers.len() + 1]; // +1 for delta field
        Self { 
            providers, 
            shared_input: String::new(),
            selected_column: 0,
            scroll_positions,
            delta_text: "🔍 Differences between AI responses will appear here after you send a query to multiple providers".to_string(),
            show_delta: true,
        }
    }
    
    fn create_welcome_message(name: &str) -> String {
        match name {
            "ChatGPT" => {
                "🤖 Welcome to ChatGPT!\n\n🧠 Model: GPT-4o\n🏢 Provider: OpenAI\n\n✨ Ready to assist with your queries!\nI excel at general knowledge, coding, writing, and analysis."
            },
            "Gemini" => {
                "🌟 Welcome to Gemini!\n\n🚀 Model: Gemini-1.5-Pro\n🏢 Provider: Google\n\n🎯 Ready for action!\nI'm great at multimodal tasks, long context understanding, and creative problem-solving."
            },
            "Claude" => {
                "🎭 Welcome to Claude!\n\n🧬 Model: Claude-3.5-Sonnet\n🏢 Provider: Anthropic\n\n👋 Hello there!\nI'm designed to be helpful, harmless, and honest. I excel at analysis, writing, coding, and thoughtful conversation."
            },
            _ => "🤖 Welcome to AI Chat!\n\nReady to help with your questions!"
        }.to_string()
    }
    
    fn create_provider_client(name: &str, config: &ClientConfig) -> Option<Box<dyn AiClient>> {
        let (env_var, provider_name, model) = match name {
            "ChatGPT" => ("CHATGPT_API_KEY", "openai", "gpt-4o"),
            "Gemini" => ("GEMINI_API_KEY", "gemini", "gemini-1.5-pro"),
            "Claude" => ("CLAUDE_API_KEY", "claude", "claude-3-5-sonnet-20241022"),
            _ => return None,
        };
        
        if let Ok(api_key) = std::env::var(env_var) {
            create_client(provider_name, &api_key, model, config.clone()).ok()
        } else {
            None
        }
    }
    
    pub fn send_to_active_providers(&mut self, prompt: &str, tx: mpsc::UnboundedSender<ResponseType>) {
        let prompt = prompt.to_string();
        for (idx, provider) in self.providers.iter_mut().enumerate() {
            if let Some(_client) = &provider.client {
                provider.chat_history.push(format!("You: {}", prompt));
                provider.chat_history.push(format!("{}: Thinking...", provider.name));
                
                // Get new client for the async task (since we can't move the trait object)
                let config = ClientConfig::default();
                if let Some(new_client) = Self::create_provider_client(provider.name, &config) {
                    let prompt_clone = prompt.clone();
                    let tx_clone = tx.clone();
                    
                    // Spawn async task for each provider
                    tokio::spawn(async move {
                        let response = match new_client.send_prompt(&prompt_clone).await {
                            Ok(resp) => resp,
                            Err(e) => format!("Error: {}", e),
                        };
                        
                        // Send result back
                        if tx_clone.send(ResponseType::Provider(idx, response)).is_err() {
                            eprintln!("Failed to send response");
                        }
                    });
                }
            }
        }
    }
    
    pub fn handle_response(&mut self, provider_idx: usize, response: String) {
        if let Some(provider) = self.providers.get_mut(provider_idx) {
            let provider_name = provider.name;
            // Replace "Thinking..." with actual response
            if let Some(last) = provider.chat_history.last_mut() {
                *last = format!("{}: {}", provider_name, response);
            }
        }
        
        // Note: Delta generation will be triggered from main loop after all responses are received
    }
    
    
    pub fn generate_delta_with_channel(&mut self, tx: mpsc::UnboundedSender<ResponseType>) {
        // Check if all enabled providers have recent responses (not "Thinking...")
        let all_responded = self.providers
            .iter()
            .filter(|p| p.state == ProviderState::Enabled)
            .all(|p| {
                p.chat_history.last()
                    .map(|msg| !msg.contains("Thinking..."))
                    .unwrap_or(false)
            });
            
        if !all_responded {
            return;
        }
        
        self.generate_delta_internal(tx);
    }
    
    fn generate_delta_internal(&mut self, tx: mpsc::UnboundedSender<ResponseType>) {
        // Get the latest responses from all enabled providers
        let responses: Vec<(String, String)> = self.providers
            .iter()
            .filter(|p| p.state == ProviderState::Enabled)
            .filter_map(|p| {
                p.chat_history.last().and_then(|msg| {
                    if let Some(colon_pos) = msg.find(": ") {
                        let response = &msg[colon_pos + 2..];
                        Some((p.name.to_string(), response.to_string()))
                    } else {
                        None
                    }
                })
            })
            .collect();
            
        if responses.len() >= 2 {
            // Create a Gemini client for delta analysis
            let config = ClientConfig::default();
            if let Some(gemini_client) = Self::create_provider_client("Gemini", &config) {
                let responses_clone = responses.clone();
                
                // Create async task for delta generation
                tokio::spawn(async move {
                    let prompt = Self::create_delta_prompt(&responses_clone);
                    match gemini_client.send_prompt(&prompt).await {
                        Ok(delta) => {
                            if tx.send(ResponseType::Delta(delta)).is_err() {
                                eprintln!("Failed to send delta response");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Error generating differences: {}", e);
                            if tx.send(ResponseType::Delta(error_msg)).is_err() {
                                eprintln!("Failed to send delta error");
                            }
                        }
                    }
                });
            }
            
            self.show_delta = true;
            self.delta_text = "Generating differences summary...".to_string();
        }
    }
    
    fn create_delta_prompt(responses: &[(String, String)]) -> String {
        let mut prompt = String::from("Please analyze the following AI responses to the same question and summarize the key differences between them. Focus on factual differences, different approaches, or varying perspectives. Be concise but thorough:\n\n");
        
        for (provider, response) in responses {
            prompt.push_str(&format!("**{}:**\n{}\n\n", provider, response));
        }
        
        prompt.push_str("**Summary of key differences:**");
        prompt
    }
    
    pub fn handle_delta_response(&mut self, delta: String) {
        self.delta_text = delta;
    }
    
    pub fn select_previous_column(&mut self) {
        let total_sections = self.providers.len() + 1; // +1 for delta field
        if self.selected_column == 0 {
            self.selected_column = total_sections - 1; // Wrap to last section (delta field)
        } else {
            self.selected_column -= 1;
        }
    }
    
    pub fn select_next_column(&mut self) {
        let total_sections = self.providers.len() + 1; // +1 for delta field
        self.selected_column = (self.selected_column + 1) % total_sections;
    }
    
    pub fn scroll_up(&mut self) {
        if let Some(scroll_pos) = self.scroll_positions.get_mut(self.selected_column) {
            if *scroll_pos > 0 {
                *scroll_pos -= 1;
            }
        }
    }
    
    pub fn scroll_down(&mut self) {
        if let Some(scroll_pos) = self.scroll_positions.get_mut(self.selected_column) {
            let max_scroll = if self.selected_column < self.providers.len() {
                // Provider column
                if let Some(provider) = self.providers.get(self.selected_column) {
                    let total_lines: usize = provider.chat_history
                        .iter()
                        .flat_map(|msg| msg.lines())
                        .count();
                    total_lines.saturating_sub(25) // Max visible lines is 25
                } else {
                    0
                }
            } else {
                // Delta field
                let total_lines = self.delta_text.lines().count();
                total_lines.saturating_sub(4) // Visible lines in delta field
            };
            
            if *scroll_pos < max_scroll {
                *scroll_pos += 1;
            }
        }
    }
}

pub async fn run_tui(provider_states: HashMap<&'static str, ProviderState>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::Hide)?;
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = AppState::new(provider_states);
    
    // Create channel for async responses
    let (tx, mut rx) = mpsc::unbounded_channel::<ResponseType>();
    
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            // Split into main area, delta area, and input area
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),           // Main provider columns
                    Constraint::Length(6),        // Delta field
                    Constraint::Length(3)         // Input field
                ])
                .split(size);
            
            // Split main area into 3 columns
            let provider_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(main_chunks[0]);

            // Render provider columns
            for (i, provider) in app.providers.iter().enumerate() {
                let is_selected = i == app.selected_column;
                let title = if is_selected {
                    format!("► {} ◄", provider.name)
                } else {
                    provider.name.to_string()
                };
                
                let block = Block::default()
                    .title(Span::styled(
                        title,
                        Style::default().fg(if provider.state == ProviderState::Enabled {
                            if is_selected { Color::Yellow } else { Color::Cyan }
                        } else {
                            Color::DarkGray
                        }),
                    ))
                    .borders(Borders::ALL)
                    .border_style(if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    });

                let chat = if provider.state == ProviderState::Enabled {
                    let scroll_pos = app.scroll_positions.get(i).copied().unwrap_or(0);
                    let all_lines: Vec<&str> = provider.chat_history
                        .iter()
                        .flat_map(|msg| msg.lines())
                        .collect();
                    
                    // Apply scrolling and limit visible lines
                    let visible_lines: Vec<&str> = all_lines
                        .iter()
                        .skip(scroll_pos)
                        .take(25) // Show max 25 lines at once
                        .copied()
                        .collect();
                    
                    let mut content = visible_lines.join("\n");
                    
                    // Add scroll indicators
                    if scroll_pos > 0 {
                        content = format!("⬆️ (scroll up for more)\n{}", content);
                    }
                    if scroll_pos + visible_lines.len() < all_lines.len() {
                        content = format!("{}\n⬇️ (scroll down for more)", content);
                    }
                    
                    content
                } else {
                    "🔒 API key missing\n\nSet the appropriate environment variable to enable this provider:\n\n• CHATGPT_API_KEY for ChatGPT\n• GEMINI_API_KEY for Gemini\n• CLAUDE_API_KEY for Claude".to_string()
                };
                
                let para = Paragraph::new(chat)
                    .block(block)
                    .wrap(Wrap { trim: true })
                    .style(if provider.state == ProviderState::Enabled {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    });
                f.render_widget(para, provider_chunks[i]);
            }
            
            // Render delta field
            let delta_field_selected = app.selected_column == app.providers.len();
            let delta_title = if delta_field_selected {
                "► 🔍 Response Differences (powered by Gemini) ◄"
            } else {
                "🔍 Response Differences (powered by Gemini)"
            };
            
            let delta_block = Block::default()
                .title(Span::styled(
                    delta_title,
                    Style::default().fg(if delta_field_selected { Color::Yellow } else { Color::Magenta }),
                ))
                .borders(Borders::ALL)
                .border_style(if delta_field_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Magenta)
                });
            
            // Handle scrolling for delta field
            let delta_content = {
                let scroll_pos = app.scroll_positions.get(app.providers.len()).copied().unwrap_or(0);
                let all_lines: Vec<&str> = app.delta_text.lines().collect();
                
                let visible_lines: Vec<&str> = all_lines
                    .iter()
                    .skip(scroll_pos)
                    .take(4) // Show max 4 lines in delta field
                    .copied()
                    .collect();
                
                let mut content = visible_lines.join("\n");
                
                // Add scroll indicators for delta field when selected
                if delta_field_selected {
                    if scroll_pos > 0 {
                        content = format!("⬆️ (scroll up)\n{}", content);
                    }
                    if scroll_pos + visible_lines.len() < all_lines.len() {
                        content = format!("{}\n⬇️ (scroll down)", content);
                    }
                }
                
                content
            };
            
            let delta_para = Paragraph::new(delta_content)
                .block(delta_block)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::White));
            f.render_widget(delta_para, main_chunks[1]);
            
            // Render shared input box
            let input_block = Block::default()
                .title("Shared Input (Enter: send, ←→: cycle sections, ↑↓: scroll, Esc: quit)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            
            let input_para = Paragraph::new(format!("> {}", app.shared_input))
                .block(input_block)
                .style(Style::default().fg(Color::White));
            f.render_widget(input_para, main_chunks[2]);
            
            // Set cursor position in input field
            f.set_cursor(
                main_chunks[2].x + app.shared_input.len() as u16 + 3, // +3 for "> " prefix and border
                main_chunks[2].y + 1 // +1 for border
            );
        })?;

        // Check for async responses
        let mut responses_received = 0;
        while let Ok(response_type) = rx.try_recv() {
            match response_type {
                ResponseType::Provider(provider_idx, response) => {
                    app.handle_response(provider_idx, response);
                    responses_received += 1;
                }
                ResponseType::Delta(delta_text) => {
                    app.handle_delta_response(delta_text);
                }
            }
        }
        
        // Check if we should generate delta after receiving responses
        if responses_received > 0 {
            app.generate_delta_with_channel(tx.clone());
        }
        
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), cursor::Show)?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Left => {
                        app.select_previous_column();
                    }
                    KeyCode::Right => {
                        app.select_next_column();
                    }
                    KeyCode::Up => {
                        app.scroll_up();
                    }
                    KeyCode::Down => {
                        app.scroll_down();
                    }
                    KeyCode::Char(c) => {
                        app.shared_input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.shared_input.pop();
                    }
                    KeyCode::Enter => {
                        let msg = app.shared_input.trim().to_string();
                        if !msg.is_empty() {
                            app.send_to_active_providers(&msg, tx.clone());
                            app.shared_input.clear();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}
