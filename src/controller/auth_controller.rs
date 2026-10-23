use crate::domain::{Credentials, PlatformError};
use crate::service::{AuthenticationProvider, TokenService};
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponseDto {
    pub authenticated: bool,
    pub username: String,
    pub source: String,
    pub token: String,
    pub authorities: Vec<crate::domain::Authority>,
}

#[derive(Serialize)]
pub struct MeResponseDto {
    pub authenticated: bool,
    pub username: String,
    pub source: String,
    pub authorities: Vec<crate::domain::Authority>,
}

#[derive(Clone)]
pub struct AuthController {
    auth_provider: Arc<dyn AuthenticationProvider>,
    token_service: Arc<dyn TokenService>,
    token_ttl_seconds: i64,
}

impl AuthController {
    pub fn new(
        auth_provider: Arc<dyn AuthenticationProvider>,
        token_service: Arc<dyn TokenService>,
        token_ttl_seconds: i64,
    ) -> Self {
        Self {
            auth_provider,
            token_service,
            token_ttl_seconds,
        }
    }

    pub fn extract_token(headers: &HeaderMap) -> Option<String> {
        if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    let trimmed = token.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        if let Some(cookie_header) = headers.get(header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for part in cookie_str.split(';') {
                    let part = part.trim();
                    if let Some(val) = part.strip_prefix("ASTRAL_SESSION=") {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                    if let Some(val) = part.strip_prefix("astral_token=") {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    pub async fn handle_login(&self, payload: LoginPayload) -> Response {
        let creds = Credentials::new(&payload.username, &payload.password);
        match self.auth_provider.authenticate(&creds).await {
            Ok(principal) => {
                let token = match self.token_service.issue_token(&principal) {
                    Ok(t) => t,
                    Err(e) => return e.to_json_response().into_response(),
                };

                let session_cookie = format!(
                    "ASTRAL_SESSION={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
                    token, self.token_ttl_seconds
                );
                let token_cookie = format!(
                    "astral_token={}; Path=/; SameSite=Lax; Max-Age={}",
                    token, self.token_ttl_seconds
                );

                let mut headers = HeaderMap::new();
                if let Ok(v) = HeaderValue::from_str(&session_cookie) {
                    headers.append(header::SET_COOKIE, v);
                }
                if let Ok(v) = HeaderValue::from_str(&token_cookie) {
                    headers.append(header::SET_COOKIE, v);
                }
                headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

                let body = serde_json::to_string(&AuthResponseDto {
                    authenticated: true,
                    username: principal.username().to_string(),
                    source: principal.source().as_str().to_string(),
                    token,
                    authorities: principal.authorities().to_vec(),
                })
                .unwrap_or_default();

                (StatusCode::OK, headers, body).into_response()
            }
            Err(e) => e.to_json_response().into_response(),
        }
    }

    pub fn handle_me(&self, headers: &HeaderMap) -> Response {
        let token = match Self::extract_token(headers) {
            Some(t) => t,
            None => return PlatformError::MissingToken.to_json_response().into_response(),
        };

        match self.token_service.verify_token(&token) {
            Ok(principal) => {
                let body = MeResponseDto {
                    authenticated: true,
                    username: principal.username().to_string(),
                    source: principal.source().as_str().to_string(),
                    authorities: principal.authorities().to_vec(),
                };
                (StatusCode::OK, Json(body)).into_response()
            }
            Err(e) => e.to_json_response().into_response(),
        }
    }

    pub fn handle_logout(&self) -> Response {
        let mut headers = HeaderMap::new();
        let clear_session = "ASTRAL_SESSION=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
        let clear_token = "astral_token=; Path=/; SameSite=Lax; Max-Age=0";
        headers.append(header::SET_COOKIE, HeaderValue::from_static(clear_session));
        headers.append(header::SET_COOKIE, HeaderValue::from_static(clear_token));
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

        (StatusCode::OK, headers, r#"{"authenticated":false}"#).into_response()
    }

    pub fn handle_sso(&self) -> Json<serde_json::Value> {
        Json(json!({
            "enabled": false,
            "authorizationUrl": "/oauth2/authorization/auth-service"
        }))
    }
}
