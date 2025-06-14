use crate::{AppState, attestation::get_pck, errors::Err};
use axum::{Json, extract::State};
use entropy_shared::{BoundedVecEncodedVerifyingKey, X25519PublicKey};
use serde::{Deserialize, Serialize};
use subxt::utils::AccountId32;

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
            match crate::attestation::get_measurement_value() {
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

/// Public signing and encryption keys associated with a server
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ServerPublicKeys {
    /// The account ID
    pub account_id: AccountId32,
    /// The public encryption key
    pub x25519_public_key: X25519PublicKey,
    /// The Provisioning Certification Key used in TDX quotes
    pub provisioning_certification_key: BoundedVecEncodedVerifyingKey,
}

/// Returns the server's public keys
#[tracing::instrument(skip_all)]
pub async fn info(State(app_state): State<AppState>) -> Result<Json<ServerPublicKeys>, Err> {
    Ok(Json(ServerPublicKeys {
        x25519_public_key: app_state.x25519_public_key(),
        account_id: app_state.subxt_account_id(),
        provisioning_certification_key: get_pck(app_state.subxt_account_id())?,
    }))
}
