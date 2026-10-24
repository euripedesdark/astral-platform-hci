mod config;
mod oauth;
mod controller;
mod domain;
mod server;
mod service;

use config::AppConfig;
use controller::{AclController, AuthController, PlatformController, TerminalController};
use domain::AuthSource;
use server::ApplicationServer;
use service::{
    DatabaseAuthRouter, DefaultAclService, JwtTokenService, LdapAuthProvider,
    MultiSourceAuthProvider, PostgresAuthProvider,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Initializing Astral Platform HCI (Rust OOP Architecture)...");

    // 1. Configuration
    let config = Arc::new(AppConfig::load());

    if !config.static_dir().join("app/index.html").is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound,
            "Frontend ausente: execute python3 deploy_platform.py ou npm run build em frontend; confira ASTRAL_STATIC_DIR").into());
    }

    // 2. Routing & Authentication Providers (Polymorphic Dependency Injection)
    let auth_router = Arc::new(DatabaseAuthRouter::new(config.pg_conn_str()));

    let ldap_provider = Arc::new(LdapAuthProvider::new(
        config.ad_url(),
        config.ad_domain(),
        config.ad_base_dn(),
        config.ad_admin_group(),
    ));

    let postgres_provider = Arc::new(PostgresAuthProvider::new(
        config.pg_host(),
        config.pg_port(),
        config.pg_dbname(),
        config.pg_admin_role(),
        AuthSource::Postgres,
    ));

    let linux_provider = Arc::new(PostgresAuthProvider::new(
        config.pg_host(),
        config.pg_port(),
        config.pg_dbname(),
        config.pg_admin_role(),
        AuthSource::Linux,
    ));

    let multi_source_provider = Arc::new(MultiSourceAuthProvider::new(
        auth_router,
        ldap_provider,
        postgres_provider,
        linux_provider,
    ));

    // 3. Domain Services
    let token_service = Arc::new(JwtTokenService::new(
        config.jwt_secret(),
        config.jwt_ttl_seconds(),
    ));

    let acl_service = Arc::new(DefaultAclService::new(token_service.clone()));

    let oauth = Arc::new(oauth::OAuthClient::load(token_service.clone())?);

    // 4. Controllers
    let auth_controller = Arc::new(AuthController::new(
        multi_source_provider,
        token_service.clone(),
        config.jwt_ttl_seconds(),
    ));

    let acl_controller = Arc::new(AclController::new(acl_service));

    let platform_controller = Arc::new(PlatformController::new(config.static_dir().to_path_buf()));

    let terminal_controller = Arc::new(TerminalController::new(token_service.clone()));

    // 5. Application Server
    let server = ApplicationServer::new(
        config,
        auth_controller,
        acl_controller,
        platform_controller,
        terminal_controller,
        oauth,
    );

    server.run().await
}
