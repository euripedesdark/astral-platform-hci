use crate::domain::{AuthSource, PlatformError};
use async_trait::async_trait;
use tokio_postgres::NoTls;
use tracing::{error, info};

#[async_trait]
pub trait AuthRouter: Send + Sync {
    async fn route(&self, username: &str) -> Result<AuthSource, PlatformError>;
}

pub struct DatabaseAuthRouter {
    pg_conn_str: String,
}

impl DatabaseAuthRouter {
    pub fn new(pg_conn_str: impl Into<String>) -> Self {
        Self {
            pg_conn_str: pg_conn_str.into(),
        }
    }
}

#[async_trait]
impl AuthRouter for DatabaseAuthRouter {
    async fn route(&self, username: &str) -> Result<AuthSource, PlatformError> {
        let sam = if let Some(idx) = username.find('@') {
            &username[..idx]
        } else {
            username
        };

        match tokio_postgres::connect(&self.pg_conn_str, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("DatabaseAuthRouter connection error: {}", e);
                    }
                });

                match client
                    .query_opt(
                        "SELECT source FROM bc_core_auth_source WHERE username = $1",
                        &[&sam],
                    )
                    .await
                {
                    Ok(Some(row)) => {
                        let src_str: String = row.get(0);
                        let source = AuthSource::from_str(&src_str);
                        info!("auth: '{}' routed to {:?}", username, source);
                        Ok(source)
                    }
                    Ok(None) => {
                        info!("auth: '{}' not in bc_core_auth_source, defaulting to AD", username);
                        Ok(AuthSource::ActiveDirectory)
                    }
                    Err(e) => {
                        error!("auth: routing query error ({}), defaulting to AD", e);
                        Ok(AuthSource::ActiveDirectory)
                    }
                }
            }
            Err(e) => {
                error!("auth: routing database unreachable ({}), defaulting to AD", e);
                Ok(AuthSource::ActiveDirectory)
            }
        }
    }
}
