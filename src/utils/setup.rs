use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    transports::{
        http::{
            reqwest::{self, Url},
            Http,
        },
        BoxTransport,
    },
};

use crate::config::testconfig;
use eyre::Result;

/// Setup a provider with an Anvil instance.
///
/// # Arguments
///
/// * `rpc_url` - Optional URL for RPC.
///
/// # Returns
///
/// * `Result<impl Provider<BoxTransport>>` - The provider.
pub async fn setup_provider_with_anvil(
    rpc_url: Option<Url>,
) -> Result<impl Provider<BoxTransport>> {
    let builder = ProviderBuilder::default().with_recommended_fillers();

    let provider = match rpc_url {
        Some(url) => builder.on_anvil_with_wallet_and_config(|anvil| anvil.fork(url)),
        None => builder.on_anvil_with_wallet(),
    };

    Ok(provider)
}

/// Setup a provider with a specified endpoint and private key.
///
/// # Arguments
///
/// * `endpoint` - The endpoint URL.
/// * `priv_key` - The private key signer.
///
/// # Returns
///
/// * `Result<impl Provider<Http<reqwest::Client>>>` - The provider.
pub async fn setup_provider(
    endpoint: Url,
    priv_key: PrivateKeySigner,
) -> Result<impl Provider<Http<reqwest::Client>>> {
    println!("Connecting to Ethereum node at: {}", endpoint);

    let wallet = EthereumWallet::from(priv_key);
    let provider = ProviderBuilder::default().wallet(wallet).on_http(endpoint);

    Ok(provider)
}

// Given a spun up Anvil instance, return a provider
pub async fn test_setup() -> (impl Provider<Http<reqwest::Client>>, Address) {
    use alloy::{
        network::{EthereumWallet, TransactionBuilder},
        primitives::U256,
        providers::ProviderBuilder,
        rpc::types::TransactionRequest,
        signers::local::PrivateKeySigner,
        transports::http::reqwest::Url,
    };

    const HUNDRED_ETH_DECIMALS: usize = 20;

    let config = testconfig::TestConfig::load();
    let signer = PrivateKeySigner::random();
    let address = signer.address();
    let wallet = EthereumWallet::new(signer);
    let funded_wallet = EthereumWallet::from(config.priv_key);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(Url::parse(&config.anvil_endpoint).unwrap());

    let funded_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(funded_wallet)
        .on_http(Url::parse(&config.anvil_endpoint).unwrap());

    // Fund our client using our existing funded_wallet
    let tx: TransactionRequest = TransactionRequest::default()
        .with_to(address)
        .with_value(U256::pow(U256::from(10), U256::from(HUNDRED_ETH_DECIMALS)));

    // Send it!
    let _ = funded_provider
        .send_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    (provider, address)
}
