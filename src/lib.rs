pub mod api_keys;
pub mod app_state;
pub mod attestation;
pub mod errors;
pub mod health;
pub mod launch;
pub mod node_info;

pub use app_state::AppState;
pub use entropy_api_key_service_shared::{DeleteApiKeyInfo, DeployApiKeyInfo, SendApiKeyMessage};

#[cfg(any(test, feature = "dev"))]
pub mod test_helpers;

#[cfg(test)]
pub mod tests;

use crate::{
    api_keys::api::{delete_secret, deploy_api_key, make_request},
    health::api::healthz,
    node_info::api::{info, version},
};
use axum::{
    routing::{get, post},
    Router,
};

pub fn app(app_state: AppState) -> Router {
    let routes = Router::new()
        .route("/healthz", get(healthz))
        .route("/deploy-api-key", post(deploy_api_key))
        .route("/delete-secret", post(delete_secret))
        .route("/make-request", post(make_request))
        .route("/version", get(version))
        .route("/info", get(info))
        .with_state(app_state);

    routes
}
