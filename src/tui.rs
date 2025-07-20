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

pub struct Provider {
    pub name: &'static str,
    pub state: ProviderState,
    pub chat_history: Vec<String>,
    pub client: Option<Box<dyn AiClient>>,
}

pub struct AppState {
    pub providers: Vec<Provider>,
    pub shared_input: String,
    pub selected_column: usize,
    pub scroll_positions: Vec<usize>,
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
                chat_history: vec![format!("Welcome to {name} chat!")],
                client,
            });
        }
        let scroll_positions = vec![0; providers.len()];
        Self { 
            providers, 
            shared_input: String::new(),
            selected_column: 0,
            scroll_positions,
        }
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
    
    pub fn send_to_active_providers(&mut self, prompt: &str, tx: mpsc::UnboundedSender<(usize, String)>) {
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
                        if tx_clone.send((idx, response)).is_err() {
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
    }
    
    pub fn select_previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
        }
    }
    
    pub fn select_next_column(&mut self) {
        if self.selected_column < self.providers.len() - 1 {
            self.selected_column += 1;
        }
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
            if let Some(provider) = self.providers.get(self.selected_column) {
                let max_scroll = provider.chat_history.len().saturating_sub(1);
                if *scroll_pos < max_scroll {
                    *scroll_pos += 1;
                }
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
    let (tx, mut rx) = mpsc::unbounded_channel::<(usize, String)>();
    
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            // Split into main area and input area
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
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
                    let visible_lines: Vec<&str> = provider.chat_history
                        .iter()
                        .flat_map(|msg| msg.lines())
                        .skip(scroll_pos)
                        .collect();
                    visible_lines.join("\n")
                } else {
                    "API key missing. Set environment variable to enable.".to_string()
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
            
            // Render shared input box
            let input_block = Block::default()
                .title("Shared Input (Enter: send, ←→: select column, ↑↓: scroll, Esc/q: quit)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            
            let input_para = Paragraph::new(format!("> {}", app.shared_input))
                .block(input_block)
                .style(Style::default().fg(Color::White));
            f.render_widget(input_para, main_chunks[1]);
            
            // Set cursor position in input field
            f.set_cursor(
                main_chunks[1].x + app.shared_input.len() as u16 + 3, // +3 for "> " prefix and border
                main_chunks[1].y + 1 // +1 for border
            );
        })?;

        // Check for async responses
        while let Ok((provider_idx, response)) = rx.try_recv() {
            app.handle_response(provider_idx, response);
        }
        
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
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
