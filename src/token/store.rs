use std::collections::BTreeMap;
use std::sync::Arc;

use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
use futures_locks::RwLock;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;

use super::decoder::LogoutClaims;
use super::decoder::PartialClaims;
use crate::error::Error;
use crate::kc::client::Keycloak;
use crate::token::decoder::Claims;
use crate::token::decoder::JwtDecoder;

pub struct JwtStore {
    keycloak: Arc<Keycloak>,
    keys: Arc<RwLock<BTreeMap<String, JwtDecoder>>>,
}

impl JwtStore {
    pub fn new(keycloak: Arc<Keycloak>) -> Self {
        Self {
            keycloak,
            keys: Default::default(),
        }
    }

    async fn get_decoder_from_realm(
        &self,
        realm: &str,
        header: Header,
    ) -> Result<JwtDecoder, Error> {
        let info = self.keycloak.info(realm).await?;
        let public_key = info
            .public_key
            .ok_or_else(|| Error::NoPublicKey(realm.to_owned()))?;
        match (header.alg, header.kid) {
            (Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512, Some(kid)) => {
                Ok(JwtDecoder::new(header.alg, kid, &public_key)?)
            }
            _ => Err(Error::InvalidToken),
        }
    }

    async fn get_decoder_from_partial_claims(&self, token: &str) -> Result<JwtDecoder, Error> {
        let token_header = jsonwebtoken::decode_header(token)?;
        let mut iter = token.split('.');
        if let Some(payload) = iter.nth(1) {
            let payload = URL_SAFE_NO_PAD.decode(payload)?;
            let partial_claims = serde_json::from_slice::<PartialClaims>(&payload)?;
            let public_url = self.keycloak.public_url();
            let issuer_url = &partial_claims.iss[0..public_url.len()];
            if partial_claims.iss.len() > public_url.len() && public_url == issuer_url {
                let s = partial_claims.iss.replace(self.keycloak.public_url(), "");
                let mut u = s.rsplit('/');
                let realm = u.next().ok_or(Error::InvalidToken)?;
                return self.get_decoder_from_realm(realm, token_header).await;
            } else {
                return Err(Error::InvalidToken);
            }
        }
        Err(Error::InvalidToken)
    }
    pub async fn decode(&self, token: &str) -> Result<Claims, Error> {
        let token_header = jsonwebtoken::decode_header(token)?;
        let kid = token_header.kid.as_ref().ok_or(Error::InvalidToken)?;
        {
            if let Some(key) = self.keys.read().await.get(kid) {
                return key.decode(token);
            }
        }
        let jwt = self.get_decoder_from_partial_claims(token).await?;
        let claims = jwt.decode(token)?;
        self.keys.write().await.insert(jwt.kid.clone(), jwt);
        Ok(claims)
    }

    pub async fn decode_logout_token(&self, token: &str) -> Result<LogoutClaims, Error> {
        let token_header = jsonwebtoken::decode_header(token)?;
        let kid = token_header.kid.as_ref().ok_or(Error::InvalidToken)?;
        {
            if let Some(key) = self.keys.read().await.get(kid) {
                return key.decode_logout_token(token);
            }
        }
        let jwt = self.get_decoder_from_partial_claims(token).await?;
        let logout_claims = jwt.decode_logout_token(token)?;
        self.keys.write().await.insert(jwt.kid.clone(), jwt);
        Ok(logout_claims)
    }
}
