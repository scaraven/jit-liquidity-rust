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

    let client = wallet::create_signer(provider.clone(), priv_key, chain_id.as_u64());

    Ok((provider, client))
}

#[cfg(test)]
pub async fn test_setup() -> (
    Provider<Http>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
) {
    let config = crate::testconfig::TestConfig::load();
    let (provider, client) = setup(
        config
            .anvil_endpoint
            .expect("ANVIL_ENDPOINT does not exist")
            .as_str(),
        config.priv_key.as_str(),
    )
    .await
    .expect("UTILS_SETUP failed");

    return (provider, client);
}
