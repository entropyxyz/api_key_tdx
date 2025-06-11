use crate::errors::Err;
use entropy_client::chain_api::{EntropyConfig, get_api, get_rpc};
use serde::Deserialize;
use sp_core::{Pair, crypto::AccountId32, sr25519};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use subxt::{
    OnlineClient, backend::legacy::LegacyRpcMethods, utils::AccountId32 as SubxtAccountId32,
};
use x25519_dalek::StaticSecret;

/// Application state struct which is cloned and made available to every axum HTTP route handler function
#[derive(Clone)]
pub struct AppState {
    /// Keypair for box id account
    pub pair: sr25519::Pair,
    /// Secret encryption key
    pub x25519_secret: StaticSecret,
    /// Configuation containing the chain endpoint
    pub configuration: Configuration,
    /// Storage for api keys
    pub api_keys: Arc<RwLock<HashMap<([u8; 32], String), String>>>,
}

impl AppState {
    /// Setup AppState with given secret keys
    pub fn new(
        configuration: Configuration,
        pair: sr25519::Pair,
        x25519_secret: StaticSecret,
    ) -> Self {
        Self {
            pair,
            x25519_secret,
            configuration,
            api_keys: Arc::new(RwLock::new(Default::default())),
        }
    }

    /// Convenience function to get chain api and rpc
    pub async fn get_api_rpc(
        &self,
    ) -> Result<(OnlineClient<EntropyConfig>, LegacyRpcMethods<EntropyConfig>), Err> {
        Ok((
            get_api(&self.configuration.endpoint).await?,
            get_rpc(&self.configuration.endpoint).await?,
        ))
    }

    /// Get the [AccountId32]
    pub fn account_id(&self) -> AccountId32 {
        AccountId32::new(self.pair.public().0)
    }

    /// Get the subxt account ID
    pub fn subxt_account_id(&self) -> SubxtAccountId32 {
        SubxtAccountId32(self.pair.public().0)
    }

    /// Get the x25519 public key
    pub fn x25519_public_key(&self) -> [u8; 32] {
        x25519_dalek::PublicKey::from(&self.x25519_secret).to_bytes()
    }

    /// Write to api key
    pub fn write_to_api_keys(&self, key: ([u8; 32], String), value: String) -> Result<(), Err> {
        self.clear_poisioned_api_keys();
        let mut api_keys = self
            .api_keys
            .write()
            .map_err(|e| Err::PosionError(e.to_string()))?;
        api_keys.insert(key, value);
        Ok(())
    }

    /// Delete from api key
    pub fn delete_from_api_keys(&self, key: ([u8; 32], String)) -> Result<(), Err> {
        self.clear_poisioned_api_keys();
        let mut api_keys = self
            .api_keys
            .write()
            .map_err(|e| Err::PosionError(e.to_string()))?;
        api_keys.remove(&key);
        Ok(())
    }

    /// Reads from api key will error if no value, call exists_in_request_limit to check
    pub fn read_from_api_keys(&self, key: &([u8; 32], String)) -> Result<Option<String>, Err> {
        self.clear_poisioned_api_keys();
        let api_keys = self
            .api_keys
            .read()
            .map_err(|e| Err::PosionError(e.to_string()))?;
        Ok(api_keys.get(key).cloned())
    }

    /// Clears a poisioned lock from request limit
    pub fn clear_poisioned_api_keys(&self) {
        if self.api_keys.is_poisoned() {
            self.api_keys.clear_poison()
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    pub endpoint: String,
}

impl Configuration {
    pub fn new(endpoint: String) -> Configuration {
        Configuration { endpoint }
    }
}
