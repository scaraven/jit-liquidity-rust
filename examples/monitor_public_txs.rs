use std::sync::Arc;

use jit_liquidity_rust::{
    addresses,
    alchemy::AlchemyProvider,
    config,
    membuilder::{create_ws_provider, MemPoolBuilder},
    subscribefilter::ShallowFilterType,
};

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load();

    let provider = Arc::new(AlchemyProvider::new(
        create_ws_provider(&config.rpc_url_ws.expect("WS URL not set")).await?,
    ));

    // Create provider instance
    let pool = MemPoolBuilder::default()
        .with_provider(provider)
        .build()
        .await?;

    println!("Listening for transactions...");

    // Filter for transactions to a uniswap v3 manager
    let usdc = addresses::get_address("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?;
    let (handle, mut recv, config) = pool.subscribe(ShallowFilterType::Recipient(usdc)).await?;

    // Listen to 10 transactions
    for _ in 0..10 {
        if let Some(tx) = recv.recv().await {
            println!("{:#?}", tx);
        }
    }

    println!("Shutting down...");

    // Shutdown the mempool
    config.shutdown();

    // Wait for handle to finish
    let _ = handle.await;

    println!("Done!");

    Ok(())
}
