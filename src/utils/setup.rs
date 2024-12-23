use std::sync::Arc;

use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::LocalWallet,
    utils::{Anvil, AnvilInstance},
};
use eyre::Result;

#[path = "wallet.rs"]
mod wallet;

pub async fn setup_anvil(anvil_path: Option<&str>, rpc_url: Option<&str>) -> Result<AnvilInstance> {
    let anvil_builder = match anvil_path {
        Some(path) => Anvil::at(path),
        None => Anvil::new(),
    };

    // Fork if necessary and then spawn
    let anvil_builder = match rpc_url {
        Some(rpc_url) => anvil_builder.fork(rpc_url),
        None => anvil_builder,
    };

    let anvil = anvil_builder.spawn();

    Ok(anvil)
}

pub async fn setup(
    endpoint: &str,
    priv_key: &str,
) -> Result<(
    Provider<Http>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
)> {
    println!("Connecting to Ethereum node at: {}", endpoint);

    let provider = wallet::create_provider(endpoint);
    let chain_id = provider.get_chainid().await?;
    let wallet = wallet::setup_wallet(priv_key, chain_id.as_u64());

    let client = wallet::create_signer(provider.clone(), wallet);

    Ok((provider, client))
}

#[cfg(test)]
pub async fn setup_with_wallet(
    endpoint: &str,
    wallet: LocalWallet,
) -> Result<(
    Provider<Http>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
)> {
    println!("Connecting to Ethereum node at: {}", endpoint);

    let provider = wallet::create_provider(endpoint);

    let client = wallet::create_signer(provider.clone(), wallet);

    Ok((provider, client))
}

#[cfg(test)]
pub async fn test_setup() -> (
    Provider<Http>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) {
    use ethers::{
        core::rand::thread_rng,
        types::{TransactionRequest, U256},
    };

    const HUNDRED_ETH_DECIMALS: usize = 20;

    let config = crate::testconfig::TestConfig::load();
    let wallet = LocalWallet::new(&mut thread_rng());
    let funded_wallet = config.priv_key.parse::<LocalWallet>().unwrap();

    let (provider, client) = setup_with_wallet(
        config
            .anvil_endpoint
            .expect("ANVIL_ENDPOINT does not exist")
            .as_str(),
        wallet,
    )
    .await
    .expect("UTILS_SETUP failed");

    let funded_client = SignerMiddleware::new(provider.clone(), funded_wallet);

    // Fund our client using our existing funded_wallet
    let tx = TransactionRequest::new()
        .to(client.address())
        .value(U256::exp10(HUNDRED_ETH_DECIMALS));

    // send it!
    let pending_tx = funded_client.send_transaction(tx, None).await.unwrap();

    // get the mined tx
    let _ = pending_tx
        .await
        .unwrap()
        .ok_or_else(|| eyre::format_err!("ETH TRANSFER FAILED"))
        .unwrap();
    return (provider, client);
}
