//! Shared types used by the API Key Service server and client
use serde::{Deserialize, Serialize};

/// The placeholder which will be replaced with your API key if given in request headers or body
pub const API_KEY_PLACEHOLDER: &str = "xxxREPLACE_MExxx";

/// Request payload for the `/deploy-api-key` HTTP route
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployApiKeyInfo {
    /// The secret API key to be deployed
    pub api_key: String,
    /// URL of the service to use it with
    pub api_url: String,
    /// Current unix time in seconds
    pub timestamp: u64,
}

/// Request payload for the `/delete-secret` HTTP route
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeleteApiKeyInfo {
    /// URL of the service to use it with
    pub api_url: String,
    /// Current unix time in seconds
    pub timestamp: u64,
}

/// Request payload for the `/make-request` HTTP route
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SendApiKeyMessage {
    /// Body of the HTTP request
    pub request_body: String,
    /// The HTTP verb to use
    pub http_verb: String,
    /// The HTTP headers to use
    pub http_headers: Vec<(String, String)>,
    /// The URL for the HTTP request
    pub api_url: String,
    /// Current unix time in seconds
    pub timestamp: u64,
}
