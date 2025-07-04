use crate::{AppState, errors::Err};
use axum::{Json, extract::State};
use entropy_client::forest::{ServerPublicKeys, get_node_info};
use entropy_shared::attestation::QuoteContext;
use serde::{Deserialize, Serialize};

/// Version information - the output of the `/version` HTTP endpoint
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VersionDetails {
    pub cargo_package_version: String,
    pub git_tag_commit: String,
    pub build: BuildDetails,
}

impl VersionDetails {
    fn new() -> Self {
        Self {
            cargo_package_version: env!("CARGO_PKG_VERSION").to_string(),
            git_tag_commit: env!("VERGEN_GIT_DESCRIBE").to_string(),
            build: BuildDetails::new(),
        }
    }
}

/// This lets us know this is a production build and gives us the measurement value of the release
/// image
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum BuildDetails {
    ProductionWithMeasurementValue(String),
    NonProduction,
}

impl BuildDetails {
    #[cfg(not(feature = "production"))]
    fn new() -> Self {
        BuildDetails::NonProduction
    }

    #[cfg(feature = "production")]
    fn new() -> Self {
        BuildDetails::ProductionWithMeasurementValue(
            match entropy_client::attestation::get_measurement_value() {
                Ok(value) => hex::encode(value),
                Err(error) => format!("Failed to get measurement value {:?}", error),
            },
        )
    }
}

/// Returns the version, commit data and build details
#[tracing::instrument]
pub async fn version() -> Json<VersionDetails> {
    Json(VersionDetails::new())
}

/// Returns the server's public keys
#[tracing::instrument(skip_all)]
pub async fn info(State(app_state): State<AppState>) -> Result<Json<ServerPublicKeys>, Err> {
    Ok(get_node_info(
        None,
        app_state.x25519_public_key(),
        app_state.subxt_account_id(),
        QuoteContext::Validate,
    )
    .await?)
}
