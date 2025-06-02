# ChatDelta

ChatDelta is a command line tool for comparing answers from multiple AI models. It sends your prompt to ChatGPT, Gemini, and Claude, then asks Gemini to summarize the differences. The goal is to help you quickly see how various LLMs respond to the same question.

## Features

- Query ChatGPT, Gemini, and Claude with a single command
- Summarize the differences between responses
- Optional logging of prompts and replies to a text file for later review
- Simple configuration through environment variables
- Written in idiomatic Rust with plentiful comments to encourage new contributors

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

Set the following environment variables with your API keys:

- `OPENAI_API_KEY` – used for ChatGPT
- `GEMINI_API_KEY` – used for Gemini
- `ANTHROPIC_API_KEY` – used for Claude

### Getting API keys

1. **Gemini** – Visit [aistudio.google.com/apikey](https://aistudio.google.com/apikey),
   create a key, and copy it.
2. **ChatGPT** – Go to [platform.openai.com/account/api-keys](https://platform.openai.com/account/api-keys)
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

Run the CLI with your prompt:

```bash
./chatdelta "Explain quantum computing" --log session.txt
```

The example above stores the prompt, every model reply, and the final digest into `session.txt`.

## How It Works

1. Your prompt is sent to ChatGPT, Gemini, and Claude in parallel.
2. Their replies are fed to Gemini with instructions to highlight the differences.
3. The digest from Gemini is printed to the terminal and optionally written to a file.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to help.

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

Made with <3 by DavidCanHelp and Codex.
