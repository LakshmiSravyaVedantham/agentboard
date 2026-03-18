use axum::{extract::{State, Path}, Json, response::IntoResponse, http::StatusCode};
use serde_json::json;
use std::sync::Arc;
use crate::state::AppState;
use crate::auth;

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
        let token = auth::create_token(&state.jwt_secret).unwrap();
        (StatusCode::OK, Json(json!({ "token": token })))
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid pairing code" })))
    }
}

pub async fn list_teams(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let teams = state.teams.read().await;
    let list: Vec<_> = teams.values().collect();
    Json(json!({ "teams": list }))
}

pub async fn submit_tasks() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn approve_plan() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn edit_plan() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn create_team() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn get_team(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({ "todo": true, "id": id }))
}

pub async fn send_message() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn kill_team() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}

pub async fn summary() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(json!({ "todo": true })))
}
