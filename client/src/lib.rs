pub mod errors;

use entropy_client::{
    chain_api::entropy::runtime_types::pallet_outtie::module::OuttieServerInfo,
    client::EncryptedSignedMessage,
};
use errors::ClientError;
use serde::{Deserialize, Serialize};
use sp_core::sr25519;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployApiKeyInfo {
    pub api_key: String,
    pub api_url: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SendApiKeyMessage {
    pub request_body: String,
    pub http_verb: String,
    pub api_url: String,
    pub timestamp: u64,
}

/// Client for API key service
pub struct ApiKeyServiceClient {
    api_key_service_info: OuttieServerInfo,
    http_client: reqwest::Client,
    pair: sr25519::Pair,
}

impl ApiKeyServiceClient {
    pub fn new(api_key_service_info: OuttieServerInfo, pair: sr25519::Pair) -> Self {
        Self {
            api_key_service_info,
            http_client: reqwest::Client::new(),
            pair,
        }
    }

    /// Deploy an API key
    pub async fn deploy_api_key(
        &self,
        api_key: String,
        api_url: String,
    ) -> Result<(), ClientError> {
        let user_api_key_info = DeployApiKeyInfo {
            api_key,
            api_url,
            timestamp: get_current_timestamp()?,
        };

        let request = serde_json::to_vec(&user_api_key_info)?;

        let response = self
            .send_http_request("/deploy-api-key".to_string(), request)
            .await?;

        let response_status = response.status();
        match response_status {
            reqwest::StatusCode::OK => Ok(()),
            _ => Err(ClientError::BadResponse(
                response_status,
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    /// Make an HTTP request
    pub async fn make_request(
        &self,
        request: reqwest::Request,
    ) -> Result<reqwest::Response, ClientError> {
        let request_body = match request.body() {
            Some(body) => String::from_utf8(body.as_bytes().unwrap_or_default().to_vec())?,
            None => String::new(),
        };

        let send_api_key_message = SendApiKeyMessage {
            request_body,
            http_verb: request.method().as_str().to_lowercase().to_string(),
            api_url: request.url().as_str().to_string(),
            timestamp: get_current_timestamp()?,
        };

        let request = serde_json::to_vec(&send_api_key_message)?;

        let response = self
            .send_http_request("/make-request".to_string(), request)
            .await?;

        Ok(response)
    }

    async fn send_http_request(
        &self,
        route: String,
        request: Vec<u8>,
    ) -> Result<reqwest::Response, ClientError> {
        let signed_message = EncryptedSignedMessage::new(
            &self.pair,
            request,
            &self.api_key_service_info.x25519_public_key,
            &[],
        )?;

        let full_url = format!(
            "{}{}",
            String::from_utf8(self.api_key_service_info.endpoint.clone())?,
            route
        );

        Ok(self
            .http_client
            .post(full_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&signed_message)?)
            .send()
            .await?)
    }
}

pub fn get_current_timestamp() -> Result<u64, ClientError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
