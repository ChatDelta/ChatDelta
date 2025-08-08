# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ChatDelta is a Rust-based TUI application that allows users to interact with multiple AI providers (OpenAI, Gemini, Claude) simultaneously in a side-by-side terminal interface. It displays responses from all providers to the same prompt, with an optional delta analysis showing differences between responses.

## Build and Development Commands

### Essential Commands
- **Build**: `cargo build --release` - Creates optimized binary in `target/release/chatdelta`
- **Check compilation**: `cargo check` - Verify code compiles without building
- **Run tests**: `cargo test` - Runs all unit and integration tests
- **Format code**: `cargo fmt` - Format code according to Rust conventions
- **Run locally**: `cargo run` - Build and run the debug version

### Running the Application
Set API keys as environment variables before running:
```bash
CHATGPT_API_KEY=... GEMINI_API_KEY=... CLAUDE_API_KEY=... cargo run
```

## Architecture

### Workspace Structure
- **Main application** (`/src/`): TUI binary and base library
  - `main.rs`: Entry point, initializes provider states based on API keys
  - `tui.rs`: Terminal UI implementation using `tui` and `crossterm`
  - `cli.rs`: Command-line interface logic
  - `output.rs`: Output formatting utilities
  
- **API Client Library** (`/chatdelta-rs/`): Core API client implementations
  - Provides `AiClient` trait and implementations for OpenAI, Gemini, and Claude
  - Uses `async_trait` for async API interactions
  - Each client handles its specific API format and authentication

### Key Design Patterns
1. **Provider Abstraction**: All AI providers implement the `AiClient` trait with a common `send_prompt` method
2. **State Management**: `AppState` manages all providers, chat history, and UI state including scroll positions and selected columns
3. **Async Communication**: Uses Tokio with mpsc channels for handling asynchronous API responses
4. **Dynamic Provider Detection**: Providers automatically enable/disable based on presence of API keys

### API Key Configuration
The application checks for these environment variables:
- `CHATGPT_API_KEY` for OpenAI (uses GPT-4o model)
- `GEMINI_API_KEY` for Google Gemini (uses Gemini-1.5-Pro)
- `CLAUDE_API_KEY` for Anthropic Claude (uses Claude-3.5-Sonnet)

### UI Layout
The TUI creates a 4-column layout:
1. Three provider columns (OpenAI, Gemini, Claude)
2. One delta analysis column
3. Shared input box at the bottom
4. Columns grey out when API keys are missing

## Testing Strategy
- Unit tests in `tests/cli.rs` and `tests/tui.rs`
- Run a single test: `cargo test test_name`
- Run with output: `cargo test -- --nocapture`