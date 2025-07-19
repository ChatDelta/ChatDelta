# ChatDelta

ChatDelta is a terminal user interface (TUI) application for chatting with multiple AI providers side by side. When launched it opens a full screen interface with three columns:

- **OpenAI**
- **Gemini**
- **Claude**

Each column shows the conversation with its provider. Columns are greyed out if the required API key is not set.

At the bottom of the screen is a shared input box. Type a message and press <kbd>Enter</kbd> to send it to all enabled providers. Replies appear asynchronously in their respective columns. Press <kbd>Esc</kbd> or `q` to quit.

## Features

- Side-by-side chat with OpenAI, Gemini and Claude
- Columns automatically disable when the API key is missing
- Shared input so you can ask all providers the same question
- Asynchronous responses update the display while each AI thinks
- Written in Rust using `tui` and `crossterm`

## Installation

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone this repository:
   ```bash
   git clone https://github.com/ChatDelta/chatdelta.git
   cd chatdelta
   ```
3. Build the binary:
   ```bash
   cargo build --release
   ```
   The resulting executable will be in `target/release/chatdelta`.

## Usage

Set your API keys as environment variables before running the program:

- `OPENAI_API_KEY` – OpenAI
- `GEMINI_API_KEY` – Gemini
- `ANTHROPIC_API_KEY` – Claude

### Getting API keys

1. **Gemini** – Visit [aistudio.google.com/apikey](https://aistudio.google.com/apikey),
   create a key, and copy it.
2. **OpenAI** – Go to [platform.openai.com/account/api-keys](https://platform.openai.com/account/api-keys)
   and generate a new secret key.
3. **Claude** – Create a key in the dashboard at
   [console.anthropic.com](https://console.anthropic.com).

Add the keys to your shell configuration so they are available every time you run
the CLI. For example, in `~/.zshrc`:

```shell
export OPENAI_API_KEY="sk-your-openai-key"
export GEMINI_API_KEY="your-gemini-key"
export ANTHROPIC_API_KEY="your-claude-key"
```

Reload your shell or run `source ~/.zshrc` for the variables to take effect.

Launch the TUI:

```bash
OPENAI_API_KEY=... GEMINI_API_KEY=... ANTHROPIC_API_KEY=... ./target/release/chatdelta
```

If a key is missing, the corresponding column is dimmed and instructs you to set the variable.

Type your prompt in the input box and press <kbd>Enter</kbd> to send it. Press <kbd>Esc</kbd> or `q` to exit the interface.

## Testing

Run the automated tests with Cargo:

```bash
cargo test
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

Made with <3 by DavidCanHelp and Codex.
