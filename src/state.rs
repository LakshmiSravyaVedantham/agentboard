use crate::agent::runner::AgentProcess;
use crate::agent::team::Team;
use crate::agent::registry::BackendRegistry;
use crate::config::Config;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::client::TriagePlan;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};
use uuid::Uuid;

pub type TeamStore = Arc<RwLock<HashMap<Uuid, Team>>>;
pub type ProcessStore = Arc<RwLock<HashMap<Uuid, AgentProcess>>>;

pub struct AppState {
    pub config: Config,
    pub registry: BackendRegistry,
    pub orchestrator: Orchestrator,
    pub teams: TeamStore,
    pub processes: ProcessStore,
    pub pending_plans: Arc<RwLock<HashMap<String, TriagePlan>>>,
    pub concurrency_semaphore: Arc<Semaphore>,
    pub pairing_code: String,
    pub jwt_secret: String,
    pub ws_broadcast: broadcast::Sender<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
}
