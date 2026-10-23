use crate::domain::{AuthSource, Authority, PlatformError, Principal};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

fn default_iss() -> String {
    "auth-service".to_string()
}

fn default_provider() -> String {
    "AD".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub authorities: Vec<Authority>,
    #[serde(default)]
    pub groups: Vec<String>,
    pub exp: usize,
    pub iat: usize,
    #[serde(default = "default_iss")]
    pub iss: String,
    #[serde(rename = "identityId", default)]
    pub identity_id: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub has_wheel: bool,
}

pub trait TokenService: Send + Sync {
    fn issue_token(&self, principal: &Principal) -> Result<String, PlatformError>;
    fn verify_token(&self, token: &str) -> Result<Principal, PlatformError>;
}

pub struct JwtTokenService {
    secret: String,
    ttl_seconds: i64,
}

impl JwtTokenService {
    pub fn new(secret: impl Into<String>, ttl_seconds: i64) -> Self {
        Self {
            secret: secret.into(),
            ttl_seconds,
        }
    }
}

impl TokenService for JwtTokenService {
    fn issue_token(&self, principal: &Principal) -> Result<String, PlatformError> {
        let now = Utc::now().timestamp() as usize;
        let exp = now + (self.ttl_seconds as usize);

        let is_admin = principal.is_admin()
            || principal.authorities().iter().any(|a| {
                let l = a.authority.to_lowercase();
                l.contains("admin") || l.contains("superuser")
            })
            || principal.username().eq_ignore_ascii_case("Administrator")
            || principal.username() == "root";

        let has_wheel = if principal.source() == &AuthSource::Postgres {
            false
        } else {
            is_admin || principal.groups().iter().any(|g| g == "wheel" || g == "sudo")
        };

        let provider_str = match principal.source() {
            AuthSource::ActiveDirectory => "AD".to_string(),
            AuthSource::Postgres => "Postgres".to_string(),
            AuthSource::Linux => "Linux".to_string(),
            AuthSource::Certificate => "Certificate".to_string(),
            AuthSource::OAuth2 => "OAuth2".to_string(),
        };

        let claims = Claims {
            sub: principal.username().to_string(),
            username: principal.username().to_string(),
            source: principal.source().as_str().to_string(),
            authorities: principal.authorities().to_vec(),
            groups: principal.groups().to_vec(),
            exp,
            iat: now,
            iss: "auth-service".to_string(),
            identity_id: principal.username().to_string(),
            provider: provider_str,
            is_admin,
            has_wheel,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| {
            error!("JwtTokenService failed to issue token: {}", e);
            PlatformError::Internal("Token generation failed".into())
        })
    }

    fn verify_token(&self, token: &str) -> Result<Principal, PlatformError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.validate_aud = false;
        validation.required_spec_claims.remove("iss");

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| {
            let claims = data.claims;
            let src = if !claims.source.is_empty() {
                AuthSource::from_str(&claims.source)
            } else {
                AuthSource::from_str(&claims.provider)
            };
            Principal::new(
                claims.username,
                src,
                claims.authorities,
                claims.groups,
            )
        })
        .map_err(|e| {
            warn!("JwtTokenService failed to verify token: {}", e);
            PlatformError::InvalidToken("Expired or malformed token".into())
        })
    }
}
