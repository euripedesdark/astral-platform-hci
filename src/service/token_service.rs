use crate::domain::{AuthSource, Authority, PlatformError, Principal};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    username: String,
    source: String,
    authorities: Vec<Authority>,
    groups: Vec<String>,
    exp: usize,
    iat: usize,
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

        let claims = Claims {
            sub: principal.username().to_string(),
            username: principal.username().to_string(),
            source: principal.source().as_str().to_string(),
            authorities: principal.authorities().to_vec(),
            groups: principal.groups().to_vec(),
            exp,
            iat: now,
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

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| {
            let claims = data.claims;
            Principal::new(
                claims.username,
                AuthSource::from_str(&claims.source),
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
