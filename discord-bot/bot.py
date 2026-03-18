import os
import sys
import asyncio
import json
import discord
from discord.ext import commands
from dotenv import load_dotenv
import anthropic
from pathlib import Path

# Load env from parent directory
load_dotenv(Path(__file__).parent.parent / '.env')

DISCORD_TOKEN = os.getenv('DISCORD_BOT_TOKEN')
ANTHROPIC_KEY = os.getenv('ANTHROPIC_API_KEY')

if not DISCORD_TOKEN:
    print("ERROR: DISCORD_BOT_TOKEN not set in .env")
    sys.exit(1)
if not ANTHROPIC_KEY:
    print("ERROR: ANTHROPIC_API_KEY not set in .env")
    sys.exit(1)

# Config
MAX_CONCURRENT_TEAMS = 4
MAX_RUNTIME_SECONDS = 600
DEFAULT_BACKEND_CMD = "claude"
DEFAULT_BACKEND_ARGS = ["--print", "{task}"]

# Try to load backend config from agentboard.toml
try:
    import tomllib
    toml_path = Path(__file__).parent.parent / 'agentboard.toml'
    if toml_path.exists():
        with open(toml_path, 'rb') as f:
            config = tomllib.load(f)
        MAX_CONCURRENT_TEAMS = config.get('limits', {}).get('max_concurrent_teams', 4)
        MAX_RUNTIME_SECONDS = config.get('limits', {}).get('max_runtime_seconds', 600)
        default_backend = config.get('backend', {}).get('default', 'claude')
        backends = config.get('backends', {})
        if default_backend in backends:
            DEFAULT_BACKEND_CMD = backends[default_backend].get('command', DEFAULT_BACKEND_CMD)
            DEFAULT_BACKEND_ARGS = backends[default_backend].get('args', DEFAULT_BACKEND_ARGS)
except Exception:
    pass

TRIAGE_SYSTEM_PROMPT = """You are an AI project manager. Given a todo list, break it into independent teams.

Rules:
- Each team gets ONE focused task
- Name teams with Greek letters: Alpha, Beta, Gamma, Delta, Epsilon, etc.
- If a task mentions a specific project, set working_dir to that project's likely path
- If unsure about working_dir, set it to null
- Return ONLY valid JSON, no markdown

Output format:
{"teams": [{"name": "Alpha", "task": "description", "working_dir": "~/path or null"}]}"""

SUMMARY_SYSTEM_PROMPT = """You are a concise technical writer. Given terminal output from an AI coding agent, write a 2-3 sentence summary. Include: what was built/fixed, key files changed, any issues. Be specific."""

# Anthropic client
client = anthropic.Anthropic(api_key=ANTHROPIC_KEY)

# Bot setup
intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix='!', intents=intents)

# Team tracking
teams = {}  # {team_name: {task, status, working_dir, process, thread, output}}
semaphore = asyncio.Semaphore(MAX_CONCURRENT_TEAMS)
pending_plans = {}  # {message_id: plan}


def call_llm(system_prompt: str, user_prompt: str) -> str:
    """Call Anthropic API synchronously."""
    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=1024,
        system=system_prompt,
        messages=[{"role": "user", "content": user_prompt}]
    )
    text = response.content[0].text
    # Strip markdown code blocks if present
    text = text.strip()
    if text.startswith("```"):
        lines = text.split('\n')
        text = '\n'.join(lines[1:])  # remove opening ```json
        text = text.rstrip('`').strip()
    return text


async def triage_tasks(todo_list: str) -> dict:
    """Call LLM to break todo list into teams."""
    loop = asyncio.get_event_loop()
    result = await loop.run_in_executor(
        None, call_llm, TRIAGE_SYSTEM_PROMPT, f"Break this todo list into teams:\n\n{todo_list}"
    )
    return json.loads(result)


async def summarize_output(output: str) -> str:
    """Call LLM to summarize agent output."""
    # Truncate to last 200 lines
    lines = output.strip().split('\n')
    if len(lines) > 200:
        lines = lines[-200:]
    truncated = '\n'.join(lines)
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(
        None, call_llm, SUMMARY_SYSTEM_PROMPT, f"Summarize:\n\n{truncated}"
    )


async def spawn_team(team_info: dict, channel: discord.TextChannel) -> None:
    """Spawn a CLI subprocess for a team and stream output to a Discord thread."""
    name = team_info['name']
    task = team_info['task']
    working_dir = team_info.get('working_dir')

    async with semaphore:
        # Create thread for this team
        thread = await channel.create_thread(
            name=f"Team {name} — {task[:50]}",
            type=discord.ChannelType.public_thread
        )

        teams[name] = {
            'task': task,
            'status': 'Running',
            'working_dir': working_dir,
            'thread': thread,
            'output': [],
            'process': None,
        }

        await thread.send(
            f"**Team {name}** starting...\n"
            f"Task: {task}\n"
            f"Dir: `{working_dir or 'current'}`"
        )

        # Build command args, substituting {task} placeholder
        args = [a.replace('{task}', task) for a in DEFAULT_BACKEND_ARGS]

        try:
            kwargs: dict = {}
            if working_dir:
                expanded = os.path.expanduser(working_dir)
                if os.path.isdir(expanded):
                    kwargs['cwd'] = expanded

            process = await asyncio.create_subprocess_exec(
                DEFAULT_BACKEND_CMD, *args,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                **kwargs
            )
            teams[name]['process'] = process

            # Stream output with rate-limiting
            buffer: list[str] = []
            last_post_time = asyncio.get_event_loop().time()

            async def post_buffer() -> None:
                nonlocal buffer, last_post_time
                if buffer:
                    text = '\n'.join(buffer)
                    # Discord 2000 char limit — split if needed
                    while text:
                        chunk = text[:1900]
                        text = text[1900:]
                        await thread.send(f"```\n{chunk}\n```")
                    buffer = []
                    last_post_time = asyncio.get_event_loop().time()

            try:
                async with asyncio.timeout(MAX_RUNTIME_SECONDS):
                    while True:
                        line = await process.stdout.readline()
                        if not line:
                            break
                        decoded = line.decode().rstrip()
                        teams[name]['output'].append(decoded)
                        buffer.append(decoded)

                        # Post every 5 seconds or every 20 lines
                        now = asyncio.get_event_loop().time()
                        if now - last_post_time >= 5 or len(buffer) >= 20:
                            await post_buffer()
            except asyncio.TimeoutError:
                process.kill()
                await thread.send(
                    f"Timed out — killed after {MAX_RUNTIME_SECONDS // 60} minutes"
                )
                teams[name]['status'] = 'Failed'
                return

            # Flush remaining buffer
            await post_buffer()
            await process.wait()

            if process.returncode == 0:
                teams[name]['status'] = 'Done'
                full_output = '\n'.join(teams[name]['output'])
                try:
                    summary = await summarize_output(full_output)
                except Exception:
                    summary = "Completed successfully."
                teams[name]['summary'] = summary
                await thread.send(f"**Team {name} done!**\n\n{summary}")
            else:
                teams[name]['status'] = 'Failed'
                await thread.send(
                    f"**Team {name} failed** (exit code {process.returncode})"
                )

        except FileNotFoundError:
            teams[name]['status'] = 'Failed'
            await thread.send(
                f"Error: Command `{DEFAULT_BACKEND_CMD}` not found. Is it installed?"
            )
        except Exception as e:
            teams[name]['status'] = 'Failed'
            await thread.send(f"Error: {str(e)}")


@bot.event
async def on_ready() -> None:
    print(f"Agentboard bot connected as {bot.user}")
    print(f"Backend: {DEFAULT_BACKEND_CMD} {' '.join(DEFAULT_BACKEND_ARGS)}")
    print(f"Max teams: {MAX_CONCURRENT_TEAMS}, Timeout: {MAX_RUNTIME_SECONDS}s")


@bot.event
async def on_message(message: discord.Message) -> None:
    if message.author.bot:
        return

    content = message.content.strip()

    # Let the command framework handle ! prefixed messages
    if content.startswith('!'):
        await bot.process_commands(message)
        return

    # Status command
    if content.lower() == 'status':
        if not teams:
            await message.reply("No active teams.")
            return
        lines = []
        for name, info in teams.items():
            status_emoji = {
                'Running': 'Running',
                'Done': 'Done',
                'Failed': 'Failed',
                'Killed': 'Killed',
            }.get(info['status'], info['status'])
            lines.append(f"**Team {name}** [{status_emoji}] — {info['task'][:60]}")
            if info.get('summary'):
                lines.append(f"   _{info['summary'][:100]}_")
        await message.reply('\n'.join(lines))
        return

    # Kill command
    if content.lower().startswith('kill '):
        team_name = content[5:].strip()
        if team_name in teams and teams[team_name].get('process'):
            teams[team_name]['process'].kill()
            teams[team_name]['status'] = 'Killed'
            await message.reply(f"Team {team_name} killed.")
        else:
            await message.reply(f"Team '{team_name}' not found or not running.")
        return

    # Ignore very short messages
    if len(content) < 5:
        return

    # Treat everything else as a todo list
    await message.add_reaction('🤔')

    try:
        plan = await triage_tasks(content)
    except Exception as e:
        await message.remove_reaction('🤔', bot.user)
        await message.reply(f"Failed to create plan: {str(e)}")
        return

    team_list = plan.get('teams', [])
    if not team_list:
        await message.remove_reaction('🤔', bot.user)
        await message.reply("Couldn't break that into tasks. Try being more specific.")
        return

    # Format plan message
    lines = ["**Plan Ready**\n"]
    for t in team_list:
        lines.append(f"> **Team {t['name']}** — {t['task']}")
        if t.get('working_dir'):
            lines.append(f"> `{t['working_dir']}`")
        lines.append(">")
    lines.append("\nReact ✅ to approve or ❌ to cancel")

    plan_msg = await message.reply('\n'.join(lines))
    await plan_msg.add_reaction('✅')
    await plan_msg.add_reaction('❌')

    # Store pending plan keyed by the plan message id
    pending_plans[plan_msg.id] = {
        'plan': team_list,
        'channel': message.channel,
        'author': message.author.id,
    }

    await message.remove_reaction('🤔', bot.user)


@bot.event
async def on_reaction_add(reaction: discord.Reaction, user: discord.User) -> None:
    if user.bot:
        return

    msg_id = reaction.message.id
    if msg_id not in pending_plans:
        return

    plan_info = pending_plans[msg_id]
    # Only the original requester can approve/cancel
    if user.id != plan_info['author']:
        return

    if str(reaction.emoji) == '✅':
        del pending_plans[msg_id]
        channel = plan_info['channel']
        team_list = plan_info['plan']

        await channel.send(f"Approved! Spawning {len(team_list)} teams...")

        # Spawn all teams concurrently and track completion
        tasks = [spawn_team(t, channel) for t in team_list]
        asyncio.gather(*tasks)
        asyncio.create_task(wait_and_summarize(team_list, channel))

    elif str(reaction.emoji) == '❌':
        del pending_plans[msg_id]
        await reaction.message.reply("Plan cancelled.")


async def wait_and_summarize(team_list: list, channel: discord.TextChannel) -> None:
    """Poll until all teams finish, then post a consolidated summary."""
    names = [t['name'] for t in team_list]

    while True:
        await asyncio.sleep(5)
        all_done = all(
            teams.get(n, {}).get('status') in ('Done', 'Failed', 'Killed')
            for n in names
        )
        if all_done:
            break

    lines = ["**All teams finished!**\n"]
    for n in names:
        info = teams.get(n, {})
        status = info.get('status', 'Unknown')
        lines.append(f"**Team {n}** [{status}] — {info.get('task', '')[:60]}")
        if info.get('summary'):
            lines.append(f"> {info['summary']}")

    await channel.send('\n'.join(lines))


bot.run(DISCORD_TOKEN)
