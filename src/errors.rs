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
    #[error("Client: {0}")]
    Client(#[from] entropy_client::ClientError),
    #[error("Input must be 32 bytes: {0}")]
    TryFromSlice(#[from] std::array::TryFromSliceError),
    #[error("Substrate: {0}")]
    SubstrateClient(#[from] entropy_client::substrate::SubstrateError),
    #[cfg(feature = "production")]
    #[error("Quote generation: {0}")]
    QuoteGeneration(String),
    #[error("Cannot encode verifying key: {0}")]
    EncodeVerifyingKey(#[from] tdx_quote::VerifyingKeyError),
    #[error("Verifying key is not 33 bytes long")]
    BadVerifyingKeyLength,
    #[error("Attestation request: {0}")]
    AttestationRequest(#[from] entropy_client::errors::AttestationRequestError),
    #[error("Invalid or unknown context value given in query string")]
    UnknownContext,
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Unable to get hostname from given URL")]
    UrlHost,
    #[error("No api key for user url")]
    UrlEmpty,
    #[cfg(feature = "production")]
    #[error("Quote parse: {0}")]
    QuoteParse(#[from] tdx_quote::QuoteParseError),
    #[error("Invalid Header name {0}")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
    #[error("Invalid Header value {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("subxt rpc error: {0}")]
    SubxtRpcError(#[from] subxt::ext::subxt_rpcs::Error),
}

impl IntoResponse for Err {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
