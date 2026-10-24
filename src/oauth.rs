//! Confidential OAuth2 client. Codes and tokens stay on the server; the browser gets a session.
use crate::{
    domain::{AuthSource, Authority, Principal},
    service::TokenService,
};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use openssl::{rand::rand_bytes, sha::sha256};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use url::Url;

#[derive(Clone)]
struct Config {
    authorization: Url,
    token: Url,
    userinfo: Url,
    redirect: Url,
    client_id: String,
    client_secret: String,
}
struct Pending {
    verifier: String,
    expires: Instant,
}
pub struct OAuthClient {
    config: Option<Config>,
    http: reqwest::Client,
    pending: Mutex<HashMap<String, Pending>>,
    tokens: Arc<dyn TokenService>,
}
fn valid_url(url: &Url) -> bool {
    (url.scheme() == "https"
        || (url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))))
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && url.query().is_none()
}
impl OAuthClient {
    pub fn load(tokens: Arc<dyn TokenService>) -> Result<Self, Box<dyn std::error::Error>> {
        let names = [
            "ASTRAL_OAUTH_AUTHORIZATION_URL",
            "ASTRAL_OAUTH_TOKEN_URL",
            "ASTRAL_OAUTH_USERINFO_URL",
            "ASTRAL_OAUTH_REDIRECT_URI",
            "ASTRAL_OAUTH_CLIENT_ID",
            "ASTRAL_OAUTH_CLIENT_SECRET",
        ];
        let values: Vec<Option<String>> = names
            .iter()
            .map(|key| std::env::var(key).ok().filter(|v| !v.trim().is_empty()))
            .collect();
        let config = if values.iter().all(Option::is_none) {
            None
        } else {
            if values.iter().any(Option::is_none) {
                return Err("Configuracao OAuth2 incompleta: execute configure_oauth2.py".into());
            }
            let value = |n: usize| values[n].as_ref().unwrap().clone();
            Some(Config {
                authorization: Url::parse(&value(0))?,
                token: Url::parse(&value(1))?,
                userinfo: Url::parse(&value(2))?,
                redirect: Url::parse(&value(3))?,
                client_id: value(4),
                client_secret: value(5),
            })
        };
        let mut http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none());
        if let Ok(path) = std::env::var("ASTRAL_OAUTH_CA_FILE") {
            http =
                http.add_root_certificate(reqwest::Certificate::from_pem(&std::fs::read(path)?)?);
        }
        Self::new(config, http.build()?, tokens)
    }
    fn new(
        config: Option<Config>,
        http: reqwest::Client,
        tokens: Arc<dyn TokenService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(c) = &config {
            if [&c.authorization, &c.token, &c.userinfo, &c.redirect]
                .iter()
                .any(|u| !valid_url(u))
                || c.client_id.is_empty()
                || c.client_secret.len() < 32
            {
                return Err("OAuth2 exige URLs HTTPS (HTTP somente loopback), sem userinfo/query/fragment e segredo >=32 caracteres".into());
            }
        }
        Ok(Self {
            config,
            http,
            pending: Mutex::new(HashMap::new()),
            tokens,
        })
    }
    pub fn status(&self) -> Json<serde_json::Value> {
        Json(
            json!({"enabled":self.config.is_some(), "authorizationUrl": if self.config.is_some() {"/oauth2/authorization/auth-service"} else {""}}),
        )
    }
}
fn constant_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && openssl::memcmp::eq(a, b)
}
fn random() -> Result<String, openssl::error::ErrorStack> {
    let mut bytes = [0u8; 32];
    rand_bytes(&mut bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|v| v.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, v)| v.to_owned())
}
fn set_cookie(response: &mut Response, name: &str, value: &str, secure: bool, age: u64) {
    if let Ok(value) = HeaderValue::from_str(&format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={age}{}",
        if secure { "; Secure" } else { "" }
    )) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}
fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response
}
fn failure(secure: bool) -> Response {
    let mut response = no_store(Redirect::to("/app/?ssoError=1&local=1").into_response());
    set_cookie(&mut response, "ASTRAL_OAUTH_STATE", "", secure, 0);
    response
}
async fn start(State(client): State<Arc<OAuthClient>>) -> Response {
    let config = match &client.config {
        Some(v) => v,
        None => return failure(true),
    };
    let secure = config.redirect.scheme() == "https";
    let state = match random() {
        Ok(v) => v,
        Err(_) => return failure(secure),
    };
    let verifier = match random() {
        Ok(v) => v,
        Err(_) => return failure(secure),
    };
    let challenge = URL_SAFE_NO_PAD.encode(sha256(verifier.as_bytes()));
    let mut pending = client.pending.lock().await;
    pending.retain(|_, item| item.expires > Instant::now());
    if pending.len() >= 1024 {
        return failure(secure);
    }
    pending.insert(
        state.clone(),
        Pending {
            verifier,
            expires: Instant::now() + Duration::from_secs(300),
        },
    );
    let mut url = config.authorization.clone();
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", config.redirect.as_str())
        .append_pair("scope", "profile groups")
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");
    let mut response = no_store(Redirect::to(url.as_str()).into_response());
    set_cookie(&mut response, "ASTRAL_OAUTH_STATE", &state, secure, 300);
    response
}
#[derive(Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}
#[derive(Deserialize)]
struct Token {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Identity {
    identity_id: String,
    username: String,
    provider: String,
    #[serde(default)]
    groups: Vec<String>,
    #[serde(default)]
    is_admin: bool,
    #[serde(default)]
    has_wheel: bool,
}
async fn callback(
    State(client): State<Arc<OAuthClient>>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Response {
    let config = match &client.config {
        Some(v) => v,
        None => return failure(true),
    };
    let secure = config.redirect.scheme() == "https";
    let state = match (query.state, cookie(&headers, "ASTRAL_OAUTH_STATE")) {
        (Some(state), Some(bound)) if constant_eq(state.as_bytes(), bound.as_bytes()) => state,
        _ => return failure(secure),
    };
    // Consume before network IO: a callback can never be exchanged twice.
    let pending = match client.pending.lock().await.remove(&state) {
        Some(value) if value.expires > Instant::now() => value,
        _ => return failure(secure),
    };
    let code = match (query.error, query.code) {
        (None, Some(v)) if !v.is_empty() => v,
        _ => return failure(secure),
    };
    let result = async {
        let response = client
            .http
            .post(config.token.clone())
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("redirect_uri", config.redirect.as_str()),
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("code_verifier", pending.verifier.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?;
        let token: Token = response.json().await?;
        if !token.token_type.eq_ignore_ascii_case("bearer")
            || token.access_token.is_empty()
            || token.access_token.len() > 8192
            || token.expires_in == 0
        {
            return Err("invalid token response".into());
        }
        // Identity is fetched only from the statically configured, trusted resource endpoint.
        let identity: Identity = client
            .http
            .get(config.userinfo.clone())
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if identity.identity_id.is_empty()
            || identity.username.is_empty()
            || !matches!(
                identity.provider.to_uppercase().as_str(),
                "AD" | "POSTGRES" | "POSTGRESQL" | "LINUX" | "CERTIFICATE"
            )
        {
            return Err("invalid identity".into());
        }
        let roles = if identity.is_admin {
            vec![
                Authority::new("ROLE_USER"),
                Authority::new("ROLE_ASTRAL_ADMIN"),
            ]
        } else {
            vec![Authority::new("ROLE_USER")]
        };
        let source = if identity.provider.eq_ignore_ascii_case("POSTGRESQL") {
            AuthSource::Postgres
        } else {
            AuthSource::from_str(&identity.provider)
        };
        let principal = Principal::new(identity.username, source, roles, identity.groups)
            .with_identity_id(identity.identity_id)
            .with_wheel(identity.has_wheel);
        let session = client
            .tokens
            .issue_token_with_ttl(&principal, token.expires_in.min(28800) as i64)
            .map_err(|_| "session issuance failed")?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((session, token.expires_in.min(28800)))
    }
    .await;
    match result {
        Ok((session, ttl)) => {
            let mut response = no_store(Redirect::to("/app/").into_response());
            set_cookie(&mut response, "ASTRAL_OAUTH_STATE", "", secure, 0);
            for name in ["ASTRAL_SESSION", "astral_token", "sso_token"] {
                set_cookie(&mut response, name, &session, secure, ttl);
            }
            response
        }
        Err(_) => {
            tracing::warn!("OAuth2 callback failed; credentials, codes and tokens omitted");
            failure(secure)
        }
    }
}
pub fn router(client: Arc<OAuthClient>) -> Router {
    Router::new()
        .route("/oauth2/authorization/auth-service", get(start))
        .route("/login/oauth2/code/auth-service", get(callback))
        .route(
            "/api/auth/sso",
            get(|State(c): State<Arc<OAuthClient>>| async move { c.status() }),
        )
        .with_state(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::JwtTokenService;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Upstream {
        challenge: Mutex<String>,
        exchanges: AtomicUsize,
    }
    async fn token_mock(
        State(state): State<Arc<Upstream>>,
        axum::extract::Form(form): axum::extract::Form<HashMap<String, String>>,
    ) -> Response {
        state.exchanges.fetch_add(1, Ordering::SeqCst);
        assert_eq!(form["client_secret"], "s".repeat(48));
        assert_eq!(
            URL_SAFE_NO_PAD.encode(sha256(form["code_verifier"].as_bytes())),
            *state.challenge.lock().await
        );
        assert_eq!(
            form["redirect_uri"],
            "https://portal.example/login/oauth2/code/auth-service"
        );
        if form["code"] != "valid-code" {
            return axum::http::StatusCode::BAD_REQUEST.into_response();
        }
        Json(json!({"access_token":"opaque-provider-token","token_type":"Bearer","expires_in":900}))
            .into_response()
    }
    async fn info_mock(headers: HeaderMap) -> Json<serde_json::Value> {
        assert_eq!(
            headers[header::AUTHORIZATION],
            "Bearer opaque-provider-token"
        );
        Json(
            json!({"identityId":"GUID-stable-123","username":"Administrator","provider":"AD","groups":["Domain Users"],"isAdmin":false,"hasWheel":false}),
        )
    }
    async fn fixture() -> (
        Arc<OAuthClient>,
        Arc<Upstream>,
        String,
        reqwest::Client,
        Arc<dyn TokenService>,
    ) {
        let upstream = Arc::new(Upstream {
            challenge: Mutex::new(String::new()),
            exchanges: AtomicUsize::new(0),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let app = Router::new()
            .route("/token", axum::routing::post(token_mock))
            .route("/userinfo", get(info_mock))
            .with_state(upstream.clone());
        tokio::spawn(async {
            axum::serve(listener, app).await.unwrap();
        });
        let tokens: Arc<dyn TokenService> =
            Arc::new(JwtTokenService::new("local-test-signing-key", 900));
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let config = Config {
            authorization: Url::parse("https://identity.example/oauth2/authorize").unwrap(),
            token: Url::parse(&format!("{base}/token")).unwrap(),
            userinfo: Url::parse(&format!("{base}/userinfo")).unwrap(),
            redirect: Url::parse("https://portal.example/login/oauth2/code/auth-service").unwrap(),
            client_id: "astral-hci".into(),
            client_secret: "s".repeat(48),
        };
        let client =
            Arc::new(OAuthClient::new(Some(config), http.clone(), tokens.clone()).unwrap());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let app = router(client.clone());
        tokio::spawn(async {
            axum::serve(listener, app).await.unwrap();
        });
        (client, upstream, base, http, tokens)
    }
    async fn initiate(http: &reqwest::Client, base: &str, upstream: &Upstream) -> (String, String) {
        let result = http
            .get(format!("{base}/oauth2/authorization/auth-service"))
            .header(header::HOST, "attacker.invalid")
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), 303);
        let auth = Url::parse(result.headers()[header::LOCATION].to_str().unwrap()).unwrap();
        assert_eq!(auth.host_str(), Some("identity.example"));
        let params: HashMap<String, String> = auth.query_pairs().into_owned().collect();
        assert_eq!(params["code_challenge_method"], "S256");
        assert_eq!(
            params["redirect_uri"],
            "https://portal.example/login/oauth2/code/auth-service"
        );
        *upstream.challenge.lock().await = params["code_challenge"].clone();
        let cookie = result.headers()[header::SET_COOKIE].to_str().unwrap();
        assert!(
            cookie.contains("HttpOnly")
                && cookie.contains("Secure")
                && cookie.contains("SameSite=Lax")
        );
        (
            params["state"].clone(),
            cookie.split(';').next().unwrap().into(),
        )
    }
    #[tokio::test]
    async fn callback_validates_browser_state_exchanges_code_and_preserves_identity() {
        let (client, upstream, base, http, tokens) = fixture().await;
        let (state, cookie) = initiate(&http, &base, &upstream).await;
        let path = format!("{base}/login/oauth2/code/auth-service");
        let bad = http
            .get(&path)
            .query(&[("state", state.as_str()), ("code", "valid-code")])
            .send()
            .await
            .unwrap();
        assert!(bad.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("ssoError"));
        assert_eq!(upstream.exchanges.load(Ordering::SeqCst), 0);
        let bad = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", "attacker-state"), ("code", "valid-code")])
            .send()
            .await
            .unwrap();
        assert!(bad.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("local=1"));
        assert_eq!(upstream.exchanges.load(Ordering::SeqCst), 0);
        let good = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", state.as_str()), ("code", "valid-code")])
            .send()
            .await
            .unwrap();
        assert_eq!(good.headers()[header::LOCATION], "/app/");
        assert_eq!(upstream.exchanges.load(Ordering::SeqCst), 1);
        let session = good
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|v| v.starts_with("ASTRAL_SESSION="))
            .unwrap();
        let jwt = session
            .split(';')
            .next()
            .unwrap()
            .split_once('=')
            .unwrap()
            .1;
        let payload: serde_json::Value = serde_json::from_slice(
            &URL_SAFE_NO_PAD
                .decode(jwt.split('.').nth(1).unwrap())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            payload["exp"].as_u64().unwrap() - payload["iat"].as_u64().unwrap(),
            900
        );
        assert_eq!(payload["provider"], "AD");
        let principal = tokens.verify_token(jwt).unwrap();
        assert_eq!(principal.identity_id(), "GUID-stable-123");
        assert_eq!(principal.username(), "Administrator");
        assert!(!principal.is_admin());
        assert_eq!(principal.wheel(), Some(false));
        let again = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", state.as_str()), ("code", "valid-code")])
            .send()
            .await
            .unwrap();
        assert!(again.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("ssoError"));
        assert_eq!(upstream.exchanges.load(Ordering::SeqCst), 1);
        let (state, cookie) = initiate(&http, &base, &upstream).await;
        client.pending.lock().await.get_mut(&state).unwrap().expires =
            Instant::now() - Duration::from_secs(1);
        let expired = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", state.as_str()), ("code", "valid-code")])
            .send()
            .await
            .unwrap();
        assert!(expired.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("ssoError"));
        let (state, cookie) = initiate(&http, &base, &upstream).await;
        let denied = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", state.as_str()), ("error", "access_denied")])
            .send()
            .await
            .unwrap();
        assert!(denied.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("ssoError"));
        let (state, cookie) = initiate(&http, &base, &upstream).await;
        let invalid = http
            .get(&path)
            .header(header::COOKIE, &cookie)
            .query(&[("state", state.as_str()), ("code", "invalid-code")])
            .send()
            .await
            .unwrap();
        assert!(invalid.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .contains("ssoError"));
    }
    #[test]
    fn reject_remote_plaintext_and_userinfo_urls() {
        for uri in [
            "http://identity.example/token",
            "https://user:pass@identity.example/token",
            "https://identity.example/token#x",
        ] {
            assert!(!valid_url(&Url::parse(uri).unwrap()));
        }
        let tokens = Arc::new(JwtTokenService::new("test", 900));
        let disabled = OAuthClient::new(None, reqwest::Client::new(), tokens).unwrap();
        assert_eq!(disabled.status().0["enabled"], false);
    }
}
