#![cfg(test)]
mod test_server;

use crate::{
    app,
    app_state::{AppState, Configuration},
    attestation::get_pck,
};
use entropy_api_key_service_client::ApiKeyServiceClient;
use entropy_client::chain_api::entropy::runtime_types::{
    bounded_collections::bounded_vec::BoundedVec, pallet_outtie::module::OuttieServerInfo,
};
use rand_core::OsRng;
use sp_core::{sr25519, Pair};
use sp_keyring::Sr25519Keyring;
use test_server::start_test_api_server;
use x25519_dalek::StaticSecret;

pub const DEFAULT_ENDPOINT: &str = "ws://localhost:9944";

pub async fn setup_client() -> AppState {
    let configuration = Configuration::new(DEFAULT_ENDPOINT.to_string());

    let (pair, _seed) = sr25519::Pair::generate();
    let x25519_secret = StaticSecret::random_from_rng(OsRng);

    let app_state = AppState::new(configuration, pair, x25519_secret);
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
pub fn make_test_client(app_state: &AppState, keyring: &Sr25519Keyring) -> ApiKeyServiceClient {
    ApiKeyServiceClient::new(
        OuttieServerInfo {
            endpoint: b"http://127.0.0.1:3001".to_vec(),
            x25519_public_key: app_state.x25519_public_key(),
            provisioning_certification_key: BoundedVec(
                get_pck(app_state.subxt_account_id()).unwrap().to_vec(),
            ),
        },
        keyring.pair(),
    )
}
