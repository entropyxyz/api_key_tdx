use crate::launch::delcare_to_chain;
use entropy_api_key_service_client::get_api_key_servers;
use entropy_client::chain_api::{
    entropy::runtime_types::pallet_outtie::module::JoiningOuttieServerInfo, get_api, get_rpc,
};
use entropy_testing_utils::substrate_context::test_node_process;
use serial_test::serial;
use sp_core::{Pair, sr25519};
use sp_keyring::AccountKeyring;

#[tokio::test]
#[serial]
async fn test_declare() {
    let alice = AccountKeyring::Alice;
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();

    let endpoint: Vec<u8> = "test".into();
    let x25519_public_key = [0; 32];
    let server_info = JoiningOuttieServerInfo {
        endpoint: endpoint.clone(),
        x25519_public_key: x25519_public_key.clone(),
    };

    let result = delcare_to_chain(&api, &rpc, server_info, &alice.pair(), None).await;
    // Alice has funds should not time out and register to chain
    assert!(result.is_ok());

    let servers = get_api_key_servers(&api, &rpc).await.unwrap();
    let (account_id, server) = servers.iter().next().unwrap();
    assert_eq!(account_id.0, alice.pair().public().0);
    assert_eq!(endpoint, server.endpoint);
    assert_eq!(x25519_public_key, server.x25519_public_key);
}

#[tokio::test]
#[serial]
async fn test_declare_times_out() {
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();
    let (pair, _seed) = sr25519::Pair::generate();

    let server_info = JoiningOuttieServerInfo {
        endpoint: "test".into(),
        x25519_public_key: [0u8; 32],
    };

    let result = delcare_to_chain(&api, &rpc, server_info, &pair, None).await;
    // Random pair does not have funds and should give an error
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Inability to pay some fees (e.g. account balance too low)")
    );
}
