use crate::{
    chain_api::{
        entropy::runtime_types::pallet_outtie::module::OuttieServerInfo, get_api, get_rpc,
    },
    launch::delcare_to_chain,
};
use entropy_testing_utils::{substrate_context::test_node_process};
use serial_test::serial;
use sp_keyring::AccountKeyring;
use sp_core::{Pair, sr25519};

#[tokio::test]
#[serial]
async fn test_declare() {
    let alice = AccountKeyring::Alice;
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();

    let server_info = OuttieServerInfo {
        endpoint: "test".into(),
        x25519_public_key: [0u8; 32],
    };

    let result = delcare_to_chain(&api, &rpc, server_info, &alice.pair(), None)
        .await;
    // Alice has funds should not time out and register to chain
    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_declare_times_out() {
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();
    let (pair, _seed) = sr25519::Pair::generate();

    let server_info = OuttieServerInfo {
        endpoint: "test".into(),
        x25519_public_key: [0u8; 32],
    };

    let result = delcare_to_chain(&api, &rpc, server_info, &pair, None)
        .await;
    // random pair does not have funds and  should time out
    assert_eq!(result.unwrap_err().to_string(), "Timed out trying to declare to chain");
}