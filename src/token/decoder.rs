use std::collections::HashSet;
use std::sync::Arc;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub account: RealmAccess,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<Arc<str>>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PartialClaims {
    pub iss: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: i64,
    pub iat: i64,
    pub auth_time: Option<i64>,
    pub jti: String,
    pub iss: String,
    pub aud: serde_json::Value,
    pub sub: Arc<str>,
    pub typ: String,
    pub azp: String,
    pub session_state: String,
    pub acr: String,
    #[serde(rename = "allowed-origins")]
    pub allowed_origins: Vec<Arc<str>>,
    pub realm_access: RealmAccess,
    pub resource_access: ResourceAccess,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub sid: String,
    pub email_verified: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub preferred_username: String,
    pub given_name: String,
    #[serde(default)]
    pub family_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(skip)]
    pub is_api_test: bool,
}

impl Default for Claims {
    fn default() -> Self {
        Self {
            exp: 0,
            iat: 0,
            auth_time: None,
            jti: "".to_string(),
            iss: "".to_string(),
            is_api_test: true,
            sub: Arc::from("user-id"),
            typ: "".to_string(),
            azp: "".to_string(),
            session_state: "".to_string(),
            acr: "".to_string(),
            allowed_origins: vec![],
            realm_access: RealmAccess { roles: vec![] },
            resource_access: ResourceAccess {
                account: RealmAccess { roles: vec![] },
            },
            scope: "".to_string(),
            sid: "".to_string(),
            email_verified: false,
            name: "".to_string(),
            preferred_username: "".to_string(),
            given_name: "".to_string(),
            family_name: "".to_string(),
            aud: Default::default(),
            email: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutClaims {
    pub iat: i64,
    pub jti: String,
    pub iss: String,
    pub aud: serde_json::Value,
    pub sub: String,
    pub typ: String,
    pub sid: String,
}

#[derive(Clone)]
pub struct JwtDecoder {
    pub kid: String,
    validation: Validation,
    logout_validation: Validation,
    decoding_key: DecodingKey,
}

impl JwtDecoder {
    pub fn new(alg: Algorithm, kid: String, public_key: &str) -> Result<Self, Error> {
        let mut validation = Validation::new(alg);
        validation.set_audience(&["account"]);
        let mut logout_validation = Validation::new(alg);
        logout_validation.validate_exp = false;
        logout_validation.required_spec_claims = HashSet::new();
        logout_validation
            .required_spec_claims
            .insert("sub".to_string());
        logout_validation
            .required_spec_claims
            .insert("iss".to_string());
        logout_validation
            .required_spec_claims
            .insert("aud".to_string());
        Ok(Self {
            kid,
            validation,
            logout_validation,
            decoding_key: DecodingKey::from_rsa_pem(
                format!("-----BEGIN PUBLIC KEY-----\n{public_key}\n-----END PUBLIC KEY-----")
                    .as_bytes(),
            )?,
        })
    }
    pub fn decode(&self, token: &str) -> Result<Claims, Error> {
        let result =
            decode::<Claims>(token, &self.decoding_key, &self.validation).map_err(|e| {
                log::error!("{e:#?}");
                e
            })?;
        Ok(result.claims)
    }
    pub fn decode_logout_token(&self, token: &str) -> Result<LogoutClaims, Error> {
        let result = decode::<LogoutClaims>(token, &self.decoding_key, &self.logout_validation)
            .map_err(|e| {
                log::error!("{e:#?}");
                e
            })?;
        Ok(result.claims)
    }
}
