use clap::Parser;
use anyhow::anyhow;
use std::{net::SocketAddr, str::FromStr};
use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = StartupArgs::parse();
    let addr = SocketAddr::from_str(&args.box_url)
        .map_err(|_| anyhow!("Failed to parse threshold url"))?;
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|_| anyhow!("Unable to bind to given server address"))?;
    
    let mut routes = Router::new();
    // TODO: add loggings

    axum::serve(listener, routes.into_make_service()).await?;
    Ok(())  
}


#[derive(Parser, Debug, Clone)]
#[command(about, version)]
pub struct StartupArgs {
     /// Url to host threshold (axum) server on.
     #[arg(short = 'u', long = "box-url", required = false, default_value = "127.0.0.1:3001")]
     pub box_url: String,
}