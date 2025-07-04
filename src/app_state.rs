use crate::errors::Err;
use entropy_client::forest::TreeState;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Application state struct which is cloned and made available to every axum HTTP route handler function
#[derive(Clone)]
pub struct AppState {
    /// Tree state from entropy-client
    pub tree_state: TreeState,
    /// Storage for api keys
    pub api_keys: Arc<RwLock<HashMap<([u8; 32], String), String>>>,
}

impl AppState {
    /// Setup AppState with given secret keys
    pub fn new(tree_state: TreeState) -> Self {
        Self {
            tree_state,
            api_keys: Arc::new(RwLock::new(Default::default())),
        }
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
