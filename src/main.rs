use std::{sync::Arc, time::Duration};

use alloy::{
    network::EthereumWallet,
    providers::{Provider, ProviderBuilder, WalletProvider},
    transports::http::reqwest::Url,
};
use eyre::Result;
use jit_liquidity_rust::{
    config::testnetconfig,
    flashbots_share::{
        jit_bundler::{IExecutor, UniswapV3LiquidityBundler},
        mev::FlashBotMev,
    },
    providers::alchemy::AlchemyProvider,
    utils::addresses,
    watcher::{
        membuilder::{create_ws_provider, MemPoolBuilder},
        subscribefilter::ShallowFilterType,
    },
};
use tokio::{signal, time::sleep};

#[tokio::main]
async fn main() -> Result<()> {
    let config = testnetconfig::Config::load();

    let ws_provider = Arc::new(AlchemyProvider::new(
        create_ws_provider(&config.rpc_url_ws.expect("SEPOLIA WS URL not set")).await?,
    ));

    // Put in your deployed executor address here!
    // TODO: Redeploy executor to contain correct UniswapV3 pool address
    let executor = addresses::get_address("0x25761766b35ed72174450c9678dc2694d6b2e740")?;

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
            .on_http(Url::parse("https://relay-sepolia.flashbots.net").unwrap()),
    );

    // Create provider instance
    let pool = MemPoolBuilder::default()
        .with_provider(ws_provider)
        .build()
        .await?;

    println!("Listening for transactions...");

    // Filter for transactions to uniswap v3 manager on sepolia
    let manager = addresses::get_address("0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD")?;
    let (_handle, mut recv, _config) = pool
        .subscribe(ShallowFilterType::Recipient(manager))
        .await?;

    loop {
        tokio::select! {
            // Allow early exit when Ctrl+C is pressed
            _ = signal::ctrl_c() => {
                println!("Ctrl+C pressed, exiting...");
                return Ok(()); // Exit the program
            },
            // Wait until the channel is not empty, then receive a transaction.
            tx = async {
                loop {
                    // Check if the channel is not empty.
                    if !recv.is_empty() {
                        // Now that we know something’s waiting, await the receive.
                        return recv.recv().await;
                    }
                    // Pause briefly to avoid a busy loop.
                    sleep(Duration::from_millis(50)).await;
                }
            } => {
                if let Some(tx) = tx {
                    println!("Received transaction: {:#?}", tx);
                    // Bundle transaction:
                    let bundler = UniswapV3LiquidityBundler::new(
                        IExecutor::new(executor, provider.clone())
                    );

                    let mev = FlashBotMev::new(
                        provider.clone(),
                        flashbot_provider.clone(),
                        provider.wallet(),
                        flashbot_signer.clone(),
                        bundler,
                        tx,
                    );

                    let block_number = provider.get_block_number().await.unwrap();
                    let response = mev.sim_bundle(block_number).await;

                    println!("{:#?}", response);
                }
            }
        }
    }

    Ok(())
}
