use thiserror::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[derive(Debug, Error)]
pub enum Err {
    #[error("Cannot get output from hasher in HKDF {0}")]
    Hkdf(hkdf::InvalidLength),
    #[error("mnemonic failure: {0:?}")]
    Mnemonic(String),
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
