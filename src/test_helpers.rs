#![cfg(test)]
use crate::{
    app,
    app_state::{AppState, Configuration},
};
use rand_core::OsRng;
use sp_core::{Pair, sr25519};
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

    app_state
}
