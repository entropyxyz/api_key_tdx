use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use entropy_protocol::{errors::ProtocolExecutionErr, sign_and_encrypt::EncryptedSignedMessageErr};

#[derive(Debug, Error)]
pub enum Err {
    #[error("Cannot get output from hasher in HKDF {0}")]
    Hkdf(hkdf::InvalidLength),
    #[error("mnemonic failure: {0:?}")]
    Mnemonic(String),
    #[error("Encryption or signing error: {0}")]
    EncryptionOrAuthentication(#[from] EncryptedSignedMessageErr),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Posion Mutex error: {0}")]
    PosionError(String),
    #[error("Message is too old")]
    StaleMessage,
    #[error("Time subtraction error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Http client: {0}")]
    HttpRequest(#[from] reqwest::Error),
}

impl From<hkdf::InvalidLength> for Err {
    fn from(invalid_length: hkdf::InvalidLength) -> Err {
        Err::Hkdf(invalid_length)
    }
}

impl IntoResponse for Err {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
