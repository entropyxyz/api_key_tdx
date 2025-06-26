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
