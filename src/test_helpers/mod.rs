#![cfg(test)]
mod test_server;

use crate::{
    app,
    app_state::{AppState},
};
use entropy_api_key_service_client::ApiKeyServiceClient;
use rand_core::OsRng;
use sp_core::{Pair, sr25519};
use sp_keyring::sr25519::Keyring;
use test_server::start_test_api_server;
use x25519_dalek::StaticSecret;
use entropy_client::forest::{TreeState, Configuration};

pub const DEFAULT_ENDPOINT: &str = "ws://localhost:9944";

pub async fn setup_client() -> AppState {
    let configuration = Configuration::new(DEFAULT_ENDPOINT.to_string());

    let (pair, _seed) = sr25519::Pair::generate();
    let x25519_secret = StaticSecret::random_from_rng(OsRng);

    let app_state = AppState::new(TreeState::new(configuration, pair, x25519_secret));
    let app = app(app_state.clone()).into_make_service();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Unable to bind to given server address.");
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Now start a server to test API calls with
    start_test_api_server().await;

    app_state
}

/// Returns a client for the test server
pub fn make_test_client(app_state: &AppState, keyring: &Keyring) -> ApiKeyServiceClient {
    ApiKeyServiceClient::new(
        "http://127.0.0.1:3001".to_string(),
        app_state.tree_state.x25519_public_key(),
        keyring.pair(),
    )
}
