use crate::domain::{AuthSource, Authority, Credentials, PlatformError, Principal};
use crate::service::{AuthenticationProvider, TokenService};
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyManualCertPayload {
    pub certificate: String, // PEM format
    pub challenge: String,
    pub signature: String,   // Base64
}

#[derive(Serialize)]
pub struct AuthResponseDto {
    pub authenticated: bool,
    pub username: String,
    pub source: String,
    pub token: String,
    pub authorities: Vec<Authority>,
}

#[derive(Serialize)]
pub struct MeResponseDto {
    pub authenticated: bool,
    pub username: String,
    pub source: String,
    pub authorities: Vec<Authority>,
}

fn extract_cn_from_dn(dn: &str) -> Option<String> {
    for part in dn.split(',') {
        let trimmed = part.trim();
        if trimmed.starts_with("CN=") || trimmed.starts_with("cn=") {
            let cn = trimmed[3..].trim();
            if let Some(slash_idx) = cn.find('/') {
                return Some(cn[..slash_idx].trim().to_string());
            }
            return Some(cn.to_string());
        }
    }
    None
}

#[derive(Clone)]
pub struct AuthController {
    auth_provider: Arc<dyn AuthenticationProvider>,
    token_service: Arc<dyn TokenService>,
    token_ttl_seconds: i64,
    challenge_cache: Arc<Mutex<HashMap<String, Instant>>>,
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
            challenge_cache: Arc::new(Mutex::new(HashMap::new())),
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
                    if let Some(val) = part.strip_prefix("sso_token=") {
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
                let sso_cookie = format!(
                    "sso_token={}; Path=/; SameSite=Lax; Max-Age={}",
                    token, self.token_ttl_seconds
                );

                let mut headers = HeaderMap::new();
                if let Ok(v) = HeaderValue::from_str(&session_cookie) {
                    headers.append(header::SET_COOKIE, v);
                }
                if let Ok(v) = HeaderValue::from_str(&token_cookie) {
                    headers.append(header::SET_COOKIE, v);
                }
                if let Ok(v) = HeaderValue::from_str(&sso_cookie) {
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

    pub async fn handle_cert_challenge(&self) -> Json<serde_json::Value> {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let challenge = format!("astral-challenge-{:x}-{:x}", nanos, std::process::id());

        let mut cache = self.challenge_cache.lock().await;
        let now = Instant::now();
        cache.retain(|_, time| now.duration_since(*time).as_secs() < 300);
        cache.insert(challenge.clone(), now);

        Json(json!({
            "challenge": challenge,
            "expires_in": 300
        }))
    }

    pub async fn handle_cert_verify_manual(&self, payload: VerifyManualCertPayload) -> Response {
        let mut cache = self.challenge_cache.lock().await;
        let now = Instant::now();
        cache.retain(|_, time| now.duration_since(*time).as_secs() < 300);
        if cache.remove(&payload.challenge).is_none() {
            return PlatformError::InvalidCredentials("Desafio criptográfico expirado ou inválido".into())
                .to_json_response()
                .into_response();
        }
        drop(cache);

        let cert = match openssl::x509::X509::from_pem(payload.certificate.as_bytes()) {
            Ok(c) => c,
            Err(e) => {
                return PlatformError::InvalidCredentials(format!("Certificado X.509 PEM inválido: {}", e))
                    .to_json_response()
                    .into_response();
            }
        };

        let now_asn1 = match openssl::asn1::Asn1Time::days_from_now(0) {
            Ok(t) => t,
            Err(_) => {
                return PlatformError::Internal("Erro ao obter timestamp do sistema".into())
                    .to_json_response()
                    .into_response();
            }
        };

        if cert.not_before() > now_asn1 || cert.not_after() < now_asn1 {
            return PlatformError::InvalidCredentials("Certificado digital fora do período de validade".into())
                .to_json_response()
                .into_response();
        }

        let pubkey = match cert.public_key() {
            Ok(k) => k,
            Err(e) => {
                return PlatformError::InvalidCredentials(format!("Falha ao extrair chave pública: {}", e))
                    .to_json_response()
                    .into_response();
            }
        };

        let mut verifier = match openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &pubkey) {
            Ok(v) => v,
            Err(e) => {
                return PlatformError::InvalidCredentials(format!("Falha no verificador de assinatura: {}", e))
                    .to_json_response()
                    .into_response();
            }
        };

        if let Err(e) = verifier.update(payload.challenge.as_bytes()) {
            return PlatformError::InvalidCredentials(format!("Erro ao processar desafio: {}", e))
                .to_json_response()
                .into_response();
        }

        let clean_sig = payload.signature.trim().replace('\n', "").replace('\r', "");
        let sig_bytes = match openssl::base64::decode_block(&clean_sig) {
            Ok(b) => b,
            Err(e) => {
                return PlatformError::InvalidCredentials(format!("Assinatura base64 corrompida: {}", e))
                    .to_json_response()
                    .into_response();
            }
        };

        match verifier.verify(&sig_bytes) {
            Ok(true) => {}
            Ok(false) => {
                return PlatformError::InvalidCredentials("Assinatura criptográfica não corresponde à chave pública do certificado".into())
                    .to_json_response()
                    .into_response();
            }
            Err(e) => {
                return PlatformError::InvalidCredentials(format!("Erro ao verificar assinatura: {}", e))
                    .to_json_response()
                    .into_response();
            }
        }

        let mut username = "cert_operator".to_string();
        for entry in cert.subject_name().entries() {
            if entry.object().nid() == openssl::nid::Nid::COMMONNAME {
                let data = entry.data();
                username = String::from_utf8_lossy(data.as_slice()).to_string();
                break;
            }
        }

        let is_admin = username.eq_ignore_ascii_case("Administrator")
            || username == "root"
            || username == "dark"
            || username == "euripedes";

        let mut authorities = vec![Authority::new("ROLE_USER")];
        let mut groups = vec!["users".to_string()];
        if is_admin {
            authorities.push(Authority::new("ROLE_ASTRAL_ADMIN"));
            groups.push("wheel".to_string());
            groups.push("Domain Admins".to_string());
        }

        let principal = Principal::new(
            username,
            AuthSource::Certificate,
            authorities,
            groups,
        );

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
        let sso_cookie = format!(
            "sso_token={}; Path=/; SameSite=Lax; Max-Age={}",
            token, self.token_ttl_seconds
        );

        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&session_cookie) {
            headers.append(header::SET_COOKIE, v);
        }
        if let Ok(v) = HeaderValue::from_str(&token_cookie) {
            headers.append(header::SET_COOKIE, v);
        }
        if let Ok(v) = HeaderValue::from_str(&sso_cookie) {
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

    pub fn handle_cert_browser(&self, headers: &HeaderMap) -> Response {
        let verify = headers
            .get("X-SSL-Client-Verify")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !verify.eq_ignore_ascii_case("SUCCESS") {
            return PlatformError::InvalidCredentials(
                "Nenhum certificado digital válido apresentado na sessão mTLS do navegador. Utilize a opção de seleção de arquivo .p12/.pfx ou acesse com certificado instalado.".into(),
            )
            .to_json_response()
            .into_response();
        }

        let s_dn = headers
            .get("X-SSL-Client-S-DN")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let username = extract_cn_from_dn(s_dn).unwrap_or_else(|| "cert_user".to_string());
        let is_admin = username.eq_ignore_ascii_case("Administrator")
            || username == "root"
            || username == "dark"
            || username == "euripedes";

        let mut authorities = vec![Authority::new("ROLE_USER")];
        let mut groups = vec!["users".to_string()];
        if is_admin {
            authorities.push(Authority::new("ROLE_ASTRAL_ADMIN"));
            groups.push("wheel".to_string());
            groups.push("Domain Admins".to_string());
        }

        let principal = Principal::new(
            username,
            AuthSource::Certificate,
            authorities,
            groups,
        );

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
        let sso_cookie = format!(
            "sso_token={}; Path=/; SameSite=Lax; Max-Age={}",
            token, self.token_ttl_seconds
        );

        let mut resp_headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&session_cookie) {
            resp_headers.append(header::SET_COOKIE, v);
        }
        if let Ok(v) = HeaderValue::from_str(&token_cookie) {
            resp_headers.append(header::SET_COOKIE, v);
        }
        if let Ok(v) = HeaderValue::from_str(&sso_cookie) {
            resp_headers.append(header::SET_COOKIE, v);
        }
        resp_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let body = serde_json::to_string(&AuthResponseDto {
            authenticated: true,
            username: principal.username().to_string(),
            source: principal.source().as_str().to_string(),
            token,
            authorities: principal.authorities().to_vec(),
        })
        .unwrap_or_default();

        (StatusCode::OK, resp_headers, body).into_response()
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
        let clear_sso = "sso_token=; Path=/; SameSite=Lax; Max-Age=0";
        headers.append(header::SET_COOKIE, HeaderValue::from_static(clear_session));
        headers.append(header::SET_COOKIE, HeaderValue::from_static(clear_token));
        headers.append(header::SET_COOKIE, HeaderValue::from_static(clear_sso));
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

        (StatusCode::OK, headers, r#"{"authenticated":false}"#).into_response()
    }


}
