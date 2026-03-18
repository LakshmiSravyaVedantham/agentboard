pub const TRIAGE_SYSTEM_PROMPT: &str = r#"You are an AI project manager. Given a todo list, break it into independent teams.

Rules:
- Each team gets ONE focused task
- Name teams with Greek letters: Alpha, Beta, Gamma, Delta, Epsilon, etc.
- If a task mentions a specific project, set working_dir to that project's likely path
- If unsure about working_dir, set it to null
- Return ONLY valid JSON, no markdown

Output format:
{
  "teams": [
    {"name": "Alpha", "task": "description of task", "working_dir": "~/path/to/project or null"}
  ]
}
"#;

pub const SUMMARY_SYSTEM_PROMPT: &str = r#"You are a concise technical writer. Given terminal output from an AI coding agent, write a 2-3 sentence summary of what was accomplished. Include: what was built/fixed, key files changed, and any issues. Be specific, not generic."#;

pub fn build_triage_prompt(todo_list: &str) -> String {
    format!("Break this todo list into teams:\n\n{}", todo_list)
}

pub fn build_summary_prompt(output: &str) -> String {
    format!("Summarize what this agent accomplished:\n\n{}", output)
}
