use std::sync::{atomic, Arc};

use alloy::{
    network::EthereumWallet, providers::ProviderBuilder, rpc::types::Transaction,
    transports::http::reqwest::Url,
};
use eyre::Result;
use jit_liquidity_rust::{
    addresses, config,
    subscribe::{self, ShutdownConfig},
    subscribe_filter,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    // Constants
    let uniswap_manager =
        addresses::get_address("0xC36442b4a4522E871399CD717aBDD847Ab11FE88").unwrap();

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
    let ws_provider =
        Arc::new(subscribe::create_ws_provider(&config.rpc_url.clone().unwrap()).await?);
    let tx_buffer = Arc::new(Mutex::new(Vec::<Transaction>::new()));

    // Filter for shallow filter
    let filter = subscribe_filter::ShallowFilterType::Recipient(uniswap_manager);

    // Shutdown mechanism
    let shutdown_flag = ShutdownConfig {
        flag: Arc::new(atomic::AtomicBool::new(false)),
        result: Arc::new(atomic::AtomicBool::new(false)),
    };

    // Start monitoring pending transactions
    let handle =
        subscribe::subscribe_to_pending(ws_provider, filter, tx_buffer.clone(), shutdown_flag)
            .await?;

    // Spawn a tokio process each time a new transaction is detected, and bundle it if need be

    Ok(())
}
