import os
import sys
import asyncio
import re
import subprocess
import tempfile
import traceback
import discord
from discord.ext import commands
from dotenv import load_dotenv
from pathlib import Path

load_dotenv(Path(__file__).parent.parent / '.env')
DISCORD_TOKEN = os.getenv('DISCORD_BOT_TOKEN')
if not DISCORD_TOKEN:
    print("ERROR: DISCORD_BOT_TOKEN not set in .env")
    sys.exit(1)

# Backends: separate terminal_cmd (interactive) from bg_cmd (for Discord output)
BACKENDS = {
    '1': {
        'name': 'Claude',
        'terminal_cmd': 'claude --dangerously-skip-permissions --name "Team-{name}" "{task}"',
        'bg_cmd': None,
    },
    '2': {
        'name': 'Codex',
        'terminal_cmd': 'codex "{task}"',
        'bg_cmd': ['codex', '{task}'],
    },
    '3': {
        'name': 'Aider',
        'terminal_cmd': 'aider --message "{task}"',
        'bg_cmd': ['aider', '--message', '{task}'],
    },
    '4': {
        'name': 'Gemini',
        'terminal_cmd': 'gemini "{task}"',
        'bg_cmd': ['gemini', '{task}'],
    },
    '5': {
        'name': 'Goose',
        'terminal_cmd': 'goose run "{task}"',
        'bg_cmd': ['goose', 'run', '{task}'],
    },
    '6': {
        'name': 'OpenCode',
        'terminal_cmd': 'opencode "{task}"',
        'bg_cmd': ['opencode', '{task}'],
    },
}

intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix='!', intents=intents)

teams = {}
pending_backend = {}

HELP_TEXT = """**Agentboard Bot**

**Assign tasks:**
```
team Alpha: Build a landing page
team Beta: Fix auth bug in ~/synaptiq
```

**Commands:** `status`, `kill Alpha`, `help`
"""

BACKEND_MENU = """**Which terminal?**
> 1️⃣ Claude  2️⃣ Codex  3️⃣ Aider
> 4️⃣ Gemini  5️⃣ Goose  6️⃣ OpenCode

React to choose."""


def parse_teams(content):
    matches = re.findall(r'team\s+(\w+)\s*:\s*(.+)', content, re.IGNORECASE | re.MULTILINE)
    result = []
    for name, task in matches:
        dm = re.search(r'\b(?:in|dir|at)\s+(~/\S+)', task)
        wd = dm.group(1) if dm else None
        if dm:
            task = task[:dm.start()].strip().rstrip(',')
        result.append({'name': name, 'task': task.strip(), 'working_dir': wd})
    return result


def open_in_terminal(name, task, backend, working_dir=None):
    """Open Terminal.app with the command via a temp script."""
    cd_part = ""
    if working_dir:
        expanded = os.path.expanduser(working_dir)
        if os.path.isdir(expanded):
            cd_part = f"cd '{expanded}'\n"

    # Build command for shell script
    full_cmd = backend['terminal_cmd'].replace('{task}', task).replace('{name}', name)

    # Write shell script to avoid AppleScript quoting issues
    tmp = tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False, prefix=f'agentboard-{name}-')
    tmp.write(f"#!/bin/bash\n{cd_part}{full_cmd}\n")
    tmp.close()
    os.chmod(tmp.name, 0o755)

    # Simple AppleScript — just runs the script file
    script = f'''tell application "Terminal"
    activate
    do script "{tmp.name}"
end tell'''

    r = subprocess.run(['osascript', '-e', script], capture_output=True, text=True)
    if r.returncode != 0:
        print(f"Terminal error: {r.stderr}", flush=True)
    else:
        print(f"Terminal opened: Team {name}", flush=True)


@bot.event
async def on_ready():
    print(f"Bot connected as {bot.user}", flush=True)


@bot.event
async def on_message(message):
    if message.author.bot:
        return
    content = message.content.strip()

    if content.lower() in ('help', 'commands'):
        await message.reply(HELP_TEXT)
        return

    if content.lower() == 'clean':
        guild = message.guild
        deleted = 0
        for ch in guild.text_channels:
            if ch.name.startswith('team-'):
                await ch.delete()
                deleted += 1
        teams.clear()
        # Clear messages in current channel
        try:
            await message.channel.purge(limit=100)
        except Exception:
            pass
        await message.channel.send(f"🧹 Cleaned up {deleted} team channel(s) and cleared messages.")
        return

    if content.lower() == 'status':
        if not teams:
            await message.reply("No active teams.")
            return
        lines = [f"**{n}** ({i.get('backend','?')}) — {i['task'][:50]}" for n, i in teams.items()]
        await message.reply('\n'.join(lines))
        return

    if content.lower().startswith('kill '):
        n = content[5:].strip()
        if n in teams and teams[n].get('process'):
            teams[n]['process'].kill()
            await message.reply(f"💀 {n} killed.")
        else:
            await message.reply(f"'{n}' not found.")
        return

    team_list = parse_teams(content)
    if not team_list:
        return

    lines = ["📋 **Teams:**"]
    for t in team_list:
        lines.append(f"> **{t['name']}** — {t['task']}")
    lines.append("")
    lines.append(BACKEND_MENU)

    msg = await message.reply('\n'.join(lines))
    for e in ['1️⃣', '2️⃣', '3️⃣', '4️⃣', '5️⃣', '6️⃣']:
        await msg.add_reaction(e)
    pending_backend[msg.id] = {'teams': team_list, 'channel': message.channel, 'author': message.author.id}


@bot.event
async def on_reaction_add(reaction, user):
    if user.bot:
        return
    mid = reaction.message.id
    if mid not in pending_backend:
        return
    info = pending_backend[mid]
    if user.id != info['author']:
        return

    emap = {'1️⃣':'1','2️⃣':'2','3️⃣':'3','4️⃣':'4','5️⃣':'5','6️⃣':'6'}
    choice = emap.get(str(reaction.emoji))
    if not choice or choice not in BACKENDS:
        return

    del pending_backend[mid]
    backend = BACKENDS[choice]
    team_list = info['teams']
    channel = info['channel']
    guild = channel.guild

    try:
        await channel.send(f"✅ **{backend['name']}** selected. Spawning {len(team_list)} team(s)...")

        for t in team_list:
            n, task, wd = t['name'], t['task'], t.get('working_dir')

            # Create Discord channel
            ch_name = f"team-{n.lower()}"
            team_ch = discord.utils.get(guild.text_channels, name=ch_name)
            if not team_ch:
                team_ch = await guild.create_text_channel(ch_name)
                print(f"Created #{ch_name}", flush=True)

            teams[n] = {'task': task, 'working_dir': wd, 'backend': backend['name'], 'channel': team_ch, 'process': None}
            await team_ch.send(f"**Team {n}** ({backend['name']})\n📋 {task}")

            # Open Terminal
            loop = asyncio.get_event_loop()
            await loop.run_in_executor(None, open_in_terminal, n, task, backend, wd)

            # Stream to Discord in background (only if backend supports it)
            if backend.get('bg_cmd'):
                asyncio.create_task(stream_output(n, task, backend, team_ch, wd))

        await channel.send(f"🖥️ **{len(team_list)} terminal(s) opened!** Output streams to team channels.")

    except Exception as e:
        print(f"ERROR: {e}", flush=True)
        traceback.print_exc()
        await channel.send(f"❌ Error: {e}")


async def stream_output(name, task, backend, team_ch, working_dir=None):
    """Run CLI in background, stream output to Discord channel."""
    cmd_parts = [a.replace('{task}', task).replace('{name}', name) for a in backend['bg_cmd']]

    kwargs = {}
    if working_dir:
        expanded = os.path.expanduser(working_dir)
        if os.path.isdir(expanded):
            kwargs['cwd'] = expanded

    env = {k: v for k, v in os.environ.items() if k != 'ANTHROPIC_API_KEY'}

    try:
        proc = await asyncio.create_subprocess_exec(
            *cmd_parts,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            env=env,
            **kwargs
        )
        teams[name]['process'] = proc

        buf = []
        last = asyncio.get_event_loop().time()

        async def flush():
            nonlocal buf, last
            if buf:
                text = '\n'.join(buf)
                while text:
                    await team_ch.send(f"```\n{text[:1800]}\n```")
                    text = text[1800:]
                buf = []
                last = asyncio.get_event_loop().time()

        try:
            async with asyncio.timeout(600):
                while True:
                    line = await proc.stdout.readline()
                    if not line:
                        break
                    buf.append(line.decode().rstrip())
                    now = asyncio.get_event_loop().time()
                    if now - last >= 5 or len(buf) >= 15:
                        await flush()
        except asyncio.TimeoutError:
            proc.kill()
            await team_ch.send("⏰ Timed out")
            return

        await flush()
        await proc.wait()
        status = "✅ Done!" if proc.returncode == 0 else f"❌ Failed (exit {proc.returncode})"
        await team_ch.send(f"**Team {name}** {status}")

    except FileNotFoundError:
        await team_ch.send(f"❌ `{cmd_parts[0]}` not found")
    except Exception as e:
        await team_ch.send(f"❌ {e}")


bot.run(DISCORD_TOKEN)
