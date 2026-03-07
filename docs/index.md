# ChatDelta

Query OpenAI, Gemini, and Claude in parallel — compare responses, run structured debates, and stream output from a single command.

## CLI

```bash
cargo install chatdelta-cli
```

Set at least one API key:

```bash
export OPENAI_API_KEY="..."
export GEMINI_API_KEY="..."
export ANTHROPIC_API_KEY="..."
```

Run a query across all configured models:

```bash
chatdelta "What are the tradeoffs of microservices?"
```

Stream a single model's response as tokens arrive:

```bash
chatdelta --only claude --stream "Explain Rust's borrow checker"
```

Set a system prompt:

```bash
chatdelta --only claude --system-prompt "You are a Rust expert" "Review this approach: ..."
```

Show token usage and latency per model:

```bash
chatdelta --show-usage "Summarize the CAP theorem"
```

### Debate Mode

Run a structured debate between two models, with a third as moderator:

```bash
chatdelta debate \
  --model-a openai:gpt-4o \
  --model-b anthropic:claude-sonnet-4-6 \
  --moderator google:gemini-2.5-flash \
  --rounds 1 \
  --prompt "LLMs will make software engineers less productive over the next five years" \
  --export debate.md
```

The moderator produces a structured report: strongest points from each side, shared conclusions, unresolved disagreements, and claims requiring verification.

[See an example debate transcript](https://github.com/ChatDelta/chatdelta-cli/blob/main/docs/debate-example.md)

### Interactive Conversation

Use `--only` to specify which model you want to talk to:

```bash
chatdelta --conversation --only claude --system-prompt "You are a Rust expert"
```

Save and resume sessions:

```bash
chatdelta --conversation --only claude --save-conversation session.json
chatdelta --conversation --only claude --load-conversation session.json
```

## Rust Library

```toml
[dependencies]
chatdelta = "0.8"
```

```rust
use chatdelta::{create_client, execute_parallel, ClientConfig};

let config = ClientConfig::builder()
    .system_message("You are a helpful assistant")
    .temperature(0.7)
    .build();

let client = create_client("anthropic", "your-key", "claude-sonnet-4-6", config)?;
let response = client.send_prompt("Hello, world!").await?;
```

Parallel execution across models:

```rust
let results = execute_parallel(clients, "Explain quantum computing").await;
```

With token metadata:

```rust
let results = execute_parallel_with_metadata(clients, "Explain quantum computing").await;
// results include tokens_used, latency_ms, finish_reason per model
```

[Rust docs on docs.rs](https://docs.rs/chatdelta) · [crates.io](https://crates.io/crates/chatdelta)

## JavaScript/TypeScript

```bash
npm install chatdelta
```

```javascript
import { createClient, executeParallel } from 'chatdelta';

const openai = createClient('openai', process.env.OPENAI_KEY, 'gpt-4o');
const results = await executeParallel([openai], 'Explain quantum computing');
```

[NPM package](https://www.npmjs.com/package/chatdelta)

## Go

```bash
go get github.com/chatdelta/chatdelta-go
```

```go
client, err := chatdelta.CreateClient("openai", os.Getenv("OPENAI_KEY"), "gpt-4o", nil)
response, err := client.SendPrompt(context.Background(), "What is AI?")
```

[Go package docs](https://pkg.go.dev/github.com/chatdelta/chatdelta-go) *(in development)*

## Links

- [GitHub Organization](https://github.com/ChatDelta)
- [chatdelta-cli](https://github.com/ChatDelta/chatdelta-cli) — the CLI tool
- [chatdelta-rs](https://github.com/ChatDelta/chatdelta-rs) — the Rust library
- MIT licensed
