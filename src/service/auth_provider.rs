use crate::domain::{AuthSource, Authority, Credentials, PlatformError, Principal};
use crate::service::auth_router::AuthRouter;
use async_trait::async_trait;
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use std::sync::Arc;
use tokio_postgres::NoTls;
use tracing::{error, info, warn};

#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Principal, PlatformError>;
}

pub struct LdapAuthProvider {
    url: String,
    domain: String,
    base_dn: String,
    admin_group: String,
}

impl LdapAuthProvider {
    pub fn new(
        url: impl Into<String>,
        domain: impl Into<String>,
        base_dn: impl Into<String>,
        admin_group: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            domain: domain.into(),
            base_dn: base_dn.into(),
            admin_group: admin_group.into(),
        }
    }

    fn extract_cn(dn: &str) -> Option<String> {
        for part in dn.split(',') {
            let part = part.trim();
            if part.len() > 3 && part[..3].eq_ignore_ascii_case("cn=") {
                return Some(part[3..].trim().to_string());
            }
        }
        None
    }
}

#[async_trait]
impl AuthenticationProvider for LdapAuthProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Principal, PlatformError> {
        if !credentials.is_valid() {
            return Err(PlatformError::MissingCredentials);
        }

        let sam = credentials.sam_account_name();
        let bind_dn = credentials.bind_dn(&self.domain);

        let settings = LdapConnSettings::new().set_no_tls_verify(true);

        // Try primary URL, then localhost fallback
        let conn_res = match LdapConnAsync::with_settings(settings.clone(), &self.url).await {
            Ok(c) => Ok(c),
            Err(e) => {
                warn!("Primary LDAP ({}) failed: {}. Falling back to ldaps://127.0.0.1:636", self.url, e);
                LdapConnAsync::with_settings(settings, "ldaps://127.0.0.1:636").await
            }
        };

        let (conn, mut ldap) = conn_res.map_err(|e| {
            error!("LDAP connection error: {}", e);
            PlatformError::ServiceUnavailable("LDAP service unreachable".into())
        })?;

        ldap3::drive!(conn);

        ldap.simple_bind(&bind_dn, credentials.password())
            .await
            .map_err(|e| {
                warn!("LDAP bind failed for '{}': {}", bind_dn, e);
                PlatformError::InvalidCredentials("Usuário ou senha inválidos.".into())
            })?;

        let filter = format!("(&(objectClass=user)(sAMAccountName={}))", sam);
        let mut authorities = vec![Authority::new("ROLE_ASTRAL_USER")];
        let mut groups = Vec::new();

        if let Ok(sr) = ldap
            .search(&self.base_dn, Scope::Subtree, &filter, vec!["memberOf", "cn"])
            .await
        {
            if let Ok((entries, _)) = sr.success() {
                for entry in entries {
                    let se = SearchEntry::construct(entry);
                    if let Some(members) = se.attrs.get("memberOf") {
                        let target = format!("cn={},", self.admin_group.to_lowercase());
                        for m in members {
                            if m.to_lowercase().contains(&target) && !authorities.iter().any(|a| a.authority == "ROLE_ASTRAL_ADMIN") {
                                authorities.push(Authority::new("ROLE_ASTRAL_ADMIN"));
                            }
                            if let Some(cn) = Self::extract_cn(m) {
                                if !groups.contains(&cn) {
                                    groups.push(cn);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Principal::new(
            credentials.username(),
            AuthSource::ActiveDirectory,
            authorities,
            groups,
        ))
    }
}

pub struct PostgresAuthProvider {
    host: String,
    port: u16,
    dbname: String,
    admin_role: String,
    source: AuthSource,
}

impl PostgresAuthProvider {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        dbname: impl Into<String>,
        admin_role: impl Into<String>,
        source: AuthSource,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            dbname: dbname.into(),
            admin_role: admin_role.into(),
            source,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for PostgresAuthProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Principal, PlatformError> {
        if !credentials.is_valid() {
            return Err(PlatformError::MissingCredentials);
        }

        let conn_str = format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, credentials.username(), credentials.password(), self.dbname
        );

        match tokio_postgres::connect(&conn_str, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        error!("PostgresAuthProvider connection background error: {}", e);
                    }
                });

                let mut authorities = vec![Authority::new("ROLE_ASTRAL_USER")];

                let is_admin = match client
                    .query_one(
                        "SELECT pg_has_role(current_user, $1, 'USAGE')",
                        &[&self.admin_role],
                    )
                    .await
                {
                    Ok(row) => row.get::<_, bool>(0),
                    Err(_) => false,
                };

                if is_admin {
                    authorities.push(Authority::new("ROLE_ASTRAL_ADMIN"));
                }

                Ok(Principal::new(
                    credentials.username(),
                    self.source.clone(),
                    authorities,
                    vec![],
                ))
            }
            Err(e) => {
                warn!("PostgreSQL authentication failed for '{}': {}", credentials.username(), e);
                Err(PlatformError::InvalidCredentials("Credenciais inválidas".into()))
            }
        }
    }
}

pub struct MultiSourceAuthProvider {
    router: Arc<dyn AuthRouter>,
    ad_provider: Arc<dyn AuthenticationProvider>,
    pg_provider: Arc<dyn AuthenticationProvider>,
    linux_provider: Arc<dyn AuthenticationProvider>,
}

impl MultiSourceAuthProvider {
    pub fn new(
        router: Arc<dyn AuthRouter>,
        ad_provider: Arc<dyn AuthenticationProvider>,
        pg_provider: Arc<dyn AuthenticationProvider>,
        linux_provider: Arc<dyn AuthenticationProvider>,
    ) -> Self {
        Self {
            router,
            ad_provider,
            pg_provider,
            linux_provider,
        }
    }
}

#[async_trait]
impl AuthenticationProvider for MultiSourceAuthProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<Principal, PlatformError> {
        let source = self.router.route(credentials.username()).await?;
        info!("MultiSourceAuthProvider: dispatching authentication for '{}' via {:?}", credentials.username(), source);
        match source {
            AuthSource::Postgres => self.pg_provider.authenticate(credentials).await,
            AuthSource::Linux => self.linux_provider.authenticate(credentials).await,
            AuthSource::ActiveDirectory => self.ad_provider.authenticate(credentials).await,
        }
    }
}
