use ethers::{providers::Middleware, types::U256};

use eyre::Result;

#[path = "config.rs"]
mod config;

#[path = "uniswap/router.rs"]
mod router;

#[path = "interfaces/erc20.rs"]
mod erc20;

#[path = "utils/blockchain_utils.rs"]
mod blockchain_utils;

#[path = "utils/utils.rs"]
mod utils;

#[path = "utils/addresses.rs"]
mod addresses;

#[cfg(test)]
#[path = "testconfig.rs"]
mod testconfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load();

    let anvil = utils::setup_anvil(config.anvil_path.as_deref(), config.rpc_url.as_deref()).await?;
    let (provider, client) =
        utils::setup(anvil.endpoint().as_str(), config.priv_key.as_str()).await?;

    let eth_balance = provider.get_balance(client.address(), None).await?;
    println!("ETH balance: {:?}", eth_balance);

    // Fetch pair address
    let pair = addresses::get_address(addresses::WETH_USDC_PAIR);

    let token0_address = router::fetch_token0(&client, pair).await?;
    let token1_address = router::fetch_token1(&client, pair).await?;

    println!("Token0 address: {:?}", token0_address);
    println!("Token1 address: {:?}", token1_address);

    let router_address = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

    // Setup approvals for token0 and token1
    let _receipt = erc20::approve(
        &client,
        token0_address,
        router_address,
        U256::from(1e18 as u32),
    )
    .await?;
    let _receipt_two = erc20::approve(
        &client,
        token1_address,
        router_address,
        U256::from(1e18 as u32),
    )
    .await?;

    // Fetch token0 and token1 balances
    let token0_balance = erc20::balance_of(&client, token0_address, client.address()).await?;
    let token1_balance = erc20::balance_of(&client, token1_address, client.address()).await?;

    println!("Token0 balance: {:?}", token0_balance);
    println!("Token1 balance: {:?}", token1_balance);

    // Approve WETH to token0
    let weth_address = addresses::get_address(addresses::WETH);
    erc20::approve(&client, weth_address, router_address, U256::exp10(18)).await?;

    // Swap ETH for token0
    let amount_in = U256::exp10(18);
    let amount_out_min = U256::zero();

    let _receipt_three = router::swap_exact_ethfor_tokens(
        &client,
        router_address,
        token0_address,
        amount_in,
        amount_out_min,
        blockchain_utils::get_block_timestamp_future(&provider, U256::from(60)).await,
    )
    .await?;

    let _receipt_four = router::swap_exact_ethfor_tokens(
        &client,
        router_address,
        token1_address,
        amount_in,
        amount_out_min,
        blockchain_utils::get_block_timestamp_future(&provider, U256::from(60)).await,
    )
    .await?;

    let token0_balance_after = erc20::balance_of(&client, token0_address, client.address()).await?;
    let token1_balance_after = erc20::balance_of(&client, token1_address, client.address()).await?;
    println!("Token0 balance after swap: {:?}", token0_balance_after);
    println!("Token1 balance after swap: {:?}", token1_balance_after);

    Ok(())
}
