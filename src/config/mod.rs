use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Clone, Debug)]
pub struct AppConfig {
    server_port: u16,
    server_address: String,
    pg_conn_str: String,
    pg_host: String,
    pg_port: u16,
    pg_dbname: String,
    pg_admin_role: String,
    ad_url: String,
    ad_domain: String,
    ad_base_dn: String,
    ad_admin_group: String,
    jwt_secret: String,
    jwt_ttl_seconds: i64,
    static_dir: PathBuf,
}

impl AppConfig {
    pub fn load() -> Self {
        if Path::new("/etc/astral/astral.env").exists() {
            let _ = dotenvy::from_path("/etc/astral/astral.env");
        }
        let _ = dotenvy::dotenv();

        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8082);

        let server_address = std::env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let user = std::env::var("ASTRAL_DB_USER").unwrap_or_else(|_| "astral".to_string());
        let password = std::env::var("ASTRAL_DB_PASSWORD").unwrap_or_else(|_| "205831nLNeN4Q6A32rN2W57g".to_string());
        let raw_url = std::env::var("ASTRAL_DB_URL")
            .or_else(|_| std::env::var("ASTRAL_AUTH_POSTGRES_URL"))
            .unwrap_or_else(|_| "127.0.0.1:5432/astral".to_string());

        let clean_url = raw_url
            .trim_start_matches("jdbc:postgresql://")
            .trim_start_matches("postgresql://");

        let (host_port, db) = clean_url.split_once('/').unwrap_or(("127.0.0.1:5432", "astral"));
        let (host, port_str) = host_port.split_once(':').unwrap_or((host_port, "5432"));
        let port: u16 = port_str.parse().unwrap_or(5432);

        let pg_conn_str = format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, user, password, db
        );

        let pg_admin_role = std::env::var("ASTRAL_AUTH_POSTGRES_ADMIN_ROLE")
            .unwrap_or_else(|_| "astral_admin".to_string());

        let ad_url = std::env::var("ASTRAL_AD_URL")
            .unwrap_or_else(|_| "ldaps://127.0.0.1:636".to_string());

        let ad_domain = std::env::var("ASTRAL_AD_DOMAIN")
            .unwrap_or_else(|_| "srvcloud.cloud".to_string());

        let ad_base_dn = std::env::var("ASTRAL_AD_BASE_DN")
            .unwrap_or_else(|_| "DC=srvcloud,DC=cloud".to_string());

        let ad_admin_group = std::env::var("ASTRAL_AD_ADMIN_GROUP")
            .unwrap_or_else(|_| "Domain Admins".to_string());

        let jwt_secret = std::env::var("ASTRAL_JWT_SECRET")
            .unwrap_or_else(|_| "dd1f4702cbb4d061075ff7c58c63e3f48a214bf6117e4453ad4ee20b2f48e3ae".to_string());

        let jwt_ttl_seconds = 28800; // 8 hours

        let candidates = [
            PathBuf::from("/opt/astral-platform/static"),
            PathBuf::from("./static"),
            PathBuf::from("./src/main/resources/static"),
        ];

        let mut static_dir = PathBuf::from("/opt/astral-platform/static");
        for candidate in &candidates {
            if candidate.join("app").exists() || candidate.join("app/index.html").exists() {
                static_dir = candidate.clone();
                break;
            }
        }

        info!("AppConfig initialized: port={}, host={}, ad_url={}, static_dir={:?}",
            server_port, host, ad_url, static_dir);

        Self {
            server_port,
            server_address,
            pg_conn_str,
            pg_host: host.to_string(),
            pg_port: port,
            pg_dbname: db.to_string(),
            pg_admin_role,
            ad_url,
            ad_domain,
            ad_base_dn,
            ad_admin_group,
            jwt_secret,
            jwt_ttl_seconds,
            static_dir,
        }
    }

    pub fn server_port(&self) -> u16 { self.server_port }
    pub fn server_address(&self) -> &str { &self.server_address }
    pub fn pg_conn_str(&self) -> &str { &self.pg_conn_str }
    pub fn pg_host(&self) -> &str { &self.pg_host }
    pub fn pg_port(&self) -> u16 { self.pg_port }
    pub fn pg_dbname(&self) -> &str { &self.pg_dbname }
    pub fn pg_admin_role(&self) -> &str { &self.pg_admin_role }
    pub fn ad_url(&self) -> &str { &self.ad_url }
    pub fn ad_domain(&self) -> &str { &self.ad_domain }
    pub fn ad_base_dn(&self) -> &str { &self.ad_base_dn }
    pub fn ad_admin_group(&self) -> &str { &self.ad_admin_group }
    pub fn jwt_secret(&self) -> &str { &self.jwt_secret }
    pub fn jwt_ttl_seconds(&self) -> i64 { self.jwt_ttl_seconds }
    pub fn static_dir(&self) -> &Path { &self.static_dir }
}
