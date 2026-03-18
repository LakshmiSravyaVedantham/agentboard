pub mod routes;
pub mod ws;

use crate::state::AppState;
use axum::{Router, routing::{get, post, delete}};
use axum::http::{Method, header::{CONTENT_TYPE, AUTHORIZATION}};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn build_router(state: Arc<AppState>) -> Router {
    let public = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/auth/pair", post(routes::pair));

    let protected = Router::new()
        .route("/api/tasks", post(routes::submit_tasks))
        .route("/api/plan/approve", post(routes::approve_plan))
        .route("/api/plan/edit", post(routes::edit_plan))
        .route("/api/teams", get(routes::list_teams))
        .route("/api/teams", post(routes::create_team))
        .route("/api/teams/{id}", get(routes::get_team))
        .route("/api/teams/{id}/message", post(routes::send_message))
        .route("/api/teams/{id}", delete(routes::kill_team))
        .route("/api/summary", get(routes::summary))
        .route("/ws", get(ws::ws_handler));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    public.merge(protected)
        .layer(cors)
        .with_state(state)
}
