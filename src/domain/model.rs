use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthSource {
    ActiveDirectory,
    Postgres,
    Linux,
}

impl AuthSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveDirectory => "AD",
            Self::Postgres => "POSTGRES",
            Self::Linux => "LINUX",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "POSTGRES" => Self::Postgres,
            "LINUX" => Self::Linux,
            _ => Self::ActiveDirectory,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authority {
    pub authority: String,
}

impl Authority {
    pub fn new(role: impl Into<String>) -> Self {
        Self { authority: role.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    username: String,
    source: AuthSource,
    authorities: Vec<Authority>,
    groups: Vec<String>,
}

impl Principal {
    pub fn new(
        username: impl Into<String>,
        source: AuthSource,
        authorities: Vec<Authority>,
        groups: Vec<String>,
    ) -> Self {
        Self {
            username: username.into(),
            source,
            authorities,
            groups,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn source(&self) -> &AuthSource {
        &self.source
    }

    pub fn authorities(&self) -> &[Authority] {
        &self.authorities
    }

    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.authorities.iter().any(|a| a.authority.eq_ignore_ascii_case(role))
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("ROLE_ASTRAL_ADMIN")
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn is_valid(&self) -> bool {
        !self.username.trim().is_empty() && !self.password.is_empty()
    }

    pub fn sam_account_name(&self) -> &str {
        if let Some(idx) = self.username.find('@') {
            &self.username[..idx]
        } else {
            &self.username
        }
    }

    pub fn bind_dn(&self, default_domain: &str) -> String {
        if self.username.contains('@') {
            self.username.clone()
        } else {
            format!("{}@{}", self.username, default_domain)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclDecision {
    Allow,
    Deny,
    Unauthorized,
}
