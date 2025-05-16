use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

/// Error on retrieving node info for the `/info` route handler
#[derive(Debug, Error)]
pub enum GetInfoError {
    // #[error("Could not get public keys: {0}")]
    // User(#[from] crate::user::errors::UserErr),
    #[error("Could not get Provisioning Certification Key: {0}")]
    Attestation(#[from] crate::attestation::errors::AttestationErr),
}

impl IntoResponse for GetInfoError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", format!("{self}"));
        let body = format!("{self}").into_bytes();
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
