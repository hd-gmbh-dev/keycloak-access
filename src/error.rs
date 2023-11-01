#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("json (de)serialization error {0:?}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("decode error {0:?}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("jwt error {0:?}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("http error {0:?}")]
    RequestError(#[from] reqwest::Error),
    #[error("unable to get public key from realm {0}")]
    NoPublicKey(String),
    #[error("invalid token")]
    InvalidToken,
}
