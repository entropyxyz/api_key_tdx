use serial_test::serial;

use super::api::{TIME_BUFFER, check_stale};
use crate::test_helpers::{make_test_client, setup_client};
use entropy_protocol::sign_and_encrypt::EncryptedSignedMessage;
use reqwest::{Body, Method, Url};
use sp_core::Pair;
use sp_keyring::{AccountKeyring, Sr25519Keyring};

#[tokio::test]
#[serial]
async fn test_deploy_api_key() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;

    let api_url = "https://github.com/".to_string();
    let api_key = "test".to_string();
    let api_key_2 = "test_2".to_string();

    let client = make_test_client(&app_state, &one);

    client
        .deploy_api_key(api_key.clone(), api_url.clone())
        .await
        .unwrap();

    let api_url_mock = Url::parse(&api_url)
        .unwrap()
        .host_str()
        .unwrap()
        .to_string();

    assert_eq!(
        app_state
            .read_from_api_keys(&(one.pair().public().0, api_url_mock.clone()))
            .unwrap()
            .unwrap(),
        api_key
    );

    client
        .deploy_api_key(api_key_2.clone(), api_url.clone())
        .await
        .unwrap();

    assert_eq!(
        app_state
            .read_from_api_keys(&(one.pair().public().0, api_url_mock))
            .unwrap()
            .unwrap(),
        api_key_2
    );
}

#[tokio::test]
#[serial]
async fn test_make_request_get() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let api_url_string = "https://api.thecatapi.com/v1/images/search?limit=1&breed_ids=beng&api_key=xxxREPLACE_MExxx";
    let api_key =
        "live_MdrxblW1YgdnmuI3jVSJNLSqcdljuF3T2PDy26hWXk7fROoojH479EkhrDhYJIy4".to_string();
    let api_url = Url::parse(api_url_string).unwrap();
    let api_url_mock = api_url.host_str().unwrap().to_string();

    let _ = app_state.write_to_api_keys((one.pair().public().0, api_url_mock.to_string()), api_key);

    let client = make_test_client(&app_state, &one);

    let mut request = reqwest::Request::new(Method::GET, api_url);
    let body = request.body_mut();
    *body = Some(Body::wrap("test".to_string()));
    let response = client.make_request(request).await.unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(&response.text().await.unwrap()[0..10], "[{\"breeds\"");
}

#[tokio::test]
#[serial]
async fn test_make_request_get_with_local_test_server() {
    let app_state = setup_client().await;
    let one = AccountKeyring::One;
    let api_url_string = "http://127.0.0.1:3002/protected?api-key=xxxREPLACE_MExxx";
    let api_key = "some-secret".to_string();
    let api_url = Url::parse(api_url_string).unwrap();
    let api_url_mock = api_url.host_str().unwrap().to_string();
    let _ = app_state.write_to_api_keys((one.pair().public().0, api_url_mock), api_key);

    let client = make_test_client(&app_state, &one);

    let mut request = reqwest::Request::new(Method::GET, api_url);
    let body = request.body_mut();
    *body = Some(Body::wrap("test".to_string()));
    let response = client.make_request(request).await.unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(&response.text().await.unwrap(), "Success response");
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

// Allowing dead code here because we probably will use this for negative tests
#[allow(dead_code)]
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
