use std::sync::Arc;

pub struct KeycloakConfigBuilder {
    address: String,
    public_url: String,
}

impl Default for KeycloakConfigBuilder {
    fn default() -> Self {
        Self {
            address: "http://localhost:8080".to_string(),
            public_url: "https://id.keycloak.local".to_string(),
        }
    }
}

impl KeycloakConfigBuilder {
    pub fn address(mut self, address: String) -> Self {
        self.address = address;
        self
    }

    pub fn public_url(mut self, public_url: String) -> Self {
        self.public_url = public_url;
        self
    }

    pub fn build(self) -> KeycloakConfig {
        KeycloakConfig {
            keycloak_tls: None,
            keycloak_port: None,
            keycloak_host: None,
            keycloak_address: Some(Arc::from(self.address)),
            keycloak_public_url: Arc::from(self.public_url),
        }
    }
}

#[derive(Clone, serde::Deserialize, Debug)]
pub struct KeycloakConfig {
    keycloak_tls: Option<bool>,
    keycloak_port: Option<u16>,
    keycloak_host: Option<Arc<str>>,
    keycloak_address: Option<Arc<str>>,
    keycloak_public_url: Arc<str>,
}

impl KeycloakConfig {
    pub fn from_env() -> envy::Result<Self> {
        let mut cfg = envy::from_env::<KeycloakConfig>()?;
        if cfg.keycloak_address.is_none() {
            let host = cfg.keycloak_host.as_deref().unwrap_or("127.0.0.1");
            let port = cfg.keycloak_port.unwrap_or(8080);
            let protocol = if cfg.keycloak_tls.unwrap_or(false) {
                "https"
            } else {
                "http"
            };
            cfg.keycloak_address = Some(Arc::from(format!("{protocol}://{}:{}/", host, port)));
        }
        Ok(cfg)
    }

    pub fn address(&self) -> Arc<str> {
        self.keycloak_address.clone().unwrap()
    }

    pub fn public_url(&self) -> Arc<str> {
        self.keycloak_public_url.clone()
    }

    pub fn builder() -> KeycloakConfigBuilder {
        KeycloakConfigBuilder::default()
    }
}
