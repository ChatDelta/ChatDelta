# ChatDelta

ChatDelta is a command line tool for connecting multiple AI APIs. It sends your prompt to ChatGPT, Gemini, and Claude, then asks Gemini to generate a concise summary of all the responses. The goal is to help you quickly connect with different LLMs using a single command.

## Features

- Query ChatGPT, Gemini, and Claude with a single command
- Summarize the responses from each API
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

- `CHATGPT_API_KEY` – used for ChatGPT
- `GEMINI_API_KEY` – used for Gemini
- `CLAUDE_API_KEY` – used for Claude

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
export CHATGPT_API_KEY="sk-your-openai-key"
export GEMINI_API_KEY="your-gemini-key"
export CLAUDE_API_KEY="your-claude-key"
```

Reload your shell or run `source ~/.zshrc` for the variables to take effect.

Run the CLI with your prompt:

```bash
./chatdelta "Explain quantum computing" --log session.txt
```

The example above stores the prompt, every model reply, and the final digest into `session.txt`.

## How It Works

1. Your prompt is sent to ChatGPT, Gemini, and Claude **in parallel** for faster responses.
2. Their replies are fed to Gemini for a combined summary of the responses.
3. The digest from Gemini is printed to the terminal and optionally written to a file.
4. Progress indicators show the status of each step.
5. Detailed error messages help troubleshoot API issues.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to help.

## Website

Documentation is hosted with GitHub Pages from the `docs/` directory. If you own
`ChatDelta.com`, point the domain's DNS at GitHub and add it as a custom domain
in the repository settings. The [`docs/CNAME`](docs/CNAME) file already contains
the domain name so GitHub Pages will serve the site at that address.

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

Made with <3 by DavidCanHelp and Codex.
