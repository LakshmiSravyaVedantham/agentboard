use agentboard::config::Config;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_config_from_file() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, r#"
[server]
port = 9000
host = "127.0.0.1"

[security]
allowed_roots = ["~/projects"]
require_plan_approval = true

[orchestrator]
provider = "anthropic"
model = "claude-sonnet-4-6"
api_key_env = "ANTHROPIC_API_KEY"

[limits]
max_concurrent_teams = 2
max_runtime_seconds = 300
max_output_lines = 5000

[backend]
default = "claude"

[backends.claude]
command = "claude"
args = ["--print", "{{task}}"]
mode = "one-shot"
description = "Claude Code CLI"
"#).unwrap();

    let config = Config::load(f.path()).unwrap();
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.limits.max_concurrent_teams, 2);
    assert_eq!(config.backend.default, "claude");
}

#[test]
fn test_config_defaults() {
    let config = Config::default();
    assert_eq!(config.server.port, 8000);
    assert_eq!(config.limits.max_concurrent_teams, 4);
    assert_eq!(config.security.require_plan_approval, true);
}

#[test]
fn test_validate_working_dir() {
    let mut config = Config::default();
    config.security.allowed_roots = vec!["~/projects".into()];
    assert!(config.validate_working_dir("~/projects/myapp"));
    assert!(!config.validate_working_dir("~/Documents/secrets"));
}

#[test]
fn test_validate_working_dir_empty_roots() {
    let config = Config::default();
    // Empty allowed_roots means no restrictions
    assert!(config.validate_working_dir("~/anything"));
}
