use crate::config::AppConfig;
use crate::controller::{AclController, AuthController, LoginPayload, PlatformController};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, Uri},
    response::Redirect,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct ApplicationContainer {
    pub config: Arc<AppConfig>,
    pub auth_controller: Arc<AuthController>,
    pub acl_controller: Arc<AclController>,
    pub platform_controller: Arc<PlatformController>,
}

pub struct ApplicationServer {
    container: ApplicationContainer,
}

impl ApplicationServer {
    pub fn new(
        config: Arc<AppConfig>,
        auth_controller: Arc<AuthController>,
        acl_controller: Arc<AclController>,
        platform_controller: Arc<PlatformController>,
    ) -> Self {
        Self {
            container: ApplicationContainer {
                config,
                auth_controller,
                acl_controller,
                platform_controller,
            },
        }
    }

    pub fn build_router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let static_dir = self.container.config.static_dir();
        let app_assets = static_dir.join("app/assets");
        let app_images = static_dir.join("app/images");
        let app_fonts = static_dir.join("app/fonts");
        let global_images = static_dir.join("images");
        let global_fonts = static_dir.join("fonts");
        let global_css = static_dir.join("css");
        let global_js = static_dir.join("js");

        Router::new()
            // Auth endpoints
            .route(
                "/api/auth/login",
                post(|State(c): State<ApplicationContainer>, Json(body): Json<LoginPayload>| async move {
                    c.auth_controller.handle_login(body).await
                }),
            )
            .route(
                "/api/auth/me",
                get(|State(c): State<ApplicationContainer>, headers: HeaderMap| async move {
                    c.auth_controller.handle_me(&headers)
                }),
            )
            .route(
                "/api/auth/logout",
                post(|State(c): State<ApplicationContainer>| async move {
                    c.auth_controller.handle_logout()
                })
                .get(|State(c): State<ApplicationContainer>| async move {
                    c.auth_controller.handle_logout()
                }),
            )
            .route(
                "/api/auth/sso",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.auth_controller.handle_sso()
                }),
            )
            // ACL endpoint
            .route(
                "/api/v1/acl/check",
                get(|State(c): State<ApplicationContainer>, headers: HeaderMap, uri: Uri| async move {
                    c.acl_controller.handle_check(&headers, &uri)
                }),
            )
            // System endpoints
            .route(
                "/v3/api-docs",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.platform_controller.handle_api_docs()
                }),
            )
            .route(
                "/health",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.platform_controller.handle_health()
                }),
            )
            .route(
                "/actuator/health",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.platform_controller.handle_health()
                }),
            )
            // Root redirects
            .route("/", get(|| async { Redirect::to("/app/") }))
            .route("/login", get(|| async { Redirect::to("/app/") }))
            .route("/home", get(|| async { Redirect::to("/app/") }))
            .route("/inicio", get(|| async { Redirect::to("/app/") }))
            .route("/app", get(|| async { Redirect::to("/app/") }))
            .route(
                "/app/",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.platform_controller.handle_spa_index().await
                }),
            )
            // Static assets
            .nest_service("/app/assets", ServeDir::new(app_assets.clone()))
            .nest_service("/app/images", ServeDir::new(app_images.clone()))
            .nest_service("/app/fonts", ServeDir::new(app_fonts.clone()))
            .nest_service("/assets", ServeDir::new(app_assets))
            .nest_service("/images", ServeDir::new(global_images))
            .nest_service("/fonts", ServeDir::new(global_fonts))
            .nest_service("/css", ServeDir::new(global_css))
            .nest_service("/js", ServeDir::new(global_js))
            // SPA Subroute & global fallbacks
            .route(
                "/app/{*path}",
                get(|State(c): State<ApplicationContainer>| async move {
                    c.platform_controller.handle_spa_index().await
                }),
            )
            .fallback(|State(c): State<ApplicationContainer>| async move {
                c.platform_controller.handle_spa_index().await
            })
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(self.container.clone())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!(
            "{}:{}",
            self.container.config.server_address(),
            self.container.config.server_port()
        )
        .parse()?;

        info!("ApplicationServer listening on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let router = self.build_router();
        axum::serve(listener, router).await?;
        Ok(())
    }
}
