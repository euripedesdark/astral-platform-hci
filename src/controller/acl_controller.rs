use crate::domain::AclDecision;
use crate::service::AclService;
use crate::controller::auth_controller::AuthController;
use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AclController {
    acl_service: Arc<dyn AclService>,
}

impl AclController {
    pub fn new(acl_service: Arc<dyn AclService>) -> Self {
        Self { acl_service }
    }

    pub fn handle_check(&self, headers: &HeaderMap, uri: &Uri) -> Response {
        let raw_uri = headers
            .get("x-original-uri")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_else(|| uri.path());

        let token = AuthController::extract_token(headers);
        match self.acl_service.evaluate(raw_uri, token.as_deref()) {
            AclDecision::Allow => StatusCode::OK.into_response(),
            AclDecision::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            AclDecision::Deny => StatusCode::FORBIDDEN.into_response(),
        }
    }
}
