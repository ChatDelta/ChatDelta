//! Metrics integration for ChatDelta TUI
//! Displays real-time performance metrics using the core library's ClientMetrics

use chatdelta::{ClientMetrics, MetricsSnapshot};
use std::collections::HashMap;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// TUI metrics manager
pub struct TuiMetrics {
    /// Metrics for each provider
    provider_metrics: HashMap<String, ClientMetrics>,
    /// Whether metrics display is enabled
    enabled: bool,
    /// Show detailed metrics
    detailed: bool,
}

impl TuiMetrics {
    pub fn new() -> Self {
        Self {
            provider_metrics: HashMap::new(),
            enabled: true,
            detailed: false,
        }
    }
    
    /// Toggle metrics display
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }
    
    /// Toggle detailed view
    pub fn toggle_detailed(&mut self) {
        self.detailed = !self.detailed;
    }
    
    /// Get or create metrics for a provider
    pub fn get_metrics(&mut self, provider: &str) -> ClientMetrics {
        self.provider_metrics
            .entry(provider.to_string())
            .or_insert_with(ClientMetrics::new)
            .clone()
    }
    
    /// Record API response
    pub fn record_response(&mut self, provider: &str, success: bool, latency_ms: u64, tokens: Option<u32>) {
        if let Some(metrics) = self.provider_metrics.get(provider) {
            metrics.record_request(success, latency_ms, tokens);
        }
    }
    
    /// Render metrics widget
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if !self.enabled {
            return;
        }
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Metrics content
            ])
            .split(area);
        
        // Title block
        let title = Paragraph::new("📊 Performance Metrics")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);
        
        // Metrics content
        let mut items: Vec<ListItem> = Vec::new();
        
        for (provider, metrics) in &self.provider_metrics {
            let stats = metrics.get_stats();
            
            if self.detailed {
                // Detailed view
                items.push(ListItem::new(Spans::from(vec![
                    Span::styled(
                        format!("{}: ", provider),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                ])));
                
                items.push(ListItem::new(Spans::from(vec![
                    Span::raw("  "),
                    Span::raw(format!(
                        "Requests: {} | Success: {:.1}% | Avg: {}ms",
                        stats.requests_total,
                        stats.success_rate,
                        stats.average_latency_ms
                    )),
                ])));
                
                if stats.total_tokens_used > 0 {
                    items.push(ListItem::new(Spans::from(vec![
                        Span::raw("  "),
                        Span::raw(format!("Tokens: {}", stats.total_tokens_used)),
                    ])));
                }
                
                if stats.cache_hit_rate > 0.0 {
                    items.push(ListItem::new(Spans::from(vec![
                        Span::raw("  "),
                        Span::raw(format!("Cache Hits: {:.1}%", stats.cache_hit_rate)),
                    ])));
                }
            } else {
                // Compact view
                let status_color = if stats.success_rate >= 90.0 {
                    Color::Green
                } else if stats.success_rate >= 70.0 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                
                items.push(ListItem::new(Spans::from(vec![
                    Span::styled(
                        format!("{}: ", provider),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:.0}% ", stats.success_rate),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(format!("{}ms", stats.average_latency_ms)),
                ])));
            }
        }
        
        if items.is_empty() {
            items.push(ListItem::new("No metrics available yet"));
        }
        
        let metrics_list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        
        f.render_widget(metrics_list, chunks[1]);
    }
    
    /// Get summary text for status bar
    pub fn get_summary(&self) -> String {
        let total_requests: u64 = self.provider_metrics
            .values()
            .map(|m| m.get_stats().requests_total)
            .sum();
        
        let total_success: u64 = self.provider_metrics
            .values()
            .map(|m| m.get_stats().requests_successful)
            .sum();
        
        if total_requests > 0 {
            let success_rate = (total_success as f64 / total_requests as f64) * 100.0;
            format!("📊 {} requests | {:.0}% success", total_requests, success_rate)
        } else {
            "📊 Metrics: Ready".to_string()
        }
    }
}

impl Default for TuiMetrics {
    fn default() -> Self {
        Self::new()
    }
}