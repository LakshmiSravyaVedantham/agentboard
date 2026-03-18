use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{Duration, Utc};
use std::sync::Arc;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_pairing_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..999999);
    code.to_string()
}

pub fn create_token(secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(30)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

/// Extractor that validates a JWT from `Authorization: Bearer <token>`.
/// Requires `Arc<AppState>` to be available in the router state.
pub struct AuthUser {
    pub claims: Claims,
}

pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(h) if h.starts_with("Bearer ") => &h["Bearer ".len()..],
            _ => return Err(AuthError::MissingToken),
        };

        verify_token(token, &app_state.jwt_secret)
            .map(|claims| AuthUser { claims })
            .map_err(|_| AuthError::InvalidToken)
    }
}

/// Query params struct for WebSocket token authentication.
#[derive(Deserialize)]
pub struct WsTokenQuery {
    pub token: Option<String>,
}

/// Validates a JWT token from the `?token=` query parameter.
/// Returns the claims on success, or an AuthError on failure.
pub fn verify_ws_token(
    query: &WsTokenQuery,
    secret: &str,
) -> Result<Claims, AuthError> {
    let token = query.token.as_deref().ok_or(AuthError::MissingToken)?;
    verify_token(token, secret).map_err(|_| AuthError::InvalidToken)
}
