use std::array::TryFromSliceError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttestationErr {
    #[error("Input must be 32 bytes: {0}")]
    TryFromSlice(#[from] TryFromSliceError),
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
    #[cfg(feature = "production")]
    #[error("Quote parse: {0}")]
    QuoteParse(#[from] tdx_quote::QuoteParseError),
}

impl IntoResponse for AttestationErr {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Error when checking quote measurement value
#[derive(Debug, Error)]
pub enum QuoteMeasurementErr {
    #[error("Substrate: {0}")]
    SubstrateClient(#[from] entropy_client::substrate::SubstrateError),
    #[error("Could not get accepted measurement values from on-chain parameters")]
    NoMeasurementValues,
    #[error("Quote verification: {0}")]
    Kv(#[from] entropy_shared::attestation::VerifyQuoteError),
}
