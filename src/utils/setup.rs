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

pub async fn setup_provider_with_anvil(rpc_url: Option<Url>) -> impl Provider<BoxTransport> {
    let builder = ProviderBuilder::default().with_recommended_fillers();

    match rpc_url {
        Some(url) => builder.on_anvil_with_wallet_and_config(|anvil| anvil.fork(url)),
        None => builder.on_anvil_with_wallet(),
    }
}

pub async fn setup_provider(
    endpoint: Url,
    priv_key: PrivateKeySigner,
) -> impl Provider<Http<reqwest::Client>> {
    println!("Connecting to Ethereum node at: {}", endpoint);

    let wallet = EthereumWallet::from(priv_key);
    ProviderBuilder::default().wallet(wallet).on_http(endpoint)
}

// Given a spun up Anvil instance, return a provider
#[cfg(test)]
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

    let config = crate::testconfig::TestConfig::load();
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
