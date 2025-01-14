use std::sync::Arc;

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    providers::{Provider, ProviderBuilder, WalletProvider},
    rpc::types::{Transaction, TransactionRequest},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use eyre::Result;
use jit_liquidity_rust::{
    addresses, config,
    mev::{
        jit_bundler::{IExecutor, UniswapV3LiquidityBundler},
        FlashBotMev,
    },
};
use revm::primitives::U256;

#[tokio::main]
async fn main() -> Result<()> {
    // Constants
    let uniswap_manager = addresses::get_address("0xC36442b4a4522E871399CD717aBDD847Ab11FE88")?;

    // Put in your deployed executor address here!
    let executor = addresses::get_address("0x840EE4C41De0792Af6aD223D73De591218432D72")?;

    // Build a provider
    let config = config::Config::load();
    let wallet = EthereumWallet::from(config.signer);

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(Url::parse(&config.rpc_url.unwrap())?);

    // Generate flashbot signer
    let flashbot_signer = config.flashbot_signer.unwrap();
    let flashbot_provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(flashbot_signer.clone()))
            .on_http(Url::parse("https://relay.flashbots.net").unwrap()),
    );

    // Build "intercepted" transaction
    let tx_raw = TransactionRequest::default()
        .with_to(uniswap_manager)
        .with_value(U256::from(0));

    let signed;
    {
        let key = PrivateKeySigner::random();
        let client_addr = key.address();
        let random_client: EthereumWallet = EthereumWallet::from(key);

        signed = Transaction {
            inner: tx_raw.build(&random_client).await?,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            effective_gas_price: None,
            from: client_addr,
        };
    }

    let provider = Arc::new(provider);
    let bundler = UniswapV3LiquidityBundler::new(IExecutor::new(executor, provider.clone()));

    let mev = FlashBotMev::new(
        provider.clone(),
        flashbot_provider,
        provider.wallet(),
        flashbot_signer,
        bundler,
        signed,
    );

    let block_number = provider.get_block_number().await?;

    let response = mev.sim_bundle(block_number + 3).await?;

    println!("{:#?}", response);

    Ok(())
}
