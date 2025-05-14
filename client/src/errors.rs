use entropy_protocol::sign_and_encrypt::EncryptedSignedMessageErr;
use thiserror::Error;

/// Error used by the API key service client methods
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Encryption or signing error: {0}")]
    EncryptionOrAuthentication(#[from] EncryptedSignedMessageErr),
    #[error("HTTP response {0}: {1}")]
    BadResponse(reqwest::StatusCode, String),
    #[error("Time subtraction error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Bad UTF8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Http client: {0}")]
    HttpRequest(#[from] reqwest::Error),
}
