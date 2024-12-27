use alloy::network::EthereumWallet;
use alloy::node_bindings::Anvil;
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};

use eyre::Result;

#[path = "config.rs"]
mod config;

#[path = "uniswap/router02.rs"]
mod router02;

#[path = "interfaces/erc20.rs"]
mod erc20;

#[macro_use]
#[path = "utils/utils.rs"]
mod utils;

#[path = "utils/setup.rs"]
mod setup;

#[path = "utils/addresses.rs"]
mod addresses;

#[cfg(test)]
#[path = "testconfig.rs"]
mod testconfig;

#[tokio::main]
async fn main() -> Result<()> {
    const BASE: u64 = 10;
    const DECIMALS: u64 = 18;

    let config = config::Config::load();
    let wallet = EthereumWallet::from(config.signer);
    let client = config.address;

    let anvil = Anvil::new().fork(config.rpc_url.unwrap()).try_spawn()?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let eth_balance = provider.get_balance(client).await?;
    println!("ETH balance: {:?}", eth_balance);

    // Fetch pair address
    let pair = addresses::get_address(addresses::WETH_USDC_PAIR);

    let token0_address = router02::fetch_token0(&provider, pair).await?;
    let token1_address = router02::fetch_token1(&provider, pair).await?;

    println!("Token0 address: {:?}", token0_address);
    println!("Token1 address: {:?}", token1_address);

    let router_address = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

    // Setup approvals for token0 and token1
    let _receipt = erc20::approve(
        &provider,
        token0_address,
        router_address,
        U256::from(1e18 as u32),
    )
    .await?;
    let _receipt_two = erc20::approve(
        &provider,
        token1_address,
        router_address,
        U256::from(1e18 as u32),
    )
    .await?;

    // Fetch token0 and token1 balances
    let token0_balance = erc20::balance_of(&provider, token0_address, client).await?;
    let token1_balance = erc20::balance_of(&provider, token1_address, client).await?;

    println!("Token0 balance: {:?}", token0_balance);
    println!("Token1 balance: {:?}", token1_balance);

    // Approve WETH to token0
    let weth_address = addresses::get_address(addresses::WETH);
    erc20::approve(
        &provider,
        weth_address,
        router_address,
        pow!(BASE, DECIMALS),
    )
    .await?;

    // Swap ETH for token0
    let amount_in = pow!(BASE, DECIMALS);
    let amount_out_min = U256::ZERO;

    let _receipt_three = router02::swap_exact_ethfor_tokens(
        &provider,
        router_address,
        token0_address,
        amount_in,
        amount_out_min,
        client,
        utils::get_block_timestamp_future(&provider, 60).await?,
    )
    .await?;

    let _receipt_four = router02::swap_exact_ethfor_tokens(
        &provider,
        router_address,
        token1_address,
        amount_in,
        amount_out_min,
        client,
        utils::get_block_timestamp_future(&provider, 60).await?,
    )
    .await?;

    let token0_balance_after = erc20::balance_of(&provider, token0_address, client).await?;
    let token1_balance_after = erc20::balance_of(&provider, token1_address, client).await?;
    println!("Token0 balance after swap: {:?}", token0_balance_after);
    println!("Token1 balance after swap: {:?}", token1_balance_after);
    Ok(())
}
