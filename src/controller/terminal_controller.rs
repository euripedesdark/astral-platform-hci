use crate::controller::auth_controller::AuthController;
use crate::domain::{AuthSource, PlatformError};
use crate::service::TokenService;
use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Deserialize)]
pub struct TerminalExecPayload {
    pub command: String,
}

#[derive(Serialize)]
pub struct TerminalStatusResponse {
    pub allowed: bool,
    pub username: String,
    pub source: String,
    pub message: String,
}

#[derive(Clone)]
pub struct TerminalController {
    token_service: Arc<dyn TokenService>,
}

impl TerminalController {
    pub fn new(token_service: Arc<dyn TokenService>) -> Self {
        Self { token_service }
    }

    pub fn handle_status(&self, headers: &HeaderMap) -> Response {
        let token = match AuthController::extract_token(headers) {
            Some(t) => t,
            None => return PlatformError::MissingToken.to_json_response().into_response(),
        };

        match self.token_service.verify_token(&token) {
            Ok(principal) => {
                if principal.source() == &AuthSource::Postgres {
                    let body = TerminalStatusResponse {
                        allowed: false,
                        username: principal.username().to_string(),
                        source: "POSTGRES".to_string(),
                        message: "Acesso Negado: O terminal do sistema operacional é restrito a administradores do Samba Active Directory ou operadores Linux (sudo/root). Superusuários PostgreSQL não possuem permissão de acesso ao terminal.".to_string(),
                    };
                    (StatusCode::FORBIDDEN, Json(body)).into_response()
                } else {
                    let body = TerminalStatusResponse {
                        allowed: true,
                        username: principal.username().to_string(),
                        source: principal.source().as_str().to_string(),
                        message: "Terminal operacional autorizado.".to_string(),
                    };
                    (StatusCode::OK, Json(body)).into_response()
                }
            }
            Err(e) => e.to_json_response().into_response(),
        }
    }

    pub async fn handle_exec(&self, headers: HeaderMap, payload: TerminalExecPayload) -> Response {
        let token = match AuthController::extract_token(&headers) {
            Some(t) => t,
            None => return PlatformError::MissingToken.to_json_response().into_response(),
        };

        let principal = match self.token_service.verify_token(&token) {
            Ok(p) => p,
            Err(e) => return e.to_json_response().into_response(),
        };

        if principal.source() == &AuthSource::Postgres {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "success": false,
                    "error": "Acesso negado: superusuários PostgreSQL não possuem permissão de acesso ao terminal.",
                    "allowed": false
                })),
            )
                .into_response();
        }

        let cmd = payload.command.trim();
        if cmd.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Comando não pode ser vazio."
                })),
            )
                .into_response();
        }

        match timeout(
            Duration::from_secs(15),
            Command::new("bash").arg("-c").arg(cmd).output(),
        )
        .await
        {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code().unwrap_or(-1);
                (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "stdout": stdout,
                        "stderr": stderr,
                        "exit_code": exit_code
                    })),
                )
                    .into_response()
            }
            Ok(Err(e)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Falha ao executar comando: {}", e)
                })),
            )
                .into_response(),
            Err(_) => (
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({
                    "success": false,
                    "error": "Timeout: o comando ultrapassou o tempo limite de 15 segundos."
                })),
            )
                .into_response(),
        }
    }
}
