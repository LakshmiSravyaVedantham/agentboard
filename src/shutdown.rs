use crate::agent::team::TeamStatus;
use crate::state::AppState;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

pub async fn graceful_shutdown(state: Arc<AppState>) {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl+c");
    tracing::info!("Shutting down...");

    let mut processes = state.processes.write().await;
    let mut teams = state.teams.write().await;

    for (id, process) in processes.iter_mut() {
        if let Err(e) = process.child.start_kill() {
            tracing::warn!("Failed to send SIGTERM to {}: {}", id, e);
            continue;
        }
        match timeout(Duration::from_secs(5), process.child.wait()).await {
            Ok(_) => tracing::info!("Team {} exited gracefully", id),
            Err(_) => {
                let _ = process.child.kill().await;
                tracing::warn!("Team {} force-killed", id);
            }
        }
        if let Some(team) = teams.get_mut(id) {
            let _ = team.transition_to(TeamStatus::Killed);
        }
    }
}
