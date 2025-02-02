use std::sync::Arc;

use alloy::{network::EthereumWallet, providers::ProviderBuilder, transports::http::reqwest::Url};
use eyre::Result;
use jit_liquidity_rust::{
    config::runconfig,
    flashbots_share::jit_bundler::{IExecutor, UniswapV3LiquidityBundler},
    providers::alchemy::AlchemyProvider,
    utils::addresses,
    watcher::{
        membuilder::{create_ws_provider, MemPoolBuilder},
        subscribefilter::ShallowFilterType,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = runconfig::Config::load();

    let ws_provider = Arc::new(AlchemyProvider::new(
        create_ws_provider(&config.rpc_url_ws.expect("WS URL not set")).await?,
    ));

    // Put in your deployed executor address here!
    let executor = addresses::get_address("0x840EE4C41De0792Af6aD223D73De591218432D72")?;

    let wallet = EthereumWallet::from(config.signer);

    let provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(Url::parse(&config.rpc_url.unwrap())?),
    );

    // Generate flashbot signer
    let flashbot_signer = config.flashbot_signer.unwrap();
    let flashbot_provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(flashbot_signer.clone()))
            .on_http(Url::parse("https://relay.flashbots.net").unwrap()),
    );

    // Create provider instance
    let pool = MemPoolBuilder::default()
        .with_provider(ws_provider)
        .build()
        .await?;

    println!("Listening for transactions...");

    // Filter for transactions to uniswap v3 manager
    let manager = addresses::get_address("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?;
    let (handle, mut recv, config) = pool
        .subscribe(ShallowFilterType::Recipient(manager))
        .await?;

    tokio::spawn(async move {
        while let Some(tx) = recv.recv().await {
            // Bundle transaction
            let bundler =
                UniswapV3LiquidityBundler::new(IExecutor::new(executor, provider.clone()));
        }
    });

    // Wait for handle to finish
    let _ = handle.await;

    todo!();

    Ok(())
}
