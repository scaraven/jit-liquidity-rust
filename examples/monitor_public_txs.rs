use jit_liquidity_rust::{
    addresses, config,
    membuilder::{create_ws_provider, MemPoolBuilder},
    subscribefilter::ShallowFilterType,
};

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load();

    // Create provider instance
    let pool = MemPoolBuilder::default()
        .with_provider(create_ws_provider(&config.rpc_url_ws.expect("WS URL not set")).await?)
        .build()
        .await?;

    println!("Listening for transactions...");

    // Filter for transactions to a uniswap v3 manager
    let non_fungible = addresses::get_address("0xC36442b4a4522E871399CD717aBDD847Ab11FE88")?;
    let (handle, mut recv, config) = pool
        .subscribe(ShallowFilterType::Recipient(non_fungible))
        .await?;

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
