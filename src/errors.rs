use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use entropy_protocol::sign_and_encrypt::EncryptedSignedMessageErr;

#[derive(Debug, Error)]
pub enum Err {
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
    #[error("Http verb is unsupported")]
    UnsupportedHttpVerb,
    #[error("Time subtraction error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Http client: {0}")]
    HttpRequest(#[from] reqwest::Error),
}

impl IntoResponse for Err {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
