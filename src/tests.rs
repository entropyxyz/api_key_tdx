use serial_test::serial;
use entropy_testing_utils::{substrate_context::test_node_process, ChainSpecType};
use crate::{chain_api::{get_api, get_rpc}, launch::delcare_to_chain};
use sp_keyring::{AccountKeyring};

#[tokio::test]
#[serial]
async fn test_declare() {
    let alice = AccountKeyring::Alice;
    let cxt = test_node_process().await;
    let api = get_api(&cxt.ws_url).await.unwrap();
    let rpc = get_rpc(&cxt.ws_url).await.unwrap();

    // let result = delcare_to_chain();

}