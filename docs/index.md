# ChatDelta

ChatDelta is an open source command line tool for connecting multiple AI APIs. This site hosts the documentation and quick start guide.

## Getting Started

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Clone the repository and build the binary:
   ```bash
   git clone https://github.com/ChatDelta/chatdelta.git
   cd chatdelta
   cargo build --release
   ```
3. Set your API keys as environment variables and run the CLI:
   ```bash
   CHATGPT_API_KEY=... GEMINI_API_KEY=... CLAUDE_API_KEY=... ./target/release/chatdelta "Your prompt"
   ```
4. Your prompt is sent to each API, and then all the responses are fed into Gemini for a combined summary.

For full usage details see the [README on GitHub](https://github.com/ChatDelta/ChatDelta/blob/main/README.md).

## Diagram

Below is a high level flow of how ChatDelta works. The diagram is generated from a Mermaid source file during the CI build.

![ChatDelta flow](assets/diagram.svg)
