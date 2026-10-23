use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Json, Redirect, Response},
};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tracing::warn;

#[derive(Clone)]
pub struct PlatformController {
    static_dir: PathBuf,
}

impl PlatformController {
    pub fn new(static_dir: PathBuf) -> Self {
        Self { static_dir }
    }

    pub fn handle_health(&self) -> Json<serde_json::Value> {
        Json(json!({ "status": "UP" }))
    }

    pub fn handle_redirect_app(&self) -> Redirect {
        Redirect::to("/app/")
    }

    pub fn handle_api_docs(&self) -> Json<serde_json::Value> {
        Json(json!({
            "openapi": "3.0.1",
            "info": {
                "title": "Astral Platform HCI API",
                "version": "2.0.0",
                "description": "API Gateway e Hub Central em Arquitetura Orientada a Objetos"
            },
            "paths": {
                "/api/auth/login": {
                    "post": {
                        "tags": ["auth"],
                        "summary": "Autenticar usuário e gerar sessão/token",
                        "responses": { "200": { "description": "OK" }, "401": { "description": "Não autorizado" } }
                    }
                },
                "/api/auth/me": {
                    "get": {
                        "tags": ["auth"],
                        "summary": "Obter perfil do usuário logado",
                        "responses": { "200": { "description": "OK" }, "401": { "description": "Não autorizado" } }
                    }
                },
                "/api/auth/logout": {
                    "post": {
                        "tags": ["auth"],
                        "summary": "Encerrar sessão ativa",
                        "responses": { "200": { "description": "OK" } }
                    }
                },
                "/api/auth/sso": {
                    "get": {
                        "tags": ["auth"],
                        "summary": "Obter configuração de SSO",
                        "responses": { "200": { "description": "OK" } }
                    }
                },
                "/api/v1/acl/check": {
                    "get": {
                        "tags": ["acl"],
                        "summary": "Validação de ACL de subrequest para Nginx",
                        "responses": { "200": { "description": "Permitido" }, "401": { "description": "Não autenticado" } }
                    }
                }
            }
        }))
    }

    pub async fn handle_spa_index(&self) -> Response {
        let index_path = self.static_dir.join("app/index.html");
        match fs::read(&index_path).await {
            Ok(bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                bytes,
            )
                .into_response(),
            Err(e) => {
                warn!("PlatformController: failed to read SPA index.html ({:?}): {}", index_path, e);
                (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/plain")],
                    "Astral Platform index.html not found",
                )
                    .into_response()
            }
        }
    }
}
