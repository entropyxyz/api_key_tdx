use crate::launch::declare_to_chain;
use entropy_api_key_service_client::get_api_key_servers;
use entropy_client::{
    chain_api::{
        entropy::runtime_types::pallet_parameters::SupportedCvmServices, get_api, get_rpc,
    },
    verify_tree_quote,
};
use entropy_testing_utils::substrate_context::test_node_process;
use serial_test::serial;
use sp_core::{sr25519, Pair};
use sp_keyring::sr25519::Keyring;

#[tokio::test]
#[serial]
async fn test_declare() {
    let alice = Keyring::Alice;
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();

    let endpoint = "test".to_string();
    let x25519_public_key = [0; 32];

    let result = declare_to_chain(
        &api,
        &rpc,
        endpoint.clone(),
        x25519_public_key,
        &alice.pair(),
        None,
    )
    .await;
    // Alice has funds should not time out and register to chain
    assert!(result.is_ok());

    let servers = get_api_key_servers(&api, &rpc).await.unwrap();
    let (account_id, server) = servers.iter().next().unwrap();
    assert_eq!(account_id.0, alice.pair().public().0);
    assert_eq!(
        <std::string::String as Into<Vec<u8>>>::into(endpoint),
        server.endpoint
    );
    assert_eq!(x25519_public_key, server.x25519_public_key);

    // Check the quote
    verify_tree_quote(
        &api,
        &rpc,
        &server,
        account_id.0,
        SupportedCvmServices::ApiKeyService,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[serial]
async fn test_declare_times_out() {
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();
    let (pair, _seed) = sr25519::Pair::generate();

    let endpoint = "test".to_string();
    let x25519_public_key = [0; 32];

    let result = declare_to_chain(&api, &rpc, endpoint, x25519_public_key, &pair, None).await;

    // Random pair does not have funds and should give an error
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("User error: Invalid Transaction (1010)"));
}
