use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TeamStatus {
    Queued, Running, Done, Failed, Killed,
}

#[derive(Debug, Clone, Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub task: String,
    pub status: TeamStatus,
    pub backend: String,
    pub working_dir: Option<PathBuf>,
    pub output: Vec<String>,
    pub summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl Team {
    fn sanitize_name(name: &str) -> String {
        name.chars().filter(|c| c.is_alphanumeric() || *c == '-').collect()
    }

    pub fn new(name: String, task: String, backend: String, working_dir: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: Self::sanitize_name(&name),
            task, status: TeamStatus::Queued, backend,
            working_dir: working_dir.map(|d| PathBuf::from(shellexpand::tilde(&d).into_owned())),
            output: Vec::new(), summary: None,
            created_at: Utc::now(), finished_at: None,
        }
    }

    pub fn transition_to(&mut self, new_status: TeamStatus) -> Result<(), String> {
        let valid = match (&self.status, &new_status) {
            (TeamStatus::Queued, TeamStatus::Running) => true,
            (TeamStatus::Running, TeamStatus::Done) => true,
            (TeamStatus::Running, TeamStatus::Failed) => true,
            (TeamStatus::Running, TeamStatus::Killed) => true,
            (TeamStatus::Queued, TeamStatus::Killed) => true,
            _ => false,
        };
        if valid {
            if matches!(new_status, TeamStatus::Done | TeamStatus::Failed | TeamStatus::Killed) {
                self.finished_at = Some(Utc::now());
            }
            self.status = new_status;
            Ok(())
        } else {
            Err(format!("Invalid transition from {:?} to {:?}", self.status, new_status))
        }
    }

    pub fn append_output(&mut self, line: &str) {
        self.output.push(line.to_string());
    }
}
