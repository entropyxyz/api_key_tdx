use anyhow::anyhow;
use api_key_tdx::{
    app,
    app_state::{AppState, Configuration},
    launch::declare_to_chain,
};
use clap::Parser;
use rand_core::OsRng;
use sp_core::{sr25519, Pair};
use std::{net::SocketAddr, str::FromStr};
use x25519_dalek::StaticSecret;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = StartupArgs::parse();
    let configuration = Configuration::new(args.chain_endpoint);

    let (pair, _seed) = sr25519::Pair::generate();
    let x25519_secret = StaticSecret::random_from_rng(OsRng);
    let app_state = AppState::new(configuration, pair.clone(), x25519_secret);
    let (api, rpc) = app_state.get_api_rpc().await.expect("No chain connection");

    let _ = declare_to_chain(
        &api,
        &rpc,
        args.box_url.clone(),
        app_state.x25519_public_key(),
        &pair,
        None,
    )
    .await
    .map_err(|_| anyhow!("Unable declare self to chain"))?;
    // TODO add loki maybe
    let addr = SocketAddr::from_str(&args.box_url)
        .map_err(|_| anyhow!("Failed to parse threshold url"))?;
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|_| anyhow!("Unable to bind to given server address"))?;
    // TODO: add loggings
    axum::serve(listener, app(app_state).into_make_service()).await?;
    Ok(())
}

#[derive(Parser, Debug, Clone)]
#[command(about, version)]
pub struct StartupArgs {
    /// Url to host threshold (axum) server on.
    #[arg(
        short = 'u',
        long = "box-url",
        required = false,
        default_value = "127.0.0.1:3001"
    )]
    pub box_url: String,
    /// Websocket endpoint for the entropy blockchain.
    #[arg(
        short = 'c',
        long = "chain-endpoint",
        required = false,
        default_value = "ws://localhost:9944"
    )]
    pub chain_endpoint: String,
}
