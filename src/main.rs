use ethers::providers::Middleware;
use ethers::types::{Address, U256};

use eyre::Result;

#[path = "config.rs"]
mod config;
#[path = "uniswap/router.rs"]
mod router;
#[path = "wallet.rs"]
mod wallet;

#[path = "interfaces/erc20.rs"]
mod erc20;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup config file
    let config = config::Config::load();

    // Setup provider
    let provider = wallet::create_provider(&config.rpc_url);
    let chain_id = provider.get_chainid().await?;

    // provider is now consumed
    let client = wallet::create_signer(provider, &config.priv_key, chain_id.as_u64());

    // Fetch pair address
    let pair = "0xa2107fa5b38d9bbd2c461d6edf11b11a50f6b974".parse::<Address>()?;

    let token0_address = router::fetch_token0(&client, pair).await?;
    let token1_address = router::fetch_token1(&client, pair).await?;

    println!("Token0 address: {:?}", token0_address);
    println!("Token1 address: {:?}", token1_address);

    let router_address = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d".parse::<Address>()?;

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
    let token0_balance = erc20::balance_of(&client, token0_address, router_address).await?;
    let token1_balance = erc20::balance_of(&client, token1_address, router_address).await?;

    println!("Token0 balance: {:?}", token0_balance);
    println!("Token1 balance: {:?}", token1_balance);

    Ok(())
}
