# Agentboard

Orchestrate multiple AI coding agents from your phone.

Send a todo list from your phone, an AI orchestrator breaks it into tasks, assigns each to a team, and each team runs your preferred AI coding CLI. Monitor progress in real-time, get clean summaries when done.

## Quick Start

```bash
# Install
cargo install agentboard

# Or build from source
git clone https://github.com/LakshmiSravyaVedantham/agentboard
cd agentboard
cargo build --release

# Configure
cp agentboard.toml.example agentboard.toml
export ANTHROPIC_API_KEY=sk-ant-...

# Run
./target/release/agentboard serve
```

Open the URL on your phone (same wifi), enter the pairing code, start orchestrating.

## How It Works

1. **Send** a todo list from your phone (text or voice)
2. **Orchestrator** (Claude/OpenAI) triages tasks into teams
3. **Review** the plan — approve, edit, or cancel
4. **Teams spawn** — each runs your preferred AI coding CLI
5. **Monitor** live terminal output from your phone
6. **Summary** delivered when teams finish

## Supported Backends

| Backend | CLI Command | Status |
|---------|-------------|--------|
| Claude Code | `claude --print` | Supported |
| Aider | `aider --message` | Supported |
| Codex CLI | `codex` | Supported |
| Gemini CLI | `gemini` | Supported |
| Goose | `goose run` | Supported |
| Custom | Configure any CLI | Supported |

## Configuration

See `agentboard.toml.example` for all options. Key sections:

- **`[server]`** — port, host
- **`[security]`** — allowed working directories, plan approval gate
- **`[orchestrator]`** — LLM provider, model, API key env var
- **`[limits]`** — max concurrent teams, timeout, output cap
- **`[backends.*]`** — configure CLI commands per backend

## Security

- **Mandatory auth** — 6-digit pairing code on startup
- **Working directory allowlist** — agents can only work in approved directories
- **Plan approval gate** — you review before any teams spawn
- **No telemetry** — zero data leaves your machine
- **API keys server-side only** — never exposed to the PWA

## Architecture

```
Phone (PWA) <-> Axum Server (your machine)
                    |
              Orchestrator (LLM)
                    |
            +---+---+---+
            T1  T2  T3  T4  (AI CLI subprocesses)
```

## Tech Stack

- **Server:** Rust (axum, tokio)
- **Frontend:** Svelte, TailwindCSS
- **Build:** Single binary with embedded frontend (rust-embed)

## License

MIT
