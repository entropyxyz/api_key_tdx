use crate::{
    attestation::get_pck,
    node_info::api::{BuildDetails, ServerPublicKeys, VersionDetails},
    test_helpers::setup_client,
};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn version_test() {
    let _app_state = setup_client().await;
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:3001/version")
        .send()
        .await
        .unwrap();
    let version_details: VersionDetails =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(
        version_details,
        VersionDetails {
            cargo_package_version: "foo".to_string(), //env!("CARGO_PKG_VERSION").to_string(),
            git_tag_commit: "foo".to_string(),        //env!("VERGEN_GIT_DESCRIBE").to_string(),
            build: BuildDetails::NonProduction,
        }
    );
}

#[tokio::test]
#[serial]
async fn info_test() {
    let app_state = setup_client().await;
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:3001/v1/info")
        .send()
        .await
        .unwrap();
    let public_keys: ServerPublicKeys = response.json().await.unwrap();
    assert_eq!(
        public_keys,
        ServerPublicKeys {
            account_id: app_state.subxt_account_id(),
            x25519_public_key: app_state.x25519_public_key(),
            provisioning_certification_key: get_pck(app_state.subxt_account_id()).unwrap(),
        }
    );
}
