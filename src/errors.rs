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
    #[error("Error getting block hash")]
    BlockHash,
    #[error("Http verb is unsupported")]
    UnsupportedHttpVerb,
    #[error("Time subtraction error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Http client: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("Subxt: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("No event following extrinsic submission")]
    NoEvent,
    #[error("Could not sumbit transaction {0}")]
    BadEvent(String),
    #[error("Timed out trying to declare to chain")]
    TimedOut,
    #[error("Attestation: {0}")]
    Attestation(#[from] crate::attestation::errors::AttestationErr),
    #[error("Client: {0}")]
    Client(#[from] entropy_client::ClientError),
}

impl IntoResponse for Err {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
