use std::sync::{Arc, Mutex};

use alloy::{
    network::EthereumWallet, node_bindings::Anvil, providers::ProviderBuilder,
    rpc::types::Transaction, transports::http::reqwest::Url,
};
use eyre::Result;
use jit_liquidity_rust::{config, subscribe};

#[tokio::main]
async fn main() -> Result<()> {
    // Build a provider
    let config = config::Config::load();
    let wallet = EthereumWallet::from(config.signer);
    let client = config.address;

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_builtin(&config.rpc_url.clone().unwrap())
        .await?;

    // Generate flashbot signer
    let flashbot_signer = config.flashbot_signer.unwrap();
    let flashbot_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(flashbot_signer))
        .on_http(Url::parse("https://relay.flashbots.net").unwrap());

    // Setup public mempool provider items
    let ws_providr = subscribe::create_ws_provider(&config.rpc_url.clone().unwrap()).await?;
    let tx_buffer = Arc::new(Mutex::new(Vec::<Transaction>::new()));

    // Start monitoring pending transactions

    Ok(())
}
