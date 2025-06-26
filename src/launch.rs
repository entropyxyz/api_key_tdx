use crate::{attestation::create_quote, errors::Err};
use backoff::ExponentialBackoff;
use entropy_client::{
    chain_api::{
        entropy::{self, runtime_types::pallet_forest::module::ForestServerInfo},
        EntropyConfig,
    },
    request_attestation,
};
use sp_core::{crypto::Ss58Codec, sr25519, Pair};
use std::time::Duration;
use subxt::{
    backend::legacy::LegacyRpcMethods,
    blocks::ExtrinsicEvents,
    config::DefaultExtrinsicParamsBuilder as Params,
    tx::{Payload, Signer, TxStatus},
    utils::{AccountId32 as SubxtAccountId32, MultiSignature, H256},
    Config, OnlineClient,
};
use subxt_core::{storage::address::Address, utils::Yes};

/// Blocks a transaction is valid for
pub const MORTALITY_BLOCKS: u64 = 32;

/// Send a transaction to the Entropy chain
///
/// Optionally takes a nonce, otherwise it grabs the latest nonce from the chain
///
pub async fn submit_transaction<Call: Payload, S: Signer<EntropyConfig>>(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    signer: &S,
    call: &Call,
    nonce_option: Option<u32>,
) -> Result<ExtrinsicEvents<EntropyConfig>, Err> {
    let block_hash = rpc
        .chain_get_block_hash(None)
        .await?
        .ok_or(Err::BlockHash)?;

    let nonce = if let Some(nonce) = nonce_option {
        nonce
    } else {
        let nonce_call = entropy::apis()
            .account_nonce_api()
            .account_nonce(signer.account_id().clone());
        api.runtime_api().at(block_hash).call(nonce_call).await?
    };

    let tx_params = Params::new()
        .mortal(MORTALITY_BLOCKS)
        .nonce(nonce.into())
        .build();
    let mut tx = api
        .tx()
        .create_signed(call, signer, tx_params)
        .await?
        .submit_and_watch()
        .await?;

    while let Some(status) = tx.next().await {
        match status? {
            TxStatus::InBestBlock(tx_in_block) | TxStatus::InFinalizedBlock(tx_in_block) => {
                return Ok(tx_in_block.wait_for_success().await?);
            }
            TxStatus::Error { message }
            | TxStatus::Invalid { message }
            | TxStatus::Dropped { message } => {
                // Handle any errors:
                return Err(Err::BadEvent(message));
            }
            // Continue otherwise:
            _ => continue,
        };
    }
    Err(Err::NoEvent)
}

/// Convenience function to send a transaction to the Entropy chain giving a sr25519::Pair to sign with
pub async fn submit_transaction_with_pair<Call: Payload>(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    pair: &sr25519::Pair,
    call: &Call,
    nonce_option: Option<u32>,
) -> Result<ExtrinsicEvents<EntropyConfig>, Err> {
    let signer = Sr25519Signer::new(pair.clone());
    submit_transaction(api, rpc, &signer, call, nonce_option).await
}

/// Gets data from the Entropy chain
///
/// Optionally takes a block hash, otherwise the latest block hash from the chain is used
pub async fn query_chain<Addr>(
    api: &OnlineClient<EntropyConfig>,
    rpc: &LegacyRpcMethods<EntropyConfig>,
    storage_call: Addr,
    block_hash_option: Option<H256>,
) -> Result<Option<Addr::Target>, Err>
where
    Addr: Address<IsFetchable = Yes>,
{
    let block_hash = if let Some(block_hash) = block_hash_option {
        block_hash
    } else {
        rpc.chain_get_block_hash(None)
            .await?
            .ok_or(Err::BlockHash)?
    };

    let result = api.storage().at(block_hash).fetch(&storage_call).await?;

    Ok(result)
}

/// A wrapper around [sr25519::Pair] which implements [Signer]
/// This is needed because on wasm we cannot use the generic `subxt::tx::PairSigner`
#[derive(Clone)]
struct Sr25519Signer {
    account_id: <EntropyConfig as Config>::AccountId,
    pair: sr25519::Pair,
}

impl Sr25519Signer {
    /// Creates a new [`Sr25519Signer`] from an [`sr25519::Pair`].
    pub fn new(pair: sr25519::Pair) -> Self {
        Self {
            account_id: SubxtAccountId32(pair.public().0),
            pair,
        }
    }
}

impl Signer<EntropyConfig> for Sr25519Signer {
    fn account_id(&self) -> <EntropyConfig as Config>::AccountId {
        self.account_id.clone()
    }

    fn sign(&self, signer_payload: &[u8]) -> <EntropyConfig as Config>::Signature {
        MultiSignature::Sr25519(self.pair.sign(signer_payload).0)
    }
}

fn create_test_backoff() -> ExponentialBackoff {
    let mut backoff = ExponentialBackoff::default();
    backoff.max_elapsed_time = Some(Duration::from_secs(5));
    backoff.initial_interval = Duration::from_millis(50);
    backoff.max_interval = Duration::from_millis(500);
    backoff
}
