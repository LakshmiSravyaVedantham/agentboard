---
title: I Built a Tool to Control AI Coding Agents from My Phone
published: true
tags: ai, productivity, opensource, rust
---

## The Problem

You're on the couch, in a coffee shop, or just away from your desk. You have a list of coding tasks you want done. You have AI coding agents (Claude Code, Aider, Codex, Gemini CLI) installed on your machine. But you can't use them from your phone.

What if you could just text your tasks from Discord and have multiple AI agents start working on them — each in its own terminal?

## What I Built

**Agentboard** — an open-source tool that lets you orchestrate multiple AI coding agents from your phone via Discord.

You type:

```
team Alpha: Build a landing page for my portfolio
team Beta: Fix the auth bug in my API
team Gamma: Write tests for the payment module
```

And on your Mac, three Terminal windows open — each running Claude (or Aider, Codex, Gemini, whatever you configure) working on its assigned task. In parallel.

<!-- INSERT SCREENSHOT: Discord showing team assignment + terminal menu -->

## How It Works

```
You (Discord on phone)
    |
    v
Discord Bot (Python, runs on your machine)
    |
    +-- Team Alpha -> Terminal.app -> claude "Build landing page..."
    +-- Team Beta  -> Terminal.app -> claude "Fix auth bug..."
    +-- Team Gamma -> Terminal.app -> claude "Write tests..."
```

1. You send a message in Discord with `team Name: task`
2. The bot parses your message
3. Opens a real Terminal window per team
4. Runs your AI coding CLI interactively
5. Each team works independently, in parallel

No API credits needed for the bot itself — it uses your existing Claude Code subscription (or whatever CLI you have installed).

## The Pluggable Backend

Agentboard isn't locked to one AI tool. It works with any CLI:

| Backend | Command |
|---|---|
| Claude Code | `claude "task"` |
| Aider | `aider --message "task"` |
| Codex CLI | `codex "task"` |
| Gemini CLI | `gemini "task"` |
| Goose | `goose run "task"` |
| Any CLI | Configure in `agentboard.toml` |

## Quick Start

```bash
# Clone
git clone https://github.com/LakshmiSravyaVedantham/agentboard
cd agentboard/discord-bot

# Setup
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# Add your Discord bot token to .env
echo 'DISCORD_BOT_TOKEN=your-token' > ../.env

# Run
python bot.py
```

Then in Discord:
```
team Alpha: Build a hello world Python script
```

A Terminal window opens on your Mac with Claude working on it.

## Setting Up the Discord Bot (2 minutes)

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. New Application -> name it "Agentboard"
3. Bot tab -> Reset Token -> copy it
4. Enable **Message Content Intent**
5. OAuth2 -> URL Generator -> scope: `bot` -> permissions: Send Messages, Manage Channels, Read Message History
6. Open the generated URL -> add bot to your server

## What's Under the Hood

Agentboard also includes a full Rust backend (for those who want it):

- **17 unit tests** covering config, auth, team state machine, backend registry, orchestrator
- **Axum server** with 12 API endpoints + WebSocket
- **Svelte PWA** with dark glassmorphism UI (pairing screen, team dashboard, live terminal output)
- **JWT auth** with 6-digit pairing codes
- **Pluggable backends** via TOML config
- **Graceful shutdown** — SIGTERM to all child processes

The Rust server compiles to a single 7.2MB binary with the frontend embedded via `rust-embed`.

But honestly? The Discord bot is the killer feature. It's 150 lines of Python and it just works.

## Commands

| You type | What happens |
|---|---|
| `team Alpha: Build a landing page` | Opens Terminal with Claude working on it |
| `team Alpha: Fix auth in ~/myproject` | Opens Terminal in that directory |
| `status` | Shows all active teams |
| `kill Alpha` | Kills a team |
| `help` | Shows all commands |

## Pick Your Terminal

When you send a task, the bot asks which AI coding tool to use:

```
Which terminal should I use?
1️⃣ Claude
2️⃣ Codex
3️⃣ Aider
4️⃣ Gemini
5️⃣ Goose
6️⃣ OpenCode
```

React with a number. Each team can use a different tool if you want.

<!-- INSERT SCREENSHOT: Terminal split panes with two teams running -->

## Split Panes + Named Sessions

Multiple teams open as **split panes** in the same Terminal window — not separate windows cluttering your desktop. Each Claude session gets a **named session** (visible in the title bar and `/resume`), so you always know which team is which.

<!-- INSERT SCREENSHOT: Discord #team-alpha channel with streamed output -->

## Discord Channels Per Team

Each team gets its own **Discord channel** (`#team-alpha`, `#team-beta`). Output streams into the channel in real-time, so you can monitor progress from your phone without switching windows.

## Why Discord?

I tried building a PWA first. It worked, but:
- Localhost firewall issues on phones
- Safari blocks Web Speech API without HTTPS
- Another app to maintain

Discord just works. It's on your phone, it handles notifications, and your data stays in your private server. The bot is 150 lines of Python with zero infrastructure cost.

## What I Learned

1. **Claude Code's `--print` flag uses API credits, not your subscription.** The interactive `claude` command uses your subscription. Big difference when you're spawning multiple agents.

2. **Multiple Discord bots can pile up.** When developing, always `pkill -9 -f bot.py` before starting a new instance. I had 6 zombie bots responding to the same messages.

3. **The simplest approach wins.** I designed an elaborate Rust+Svelte+WebSocket+JWT system. The Discord bot that actually ships is 150 lines of Python that opens Terminal windows. Ship the simple thing.

## Try It

Repo: [github.com/LakshmiSravyaVedantham/agentboard](https://github.com/LakshmiSravyaVedantham/agentboard)

Star it if you find it useful. PRs welcome — especially for Linux/Windows terminal support.

---

*Agentboard is open source (MIT). No telemetry, no cloud, no data leaves your machine.*
