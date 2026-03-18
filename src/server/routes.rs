use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use crate::agent::runner::spawn_agent;
use crate::agent::team::{Team, TeamStatus};
use crate::auth::AuthUser;
use crate::state::AppState;

// ── Public routes ────────────────────────────────────────────────────────────

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let teams = state.teams.read().await;
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": (chrono::Utc::now() - state.start_time).num_seconds(),
        "active_teams": teams.len(),
    }))
}

pub async fn pair(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let code = body["code"].as_str().unwrap_or("");
    if code == state.pairing_code {
        let token = crate::auth::create_token(&state.jwt_secret).unwrap();
        (StatusCode::OK, Json(json!({ "token": token })))
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid pairing code" })),
        )
    }
}

// ── Protected routes ─────────────────────────────────────────────────────────

pub async fn list_teams(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let teams = state.teams.read().await;
    let list: Vec<_> = teams.values().collect();
    Json(json!({ "teams": list }))
}

/// POST /api/tasks  {"input": "todo list text"}
pub async fn submit_tasks(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let input = match body["input"].as_str() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'input' field" })),
            )
                .into_response();
        }
    };

    let plan = match state.orchestrator.triage(&input).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Orchestrator triage failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Orchestrator error", "detail": e.to_string() })),
            )
                .into_response();
        }
    };

    // Validate all working_dirs up-front before storing the plan.
    for team in &plan.teams {
        if let Some(dir) = &team.working_dir {
            if !state.config.validate_working_dir(dir) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "working_dir not in allowed roots",
                        "path": dir
                    })),
                )
                    .into_response();
            }
        }
    }

    let plan_id = Uuid::new_v4().to_string();

    {
        let mut plans = state.pending_plans.write().await;
        plans.insert(plan_id.clone(), plan.clone());
    }

    let teams_json: Vec<serde_json::Value> = plan
        .teams
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "task": t.task,
                "working_dir": t.working_dir,
            })
        })
        .collect();

    let event = json!({
        "v": 1,
        "type": "orchestrator_plan",
        "plan_id": plan_id,
        "teams": teams_json,
    });
    let _ = state.ws_broadcast.send(event.to_string());

    (
        StatusCode::OK,
        Json(json!({
            "plan_id": plan_id,
            "teams": teams_json,
        })),
    )
        .into_response()
}

/// POST /api/plan/approve  {"plan_id": "..."}
pub async fn approve_plan(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let plan_id = match body["plan_id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'plan_id' field" })),
            )
                .into_response();
        }
    };

    let plan = {
        let mut plans = state.pending_plans.write().await;
        match plans.remove(&plan_id) {
            Some(p) => p,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Plan not found" })),
                )
                    .into_response();
            }
        }
    };

    let mut created_teams: Vec<serde_json::Value> = Vec::new();

    for assignment in plan.teams {
        let backend_name = state.registry.default_backend.clone();
        let backend = match state.registry.get(&backend_name) {
            Some(b) => b.clone(),
            None => {
                tracing::warn!("No backend '{}' found, skipping team", backend_name);
                continue;
            }
        };

        let team = Team::new(
            assignment.name.clone(),
            assignment.task.clone(),
            backend_name.clone(),
            assignment.working_dir.clone(),
        );
        let team_id = team.id;

        {
            let mut teams = state.teams.write().await;
            teams.insert(team_id, team.clone());
        }

        let team_json = json!({
            "id": team_id,
            "name": team.name,
            "task": team.task,
            "status": "Running",
        });
        created_teams.push(team_json.clone());

        let event = json!({
            "v": 1,
            "type": "team_created",
            "team": team_json,
        });
        let _ = state.ws_broadcast.send(event.to_string());

        let log_dir = shellexpand::tilde(&state.config.logging.output_dir).into_owned();
        let log_file = if state.config.logging.enabled {
            Some(PathBuf::from(format!("{}/{}.log", log_dir, team_id)))
        } else {
            None
        };

        let working_dir = team.working_dir.clone();
        let max_output = state.config.limits.max_output_lines;
        let semaphore = state.concurrency_semaphore.clone();
        let processes_store = state.processes.clone();
        let teams_store = state.teams.clone();
        let ws_tx = state.ws_broadcast.clone();
        let orchestrator = state.orchestrator.clone();

        tokio::spawn(async move {
            // Acquire the concurrency permit before spawning the subprocess.
            let _permit = semaphore.acquire().await.unwrap();

            let mut process = match spawn_agent(&backend, &assignment.task, working_dir.as_ref(), max_output, log_file).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to spawn agent for team {}: {}", team_id, e);
                    let mut teams = teams_store.write().await;
                    if let Some(t) = teams.get_mut(&team_id) {
                        let _ = t.transition_to(TeamStatus::Failed);
                    }
                    let event = json!({
                        "v": 1,
                        "type": "team_error",
                        "team_id": team_id,
                        "error": e.to_string(),
                    });
                    let _ = ws_tx.send(event.to_string());
                    return;
                }
            };

            // Transition team to Running.
            {
                let mut teams = teams_store.write().await;
                if let Some(t) = teams.get_mut(&team_id) {
                    let _ = t.transition_to(TeamStatus::Running);
                }
            }

            // Subscribe to output lines and forward to WebSocket + team store.
            let mut output_rx = process.output_tx.subscribe();
            let teams_for_output = teams_store.clone();
            let ws_for_output = ws_tx.clone();
            let output_task = tokio::spawn(async move {
                while let Ok(line) = output_rx.recv().await {
                    {
                        let mut teams = teams_for_output.write().await;
                        if let Some(t) = teams.get_mut(&team_id) {
                            t.append_output(&line);
                        }
                    }
                    let event = json!({
                        "v": 1,
                        "type": "team_output",
                        "team_id": team_id,
                        "line": line,
                    });
                    let _ = ws_for_output.send(event.to_string());
                }
            });

            // Wait for process to finish.
            let success = match process.child.wait().await {
                Ok(status) => status.success(),
                Err(e) => {
                    tracing::error!("Error waiting on team {} process: {}", team_id, e);
                    false
                }
            };

            // Stop forwarding output.
            output_task.abort();
            let _ = output_task.await;

            // Store process handle (for kill_team) — store it now that it has exited.
            {
                let mut procs = processes_store.write().await;
                procs.insert(team_id, process);
            }

            // Generate summary from collected output.
            let output_text = {
                let teams = teams_store.read().await;
                teams
                    .get(&team_id)
                    .map(|t| t.output.join("\n"))
                    .unwrap_or_default()
            };

            let summary = orchestrator
                .summarize(&output_text)
                .await
                .unwrap_or_else(|_| "Agent completed.".to_string());

            // Transition to final state and store summary.
            let new_status = if success { TeamStatus::Done } else { TeamStatus::Failed };
            {
                let mut teams = teams_store.write().await;
                if let Some(t) = teams.get_mut(&team_id) {
                    let _ = t.transition_to(new_status.clone());
                    t.summary = Some(summary.clone());
                }
            }

            if matches!(new_status, TeamStatus::Done) {
                let event = json!({
                    "v": 1,
                    "type": "team_done",
                    "team_id": team_id,
                    "summary": summary,
                });
                let _ = ws_tx.send(event.to_string());
            } else {
                let event = json!({
                    "v": 1,
                    "type": "team_error",
                    "team_id": team_id,
                    "error": "Agent process exited with non-zero status",
                });
                let _ = ws_tx.send(event.to_string());
            }
        });
    }

    (
        StatusCode::OK,
        Json(json!({ "teams": created_teams })),
    )
        .into_response()
}

/// POST /api/plan/edit  {"plan_id": "...", "teams": [...]}
pub async fn edit_plan(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let plan_id = match body["plan_id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'plan_id' field" })),
            )
                .into_response();
        }
    };

    let teams_val = match body["teams"].as_array() {
        Some(t) => t.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing or invalid 'teams' field" })),
            )
                .into_response();
        }
    };

    // Validate working_dirs in the proposed edit.
    for team in &teams_val {
        if let Some(dir) = team["working_dir"].as_str() {
            if !state.config.validate_working_dir(dir) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "working_dir not in allowed roots",
                        "path": dir
                    })),
                )
                    .into_response();
            }
        }
    }

    let mut plans = state.pending_plans.write().await;
    match plans.get_mut(&plan_id) {
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Plan not found" })),
        )
            .into_response(),
        Some(plan) => {
            // Rebuild teams from the provided JSON array.
            let new_teams: Vec<crate::orchestrator::client::TeamAssignment> = teams_val
                .iter()
                .filter_map(|t| {
                    let name = t["name"].as_str()?.to_string();
                    let task = t["task"].as_str()?.to_string();
                    let working_dir = t["working_dir"].as_str().map(|s| s.to_string());
                    Some(crate::orchestrator::client::TeamAssignment { name, task, working_dir })
                })
                .collect();
            plan.teams = new_teams;
            (StatusCode::OK, Json(json!({ "plan_id": plan_id, "teams": teams_val }))).into_response()
        }
    }
}

/// POST /api/teams  (not used in current flow but kept for completeness)
pub async fn create_team(
    _auth: AuthUser,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "error": "Use /api/tasks + /api/plan/approve" })))
}

/// GET /api/teams/:id
pub async fn get_team(
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let teams = state.teams.read().await;
    match teams.get(&id) {
        Some(team) => (StatusCode::OK, Json(json!(team))).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "Team not found" }))).into_response(),
    }
}

/// POST /api/teams/:id/message  {"message": "..."}
pub async fn send_message(
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let message = match body["message"].as_str() {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'message' field" })),
            )
                .into_response();
        }
    };

    // Look up the team to get its working_dir and backend.
    let (working_dir, backend_name, _task) = {
        let teams = state.teams.read().await;
        match teams.get(&id) {
            None => {
                return (StatusCode::NOT_FOUND, Json(json!({ "error": "Team not found" }))).into_response();
            }
            Some(team) => (
                team.working_dir.clone(),
                team.backend.clone(),
                team.task.clone(),
            ),
        }
    };

    let backend = match state.registry.get(&backend_name) {
        Some(b) => b.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Backend not found" })),
            )
                .into_response();
        }
    };

    let log_dir = shellexpand::tilde(&state.config.logging.output_dir).into_owned();
    let log_file = if state.config.logging.enabled {
        Some(PathBuf::from(format!("{}/{}_followup.log", log_dir, id)))
    } else {
        None
    };

    let max_output = state.config.limits.max_output_lines;
    let teams_store = state.teams.clone();
    let ws_tx = state.ws_broadcast.clone();
    let team_id = id;

    // For one-shot backends: spawn a new process with the follow-up message.
    tokio::spawn(async move {
        let mut process = match spawn_agent(&backend, &message, working_dir.as_ref(), max_output, log_file).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Failed to spawn follow-up agent for team {}: {}", team_id, e);
                let event = json!({
                    "v": 1,
                    "type": "team_error",
                    "team_id": team_id,
                    "error": e.to_string(),
                });
                let _ = ws_tx.send(event.to_string());
                return;
            }
        };

        let mut output_rx = process.output_tx.subscribe();
        while let Ok(line) = output_rx.recv().await {
            {
                let mut teams = teams_store.write().await;
                if let Some(t) = teams.get_mut(&team_id) {
                    t.append_output(&line);
                }
            }
            let event = json!({
                "v": 1,
                "type": "team_output",
                "team_id": team_id,
                "line": line,
            });
            let _ = ws_tx.send(event.to_string());
        }

        let _ = process.child.wait().await;
    });

    (StatusCode::ACCEPTED, Json(json!({ "status": "message queued" }))).into_response()
}

/// DELETE /api/teams/:id
pub async fn kill_team(
    _auth: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut processes = state.processes.write().await;
    match processes.get_mut(&id) {
        None => {
            // Team may not have a running process (e.g. already done).
            // Still transition status if the team exists.
            let mut teams = state.teams.write().await;
            if teams.get(&id).is_none() {
                return (StatusCode::NOT_FOUND, Json(json!({ "error": "Team not found" }))).into_response();
            }
            if let Some(team) = teams.get_mut(&id) {
                let _ = team.transition_to(TeamStatus::Killed);
            }
            let event = json!({ "v": 1, "type": "team_done", "team_id": id, "summary": "Killed by user" });
            let _ = state.ws_broadcast.send(event.to_string());
            (StatusCode::OK, Json(json!({ "status": "killed" }))).into_response()
        }
        Some(process) => {
            let _ = process.child.start_kill();
            {
                let mut teams = state.teams.write().await;
                if let Some(team) = teams.get_mut(&id) {
                    let _ = team.transition_to(TeamStatus::Killed);
                }
            }
            let event = json!({ "v": 1, "type": "team_done", "team_id": id, "summary": "Killed by user" });
            let _ = state.ws_broadcast.send(event.to_string());
            (StatusCode::OK, Json(json!({ "status": "killed" }))).into_response()
        }
    }
}

/// GET /api/summary
pub async fn summary(
    _auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let teams = state.teams.read().await;
    let list: Vec<serde_json::Value> = teams
        .values()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "task": t.task,
                "status": t.status,
                "summary": t.summary,
                "created_at": t.created_at,
                "finished_at": t.finished_at,
            })
        })
        .collect();
    Json(json!({ "teams": list }))
}
