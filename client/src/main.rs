//! Simple CLI for testing the API Key Service
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use entropy_api_key_service_client::ApiKeyServiceClient;
use reqwest::{
    Body, Method, Request, Url,
    header::{HeaderName, HeaderValue},
};
use sp_core::{Pair, sr25519};

#[derive(Parser, Debug, Clone)]
#[command(about, version)]
pub struct Cli {
    /// Url of the API key service
    #[arg(
        short = 'u',
        long = "url",
        required = false,
        default_value = "127.0.0.1:3001"
    )]
    service_url: String,
    /// Hex encoded 32 byte x25519 Public key of the server
    #[arg(short, long)]
    service_x25519_public_key: String,
    /// Mnemonic or derivation path for keypair
    #[arg(short, long)]
    mnemonic: Option<String>,
    #[clap(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug, Clone)]
enum CliCommand {
    /// Deploy an API key to the service
    DeployApiKey {
        /// API key to deploy
        api_key: String,
        /// URL of the HTTP service associated with this key
        api_url: String,
    },
    /// Delete an API key from the service
    DeleteApiKey {
        /// URL of the HTTP service associated with this key
        api_url: String,
    },
    /// Make a request substituting `xxxREPLACE_MExxx` with your API key
    MakeRequest {
        /// The full URL for the desired request
        url: Url,
        /// The HTTP verb to use. Defaults to GET.
        #[arg(long)]
        verb: Option<Method>,
        /// The request body (UTF8 only)
        #[arg(long)]
        body: Option<String>,
        // The Headers to be sent to the request ex: "Authorization:Bearer xxx"
        #[arg(long, value_parser = parse_key_val)]
        header_request: Option<Vec<(String, String)>>,
        /// header given in the form "name:value". Can be given multiple times.
        #[arg(long)]
        header: Vec<String>,
    },
}

/// Parses the header request
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let mut parts = s.splitn(2, ':');
    match (parts.next(), parts.next()) {
        (Some(k), Some(v)) => Ok((k.to_string(), v.to_string())),
        _ => Err(format!("invalid KEY:VAL: {}", s)),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // For test perposes we can also get this from the user or the `/info` endpoint - but in
    // production this will not give any guarantees that an attestation has been made
    let x25519_public_key = hex::decode(args.service_x25519_public_key)?
        .try_into()
        .map_err(|_| anyhow!("x25519 public key must be 32 bytes"))?;

    let client = ApiKeyServiceClient::new(
        args.service_url,
        x25519_public_key,
        handle_mnemonic(args.mnemonic)?,
    );

    match args.command {
        CliCommand::DeployApiKey { api_key, api_url } => {
            client.deploy_api_key(api_key, api_url).await?;
            println!("Api key deployed successfully");
        }
        CliCommand::DeleteApiKey { api_url } => {
            client.delete_api_key(api_url).await?;
            println!("Api key deleted successfully");
        }
        CliCommand::MakeRequest {
            verb,
            url,
            body,
            header,
            header_request,
        } => {
            let mut request = Request::new(verb.unwrap_or(Method::GET), url);

            // Handle body
            if let Some(body_text) = body {
                let request_body = request.body_mut();
                *request_body = Some(Body::wrap(body_text));
            }

            // Insert given headers
            let header_map = request.headers_mut();
            for single_header in header {
                let mut single_header_kv = single_header.splitn(2, ':');
                let header_name = single_header_kv
                    .next()
                    .ok_or(anyhow!("Badly formed header"))?
                    .to_string();
                let header_value = single_header_kv
                    .next()
                    .ok_or(anyhow!("Badly formed header"))?
                    .to_string();
                header_map.insert(
                    HeaderName::from_bytes(header_name.as_bytes())?,
                    HeaderValue::from_str(&header_value)?,
                );
            }

            let response = client
                .make_request(request, header_request.unwrap_or(vec![]))
                .await?;
            println!("Response: {response:?}");
        }
    }

    Ok(())
}

/// Get an sr25519 from a mnemonic given as either option or environment variable
fn handle_mnemonic(mnemonic_option: Option<String>) -> anyhow::Result<sr25519::Pair> {
    let mnemonic = if let Some(mnemonic) = mnemonic_option {
        mnemonic
    } else {
        std::env::var("API_SERVICE_CLIENT_MNEMONIC")
            .map_err(|_| anyhow!("A mnemonic must be given either by the command line option or API_SERVICE_CLIENT_MNEMONIC environment variable"))?
    };
    Ok(<sr25519::Pair as Pair>::from_string(&mnemonic, None)?)
}
