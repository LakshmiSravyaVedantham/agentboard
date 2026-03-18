pub mod routes;
pub mod ws;

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post, delete},
    response::{Html, IntoResponse},
    http::{header, Method, StatusCode as SC},
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct FrontendAssets;

async fn serve_frontend(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try exact path first
    if !path.is_empty() {
        if let Some(file) = FrontendAssets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (
                [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                file.data.to_vec(),
            )
                .into_response();
        }
    }

    // SPA fallback: serve index.html for all non-API, non-WS routes
    if let Some(file) = FrontendAssets::get("index.html") {
        return Html(String::from_utf8_lossy(&file.data).to_string()).into_response();
    }

    (SC::NOT_FOUND, "Not found").into_response()
}

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
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    public.merge(protected)
        .fallback(serve_frontend)
        .layer(cors)
        .with_state(state)
}
