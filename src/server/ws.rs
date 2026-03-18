use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use crate::auth::{verify_ws_token, WsTokenQuery, AuthError};
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsTokenQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Validate token before upgrading; reject unauthenticated connections.
    if verify_ws_token(&query, &state.jwt_secret).is_err() {
        return AuthError::InvalidToken.into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_broadcast.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if socket.send(Message::Text(text.into())).await.is_err() { break; }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}
