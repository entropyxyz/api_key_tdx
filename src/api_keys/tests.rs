use serial_test::serial;

use super::api::{DeployApiKeyInfo, TIME_BUFFER, check_stale, get_current_timestamp};
use crate::test_helpers::setup_client;
use entropy_protocol::sign_and_encrypt::{
    EncryptedSignedMessage, EncryptedSignedMessageErr, SignedMessage,
};
use sp_core::Pair;
use sp_keyring::{AccountKeyring, Sr25519Keyring};

#[tokio::test]
#[serial]
async fn test_deploy_api_key() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let box_url_and_key = ("127.0.0.1:3001".to_string(), app_state.x25519_public_key());
    let user_api_key_info = DeployApiKeyInfo {
        api_key: "test".to_string(),
        api_service: "test".to_string(),
        timestamp: get_current_timestamp().unwrap(),
    };

    let test_deploy_api_key_result =
        submit_transaction_request(box_url_and_key, user_api_key_info.clone(), one)
            .await
            .unwrap();
    assert_eq!(test_deploy_api_key_result.status(), 200);

    assert_eq!(
        app_state
            .read_from_api_keys(&(one.pair().public().0, user_api_key_info.clone().api_service))
            .unwrap()
            .unwrap(),
        user_api_key_info.api_service
    )
}

#[tokio::test]
async fn test_stale_check() {
    let result = check_stale(1, 1).await;
    assert!(result.is_ok());

    let result_server_larger = check_stale(1, 2).await;
    assert!(result_server_larger.is_ok());

    let result_user_larger = check_stale(2, 1).await;
    assert!(result_user_larger.is_ok());

    let fail_stale = check_stale(1, 2 + TIME_BUFFER).await.unwrap_err();
    assert_eq!(fail_stale.to_string(), "Message is too old".to_string());
}

pub async fn submit_transaction_request(
    box_url_and_key: (String, [u8; 32]),
    signature_request: DeployApiKeyInfo,
    keyring: Sr25519Keyring,
) -> std::result::Result<reqwest::Response, reqwest::Error> {
    let mock_client = reqwest::Client::new();
    let signed_message = EncryptedSignedMessage::new(
        &keyring.pair(),
        serde_json::to_vec(&signature_request.clone()).unwrap(),
        &box_url_and_key.1,
        &[],
    )
    .unwrap();

    let url = format!("http://{}/deploy-api-key", box_url_and_key.0.clone());
    mock_client
        .post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&signed_message).unwrap())
        .send()
        .await
}
