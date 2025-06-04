use serial_test::serial;

use super::api::{
    DeployApiKeyInfo, SendApiKeyMessage, TIME_BUFFER, check_stale, get_current_timestamp,
};
use crate::test_helpers::setup_client;
use entropy_protocol::sign_and_encrypt::EncryptedSignedMessage;
use sp_core::Pair;
use sp_keyring::{AccountKeyring, Sr25519Keyring};

#[tokio::test]
#[serial]
async fn test_deploy_api_key() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let box_url_and_key = (
        "http://127.0.0.1:3001/deploy-api-key".to_string(),
        app_state.x25519_public_key(),
    );
    let user_api_key_info = DeployApiKeyInfo {
        api_key: "test".to_string(),
        api_url_base: "test".to_string(),
        timestamp: get_current_timestamp().unwrap(),
    };

    let test_deploy_api_key_result = submit_transaction_request(
        box_url_and_key,
        serde_json::to_vec(&user_api_key_info.clone()).unwrap(),
        one,
    )
    .await
    .unwrap();
    assert_eq!(test_deploy_api_key_result.status(), 200);

    assert_eq!(
        app_state
            .read_from_api_keys(&(
                one.pair().public().0,
                user_api_key_info.clone().api_url_base
            ))
            .unwrap()
            .unwrap(),
        user_api_key_info.api_key
    )
}

#[tokio::test]
#[serial]
async fn test_make_request_get() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let box_url_and_key = (
        "http://127.0.0.1:3001/make-request".to_string(),
        app_state.x25519_public_key(),
    );
    let api_key =
        "live_MdrxblW1YgdnmuI3jVSJNLSqcdljuF3T2PDy26hWXk7fROoojH479EkhrDhYJIy4".to_string();
    let api_url = "https://api.thecatapi.com".to_string();
    let api_url_extra =
        "/v1/images/search?limit=1&breed_ids=beng&api_key=xxxREPLACE_MExxx".to_string();

    let _ = app_state.write_to_api_keys((one.pair().public().0, api_url.clone()), api_key);

    let user_make_request_info = SendApiKeyMessage {
        request_body: "test".to_string(),
        http_verb: "get".to_string(),
        api_url_base: api_url.clone(),
        api_url_extra,
        timestamp: get_current_timestamp().unwrap(),
    };

    let test_deploy_api_key_result = submit_transaction_request(
        box_url_and_key,
        serde_json::to_vec(&user_make_request_info.clone()).unwrap(),
        one,
    )
    .await
    .unwrap();
    assert_eq!(test_deploy_api_key_result.status(), 200);
    assert_eq!(
        &test_deploy_api_key_result.text().await.unwrap()[0..10],
        "[{\"breeds\""
    );
}

#[tokio::test]
#[serial]
async fn test_make_request_get_with_local_test_server() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let box_url_and_key = (
        "http://127.0.0.1:3001/make-request".to_string(),
        app_state.x25519_public_key(),
    );
    let api_key = "some-secret".to_string();
    let api_url = "http://127.0.0.1:3002".to_string();
    let api_url_extra = "/protected?api-key=xxxREPLACE_MExxx".to_string();
    let _ = app_state.write_to_api_keys((one.pair().public().0, api_url.clone()), api_key);

    let user_make_request_info = SendApiKeyMessage {
        request_body: "test".to_string(),
        http_verb: "get".to_string(),
        api_url_base: api_url.clone(),
        api_url_extra,
        timestamp: get_current_timestamp().unwrap(),
    };

    let test_deploy_api_key_result = submit_transaction_request(
        box_url_and_key,
        serde_json::to_vec(&user_make_request_info.clone()).unwrap(),
        one,
    )
    .await
    .unwrap();

    assert_eq!(test_deploy_api_key_result.status(), 200);
    assert_eq!(
        &test_deploy_api_key_result.text().await.unwrap(),
        "Success response"
    );
}

// TODO: negative test for deploy key and make request
// TODO: test post
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
    signature_request: Vec<u8>,
    keyring: Sr25519Keyring,
) -> std::result::Result<reqwest::Response, reqwest::Error> {
    let mock_client = reqwest::Client::new();
    let signed_message =
        EncryptedSignedMessage::new(&keyring.pair(), signature_request, &box_url_and_key.1, &[])
            .unwrap();

    mock_client
        .post(box_url_and_key.0.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&signed_message).unwrap())
        .send()
        .await
}
