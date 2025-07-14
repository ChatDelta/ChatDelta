//! Terminal UI for ChatDelta
//!
//! Displays a column for each AI provider (OpenAI, Gemini, Claude). If the API key is missing, the column is greyed out.

use std::collections::HashMap;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderState {
    Enabled,
    Disabled,
}

pub struct Provider {
    pub name: &'static str,
    pub state: ProviderState,
    pub chat_history: Vec<String>,
    pub input: String,
}

pub struct AppState {
    pub providers: Vec<Provider>,
    pub selected: usize,
}

impl AppState {
    pub fn new(provider_states: HashMap<&'static str, ProviderState>) -> Self {
        let mut providers = Vec::new();
        for &name in ["OpenAI", "Gemini", "Claude"].iter() {
            providers.push(Provider {
                name,
                state: *provider_states.get(name).unwrap_or(&ProviderState::Disabled),
                chat_history: vec![format!("Welcome to {name} chat!")],
                input: String::new(),
            });
        }
        Self { providers, selected: 0 }
    }
}

pub fn run_tui(provider_states: HashMap<&'static str, ProviderState>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(provider_states);
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(size);

            for (i, provider) in app.providers.iter().enumerate() {
                let block = Block::default()
                    .title(Span::styled(
                        provider.name,
                        Style::default().fg(if provider.state == ProviderState::Enabled {
                            Color::Cyan
                        } else {
                            Color::DarkGray
                        }),
                    ))
                    .borders(Borders::ALL)
                    .border_style(if i == app.selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    });

                let chat = if provider.state == ProviderState::Enabled {
                    provider.chat_history.join("\n")
                } else {
                    "API key missing. Set environment variable to enable.".to_string()
                };
                let input = if provider.state == ProviderState::Enabled {
                    format!("\n> {}", provider.input)
                } else {
                    "\n[disabled]".to_string()
                };
                let para = Paragraph::new(vec![Spans::from(chat), Spans::from(input)])
                    .block(block)
                    .style(if provider.state == ProviderState::Enabled {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    });
                f.render_widget(para, chunks[i]);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Tab => {
                        let n = app.providers.len();
                        app.selected = (app.selected + 1) % n;
                    }
                    KeyCode::BackTab => {
                        let n = app.providers.len();
                        app.selected = (app.selected + n - 1) % n;
                    }
                    KeyCode::Char(c) => {
                        if app.providers[app.selected].state == ProviderState::Enabled {
                            app.providers[app.selected].input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if app.providers[app.selected].state == ProviderState::Enabled {
                            app.providers[app.selected].input.pop();
                        }
                    }
                    KeyCode::Enter => {
                        if app.providers[app.selected].state == ProviderState::Enabled {
                            let msg = app.providers[app.selected].input.trim().to_string();
                            if !msg.is_empty() {
                                app.providers[app.selected].chat_history.push(format!("You: {}", msg));
                                // TODO: async send to API and append response
                                app.providers[app.selected].input.clear();
                            }
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
