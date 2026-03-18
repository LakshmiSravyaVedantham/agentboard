use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_key: String,
    model: String,
    provider: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TeamAssignment {
    pub name: String,
    pub task: String,
    pub working_dir: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TriagePlan {
    pub teams: Vec<TeamAssignment>,
}

impl LlmClient {
    pub fn new(provider: &str, model: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            provider: provider.to_string(),
        }
    }

    pub async fn triage(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<TriagePlan, Box<dyn std::error::Error + Send + Sync>> {
        let response_text = self.call(system_prompt, user_prompt).await?;
        let plan: TriagePlan = serde_json::from_str(&response_text)?;
        Ok(plan)
    }

    pub async fn summarize(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.call(system_prompt, user_prompt).await
    }

    async fn call(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.provider.as_str() {
            "anthropic" => self.call_anthropic(system_prompt, user_prompt).await,
            "openai" => self.call_openai(system_prompt, user_prompt).await,
            _ => Err("Unsupported provider".into()),
        }
    }

    async fn call_anthropic(
        &self,
        system: &str,
        user: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "system": system,
            "messages": [{"role": "user", "content": user}]
        });
        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;
        let data: serde_json::Value = resp.json().await?;
        Ok(data["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    async fn call_openai(
        &self,
        system: &str,
        user: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });
        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;
        let data: serde_json::Value = resp.json().await?;
        Ok(data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }
}
