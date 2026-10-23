use crate::domain::AclDecision;
use crate::service::token_service::TokenService;
use std::sync::Arc;

pub trait AclService: Send + Sync {
    fn evaluate(&self, path: &str, token: Option<&str>) -> AclDecision;
}

pub struct DefaultAclService {
    token_service: Arc<dyn TokenService>,
    public_prefixes: Vec<&'static str>,
}

impl DefaultAclService {
    pub fn new(token_service: Arc<dyn TokenService>) -> Self {
        Self {
            token_service,
            public_prefixes: vec![
                "/login",
                "/home",
                "/inicio",
                "/app",
                "/assets/",
                "/images/",
                "/fonts/",
                "/css/",
                "/js/",
                "/favicon.ico",
                "/api/auth/",
                "/actuator/",
                "/health",
                "/v3/api-docs",
                "/swagger-ui",
            ],
        }
    }

    pub fn is_public(&self, path: &str) -> bool {
        if path == "/" || path == "/favicon.ico" {
            return true;
        }

        for prefix in &self.public_prefixes {
            if path == *prefix || path.starts_with(prefix) {
                return true;
            }
        }

        false
    }
}

impl AclService for DefaultAclService {
    fn evaluate(&self, raw_path: &str, token: Option<&str>) -> AclDecision {
        let path = raw_path.split('?').next().unwrap_or(raw_path);

        if self.is_public(path) {
            return AclDecision::Allow;
        }

        if let Some(tok) = token {
            if self.token_service.verify_token(tok).is_ok() {
                return AclDecision::Allow;
            }
        }

        AclDecision::Unauthorized
    }
}
