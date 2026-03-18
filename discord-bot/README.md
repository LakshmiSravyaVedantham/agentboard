# Agentboard Discord Bot

Control AI coding agents from Discord.

## Setup

1. Create a Discord application and bot at https://discord.com/developers/applications
2. Enable "Message Content Intent" under Bot > Privileged Gateway Intents
3. Add `DISCORD_BOT_TOKEN` and `ANTHROPIC_API_KEY` to `../.env`:
   ```
   DISCORD_BOT_TOKEN=your-discord-bot-token
   ANTHROPIC_API_KEY=your-anthropic-key
   ```
4. Install and run:
   ```bash
   cd discord-bot
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   python bot.py
   ```

## Usage

Send a message in any channel where the bot is present:

```
Build landing page for consentmap, fix auth bug in synaptiq
```

The bot will:
1. Show a plan broken into teams — react ✅ to approve or ❌ to cancel
2. Spawn each team in a separate thread with live output
3. Post a consolidated summary when all teams finish

Other commands (no prefix needed):

- `status` — list all active teams and their state
- `kill Alpha` — kill a specific team by name

## Backend config

By default the bot runs `claude --print "{task}"` per team. Override this via
`agentboard.toml` in the project root:

```toml
[backend]
default = "claude"

[backends.claude]
command = "claude"
args = ["--print", "{task}"]

[limits]
max_concurrent_teams = 4
max_runtime_seconds = 600
```

## Notes

- Only the user who submitted the todo list can approve or cancel the plan
- Each team gets its own Discord thread for output streaming
- Output is posted in batches (every 5 seconds or 20 lines) to stay within Discord rate limits
- Teams time out after `max_runtime_seconds` (default 10 minutes)
