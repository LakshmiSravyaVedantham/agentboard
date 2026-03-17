use crate::agent::{BackendConfig, BackendMode};
use crate::config::Config;
use std::collections::HashMap;

pub struct BackendRegistry {
    backends: HashMap<String, BackendConfig>,
    pub default_backend: String,
}

impl BackendRegistry {
    pub fn from_config(config: &Config) -> Self {
        let mut backends = HashMap::new();
        for (name, entry) in &config.backends {
            let mode = match entry.mode.as_str() {
                "interactive" => BackendMode::Interactive,
                _ => BackendMode::OneShot,
            };
            backends.insert(name.clone(), BackendConfig {
                name: name.clone(),
                command: entry.command.clone(),
                args: entry.args.clone(),
                mode,
            });
        }
        Self { backends, default_backend: config.backend.default.clone() }
    }

    pub fn get(&self, name: &str) -> Option<&BackendConfig> { self.backends.get(name) }
    pub fn get_default(&self) -> Option<&BackendConfig> { self.backends.get(&self.default_backend) }
    pub fn list(&self) -> Vec<&str> { self.backends.keys().map(|k| k.as_str()).collect() }
}
