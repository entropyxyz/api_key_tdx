use crate::errors::Err;
use entropy_shared::attestation::{QuoteContext, QuoteInputData};
use subxt::utils::AccountId32;

/// Create a mock quote for testing on non-TDX hardware
#[cfg(not(feature = "production"))]
pub async fn create_quote(
    nonce: [u8; 32],
    account_id: AccountId32,
    x25519_public_key: [u8; 32],
) -> Result<Vec<u8>, Err> {
    let context = QuoteContext::ForestAddTree;
    use rand::{rngs::StdRng, SeedableRng};

    let mut seeder = StdRng::from_seed(account_id.0);

    // This is generated deterministically from account id
    let pck = tdx_quote::SigningKey::random(&mut seeder);

    // In the real thing this is the key used in the quoting enclave
    let signing_key = tdx_quote::SigningKey::random(&mut seeder);

    let input_data = QuoteInputData::new(account_id.clone(), x25519_public_key, nonce, context);

    let pck_encoded = tdx_quote::encode_verifying_key(pck.verifying_key())?.to_vec();
    let quote = tdx_quote::Quote::mock(signing_key.clone(), pck, input_data.0, pck_encoded)
        .as_bytes()
        .to_vec();
    Ok(quote)
}

/// Create a TDX quote in production
#[cfg(feature = "production")]
pub async fn create_quote(
    nonce: [u8; 32],
    account_id: AccountId32,
    x25519_public_key: [u8; 32],
) -> Result<Vec<u8>, Err> {
    let context = QuoteContext::ForestAddTree;

    let input_data = QuoteInputData::new(account_id, x25519_public_key, nonce, context);

    Ok(configfs_tsm::create_quote(input_data.0)
        .map_err(|e| Err::QuoteGeneration(format!("{:?}", e)))?)
}

/// Get the measurement value from this build by generating a quote.
/// This is used by the `/version` HTTP route to display measurement details of the current build.
#[cfg(feature = "production")]
pub fn get_measurement_value() -> Result<[u8; 32], Err> {
    let quote_raw = configfs_tsm::create_quote([0; 64])
        .map_err(|e| Err::QuoteGeneration(format!("{:?}", e)))?;
    let quote = tdx_quote::Quote::from_bytes(&quote_raw)?;
    Ok(entropy_shared::attestation::compute_quote_measurement(
        &quote,
    ))
}
