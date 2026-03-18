pub mod client;
pub mod prompt;

use crate::config::Config;
use client::{LlmClient, TriagePlan};

#[derive(Clone)]
pub struct Orchestrator {
    client: LlmClient,
}

impl Orchestrator {
    pub fn new(config: &Config) -> Self {
        let api_key = std::env::var(&config.orchestrator.api_key_env).unwrap_or_default();
        Self {
            client: LlmClient::new(
                &config.orchestrator.provider,
                &config.orchestrator.model,
                &api_key,
            ),
        }
    }

    pub async fn triage(
        &self,
        todo_list: &str,
    ) -> Result<TriagePlan, Box<dyn std::error::Error + Send + Sync>> {
        let user_prompt = prompt::build_triage_prompt(todo_list);
        self.client
            .triage(prompt::TRIAGE_SYSTEM_PROMPT, &user_prompt)
            .await
    }

    pub async fn summarize(
        &self,
        output: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let lines: Vec<&str> = output.lines().collect();
        let truncated = if lines.len() > 200 {
            lines[lines.len() - 200..].join("\n")
        } else {
            output.to_string()
        };
        let user_prompt = prompt::build_summary_prompt(&truncated);
        self.client
            .summarize(prompt::SUMMARY_SYSTEM_PROMPT, &user_prompt)
            .await
    }
}
