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
        // These two routes start/complete authentication and must also work
        // behind nginx auth_request before a browser has a session cookie.
        if matches!(path,
            "/oauth2/authorization/auth-service" |
            "/login/oauth2/code/auth-service") {
            return true;
        }
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::JwtTokenService;

    #[test]
    fn anonymous_oauth_entry_and_callback_pass_acl_but_other_resources_do_not() {
        let acl = DefaultAclService::new(Arc::new(JwtTokenService::new("test-signing-key", 900)));
        for path in ["/oauth2/authorization/auth-service", "/oauth2/authorization/auth-service?continue=1",
            "/login/oauth2/code/auth-service?code=test&state=test"] {
            assert!(matches!(acl.evaluate(path, None), AclDecision::Allow), "{path}");
        }
        for path in ["/oauth2/authorization/auth-service/admin", "/oauth2/authorization/other-client",
            "/oauth2/admin", "/api/terminal/exec", "/api/v1/privileges/status"] {
            assert!(matches!(acl.evaluate(path, None), AclDecision::Unauthorized), "{path}");
        }
    }
}
