//! Simple client library for the API Key Service
pub mod errors;
pub use entropy_client::chain_api::entropy::runtime_types::pallet_forest::module::ForestServerInfo;

use entropy_api_key_service_shared::{DeleteApiKeyInfo, DeployApiKeyInfo, SendApiKeyMessage};
use entropy_client::{
    chain_api::{
        EntropyConfig,
        entropy::{self, runtime_types::pallet_parameters::SupportedCvmServices},
    },
    client::EncryptedSignedMessage,
    verify_tree_quote,
};
use errors::ClientError;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use sp_core::{Pair, sr25519};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::{OnlineClient, backend::legacy::LegacyRpcMethods, utils::AccountId32};

/// Client for API key service
pub struct ApiKeyServiceClient {
    /// Socket address or hostname of the api key service instance to use
    api_key_service_endpoint: String,
    /// X25519 public key of the api key service instance to use
    api_key_service_x25519_public_key: [u8; 32],
    /// Client for requests
    http_client: reqwest::Client,
    /// The user's keypair for authentication
    pair: sr25519::Pair,
}

impl ApiKeyServiceClient {
    /// Create a new client with given server details
    pub fn new(
        api_key_service_endpoint: String,
        api_key_service_x25519_public_key: [u8; 32],
        pair: sr25519::Pair,
    ) -> Self {
        Self {
            api_key_service_endpoint,
            api_key_service_x25519_public_key,
            http_client: reqwest::Client::new(),
            pair,
        }
    }

    /// Create a new client with given server details
    pub fn new_with_service_info(
        api_key_service_info: ForestServerInfo,
        pair: sr25519::Pair,
    ) -> Result<Self, ClientError> {
        Ok(Self {
            api_key_service_endpoint: String::from_utf8(api_key_service_info.endpoint)?,
            api_key_service_x25519_public_key: api_key_service_info.x25519_public_key,
            http_client: reqwest::Client::new(),
            pair,
        })
    }

    /// Create a new client selecting a server from the chain
    pub async fn select_from_chain(
        api: &OnlineClient<EntropyConfig>,
        rpc: &LegacyRpcMethods<EntropyConfig>,
        pair: sr25519::Pair,
    ) -> Result<Self, ClientError> {
        let api_key_servers = get_api_key_servers(api, rpc).await?;

        let mut rng = StdRng::from_seed(pair.public().0);
        let (api_key_service_account_id, api_key_service_info) =
            api_key_servers
                .choose(&mut rng)
                .ok_or(ClientError::NoAvailableApiKeyServices)?;

        // TODO derive Clone on ForestServerInfo so that this manual clone is not needed
        let api_key_service_info = ForestServerInfo {
            x25519_public_key: api_key_service_info.x25519_public_key.clone(),
            endpoint: api_key_service_info.endpoint.clone(),
            tdx_quote: api_key_service_info.tdx_quote.clone(),
        };

        verify_tree_quote(
            api,
            rpc,
            &api_key_service_info,
            api_key_service_account_id.0,
            SupportedCvmServices::ApiKeyService,
        )
        .await?;

        Ok(Self::new_with_service_info(api_key_service_info, pair)?)
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

    /// Deletes an API key
    pub async fn delete_api_key(&self, api_url: String) -> Result<(), ClientError> {
        let user_info = DeleteApiKeyInfo {
            api_url,
            timestamp: get_current_timestamp()?,
        };

        let request = serde_json::to_vec(&user_info)?;

        let response = self
            .send_http_request("/delete-secret".to_string(), request)
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
        http_headers: Vec<(String, String)>,
    ) -> Result<reqwest::Response, ClientError> {
        let request_body = match request.body() {
            Some(body) => String::from_utf8(body.as_bytes().unwrap_or_default().to_vec())?,
            None => String::new(),
        };
        let send_api_key_message = SendApiKeyMessage {
            request_body,
            http_verb: request.method().as_str().to_lowercase().to_string(),
            http_headers,
            api_url: request
                .url()
                .as_str()
                .to_string()
                .strip_suffix("/")
                .unwrap_or(request.url().as_str())
                .to_string(),
            timestamp: get_current_timestamp()?,
        };

        let request = serde_json::to_vec(&send_api_key_message)?;

        let response = self
            .send_http_request("/make-request".to_string(), request)
            .await?;

        Ok(response)
    }

    /// Internal helper to make a request to the service
    async fn send_http_request(
        &self,
        route: String,
        request: Vec<u8>,
    ) -> Result<reqwest::Response, ClientError> {
        let signed_message = EncryptedSignedMessage::new(
            &self.pair,
            request,
            &self.api_key_service_x25519_public_key,
            &[],
        )?;

        let full_url = format!("{}{}", self.api_key_service_endpoint.clone(), route);

        Ok(self
            .http_client
            .post(full_url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&signed_message)?)
            .send()
            .await?)
    }
}

/// Returns the current unix time in seconds
pub fn get_current_timestamp() -> Result<u64, ClientError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Get all available API key servers from the chain
pub async fn get_api_key_servers(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
) -> Result<Vec<(AccountId32, ForestServerInfo)>, ClientError> {
    let block_hash = rpc
        .chain_get_block_hash(None)
        .await?
        .ok_or(ClientError::BlockHash)?;
    let storage_address = entropy::storage().forest().trees_iter();
    let mut iter = api.storage().at(block_hash).iter(storage_address).await?;
    let mut servers = Vec::new();
    while let Some(Ok(kv)) = iter.next().await {
        let key: [u8; 32] = kv.key_bytes[kv.key_bytes.len() - 32..].try_into()?;
        servers.push((key.into(), kv.value))
    }
    Ok(servers)
}
