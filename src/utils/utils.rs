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

#[path = "../config.rs"]
mod config;

pub async fn setup() -> Result<(
    Provider<Http>,
    Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    AnvilInstance,
)> {
    let config = config::Config::load();

    let anvil_builder = match config.anvil_path {
        Some(path) => Anvil::at(path),
        None => Anvil::new(),
    };

    // Fork if necessary and then spawn
    let anvil_builder = match config.rpc_url {
        Some(rpc_url) => anvil_builder.fork(rpc_url),
        None => anvil_builder,
    };

    let anvil = anvil_builder.spawn();

    println!("Connecting to Ethereum node at: {}", anvil.endpoint());

    let provider = wallet::create_provider(anvil.endpoint().as_str());
    let chain_id = provider.get_chainid().await?;

    let client = wallet::create_signer(provider.clone(), &config.priv_key, chain_id.as_u64());

    Ok((provider, client, anvil))
}
