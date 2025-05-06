use serde::Deserialize;
use sp_core::{crypto::AccountId32, sr25519};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
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
    pub api_keys: Arc<RwLock<HashMap<AccountId32, String>>>,
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
