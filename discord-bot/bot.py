import os
import sys
import asyncio
import re
import subprocess
import discord
from discord.ext import commands
from dotenv import load_dotenv
from pathlib import Path

# Load env from parent directory
load_dotenv(Path(__file__).parent.parent / '.env')

DISCORD_TOKEN = os.getenv('DISCORD_BOT_TOKEN')

if not DISCORD_TOKEN:
    print("ERROR: DISCORD_BOT_TOKEN not set in .env")
    sys.exit(1)

# Bot setup
intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix='!', intents=intents)

# Team tracking
teams = {}

HELP_TEXT = """**Agentboard Bot**

**Assign tasks to teams (opens a Terminal window per team):**
```
team Alpha: Build a landing page for consentmap
team Beta: Fix auth bug in synaptiq
```

**Multiple teams at once:**
```
team Alpha: Build landing page in ~/consentmap
team Beta: Fix auth bug in ~/synaptiq
team Gamma: Write tests for datawipe
```

**Other commands:**
- `status` — show all active teams
- `kill Alpha` — kill a specific team
- `help` — show this message
"""


def parse_team_assignments(content):
    """Parse 'team Name: task' lines from message."""
    pattern = r'team\s+(\w+)\s*:\s*(.+)'
    matches = re.findall(pattern, content, re.IGNORECASE | re.MULTILINE)
    result = []
    for name, task in matches:
        dir_match = re.search(r'\b(?:in|dir|at)\s+(~/\S+)', task)
        working_dir = dir_match.group(1) if dir_match else None
        if dir_match:
            task = task[:dir_match.start()].strip().rstrip(',')
        result.append({'name': name, 'task': task.strip(), 'working_dir': working_dir})
    return result


def open_terminal_with_claude(name, task, working_dir=None):
    """Open a Terminal.app window running claude interactively."""
    escaped_task = task.replace('\\', '\\\\').replace('"', '\\"').replace("'", "'\"'\"'")

    cd_part = ""
    if working_dir:
        expanded = os.path.expanduser(working_dir)
        if os.path.isdir(expanded):
            cd_part = f"cd '{expanded}' && "

    # AppleScript to open Terminal with claude
    script = f'''
    tell application "Terminal"
        activate
        do script "{cd_part}echo 'Team {name}: {escaped_task}' && claude \\"{escaped_task}\\""
    end tell
    '''

    subprocess.run(['osascript', '-e', script], capture_output=True)


@bot.event
async def on_ready():
    print(f"Agentboard bot connected as {bot.user}")
    print("Ready! Send 'team Alpha: your task' in Discord.")


@bot.event
async def on_message(message):
    if message.author.bot:
        return

    content = message.content.strip()

    if content.startswith('!'):
        await bot.process_commands(message)
        return

    # Help
    if content.lower() in ('help', 'commands'):
        await message.reply(HELP_TEXT)
        return

    # Status
    if content.lower() == 'status':
        if not teams:
            await message.reply("No active teams.")
            return
        lines = []
        for name, info in teams.items():
            lines.append(f"**Team {name}** — {info['task'][:60]}")
            if info.get('working_dir'):
                lines.append(f"   📁 `{info['working_dir']}`")
        await message.reply('\n'.join(lines))
        return

    # Parse team assignments
    team_list = parse_team_assignments(content)

    if not team_list:
        return

    # Confirm and spawn
    lines = ["📋 **Spawning teams:**\n"]
    for t in team_list:
        lines.append(f"> **Team {t['name']}** — {t['task']}")
        if t.get('working_dir'):
            lines.append(f"> 📁 `{t['working_dir']}`")

    lines.append(f"\nOpening {len(team_list)} terminal(s) on your Mac...")
    await message.reply('\n'.join(lines))

    # Open terminal windows
    for t in team_list:
        teams[t['name']] = {
            'task': t['task'],
            'working_dir': t.get('working_dir'),
        }

        loop = asyncio.get_event_loop()
        await loop.run_in_executor(
            None,
            open_terminal_with_claude,
            t['name'],
            t['task'],
            t.get('working_dir'),
        )

    await message.channel.send(f"✅ **{len(team_list)} terminal(s) opened!** Check your Mac.")


bot.run(DISCORD_TOKEN)
