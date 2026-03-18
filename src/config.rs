use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub orchestrator: OrchestratorConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub backend: BackendConfig,
    #[serde(default)]
    pub backends: HashMap<String, BackendEntry>,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allowed_roots: Vec<String>,
    #[serde(default = "default_true")]
    pub require_plan_approval: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrchestratorConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LimitsConfig {
    #[serde(default = "default_max_teams")]
    pub max_concurrent_teams: usize,
    #[serde(default = "default_max_runtime")]
    pub max_runtime_seconds: u64,
    #[serde(default = "default_max_output")]
    pub max_output_lines: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    #[serde(default = "default_backend")]
    pub default: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendEntry {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default = "default_log_dir")]
    pub output_dir: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_port() -> u16 {
    8000
}
fn default_host() -> String {
    "0.0.0.0".into()
}
fn default_true() -> bool {
    true
}
fn default_provider() -> String {
    "anthropic".into()
}
fn default_model() -> String {
    "claude-sonnet-4-20250514".into()
}
fn default_api_key_env() -> String {
    "ANTHROPIC_API_KEY".into()
}
fn default_max_teams() -> usize {
    4
}
fn default_max_runtime() -> u64 {
    600
}
fn default_max_output() -> usize {
    10000
}
fn default_backend() -> String {
    "claude".into()
}
fn default_mode() -> String {
    "one-shot".into()
}
fn default_log_dir() -> String {
    "~/.agentboard/logs".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            orchestrator: OrchestratorConfig::default(),
            limits: LimitsConfig::default(),
            backend: BackendConfig::default(),
            backends: HashMap::new(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_roots: vec![],
            require_plan_approval: true,
        }
    }
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            api_key_env: default_api_key_env(),
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_teams: default_max_teams(),
            max_runtime_seconds: default_max_runtime(),
            max_output_lines: default_max_output(),
        }
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            default: default_backend(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            output_dir: default_log_dir(),
            enabled: true,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn validate_working_dir(&self, dir: &str) -> bool {
        if self.security.allowed_roots.is_empty() {
            return true;
        }
        let expanded = shellexpand::tilde(dir);
        self.security.allowed_roots.iter().any(|root| {
            let expanded_root = shellexpand::tilde(root);
            expanded.starts_with(expanded_root.as_ref())
        })
    }
}
