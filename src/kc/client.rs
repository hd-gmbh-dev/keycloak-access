use std::sync::Arc;

use serde::de::DeserializeOwned;

use super::config::KeycloakConfig;
use crate::error::Error;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RealmInfo {
    #[serde(default)]
    pub realm: Option<Arc<str>>,
    #[serde(default)]
    pub public_key: Option<Arc<str>>,
}

struct Inner {
    url: Arc<str>,
    public_url: Arc<str>,
    client: reqwest::Client,
}

#[derive(Clone)]
pub struct Keycloak {
    inner: Arc<Inner>,
}

fn info_url(url: &str, realm: &str) -> String {
    format!("{url}/realms/{realm}")
}

impl Keycloak {
    pub fn new(config: &KeycloakConfig) -> Self {
        let url = config.address();
        let public_url = config.public_url();
        let client = reqwest::Client::new();
        Self {
            inner: Arc::new(Inner {
                url,
                public_url,
                client,
            }),
        }
    }

    pub fn public_url(&self) -> &str {
        self.inner.public_url.as_ref()
    }

    async fn fetch<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        Ok(self.inner.client.get(url).send().await?.json::<T>().await?)
    }

    pub async fn info(&self, realm: &str) -> Result<RealmInfo, Error> {
        self.fetch(&info_url(&self.inner.url, realm)).await
    }
}
