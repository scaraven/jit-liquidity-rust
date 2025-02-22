use alloy::{
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::U256,
    providers::{Provider, ProviderBuilder},
};
use eyre::Result;
use jit_liquidity_rust::{
    config::runconfig,
    interfaces::{erc20, executor::Executor, router02},
    pow,
    utils::{addresses, blockchain_utils},
};

// Main function to execute the liquidity operations
#[tokio::main]
async fn main() -> Result<()> {
    const BASE: u64 = 10;
    const DECIMALS: u64 = 18;

    let config = runconfig::Config::load();
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
    let pair = *addresses::USDC_WBTC_PAIR;

    let token0_address = router02::fetch_token0(&provider, pair).await?;
    let token1_address = router02::fetch_token1(&provider, pair).await?;

    println!("Token0 address: {:?}", token0_address);
    println!("Token1 address: {:?}", token1_address);

    let router_address = *addresses::UNISWAP_V2_ROUTER;

    // Setup approvals for token0 and token1
    Executor::new(
        &provider,
        erc20::approve(
            &provider,
            token0_address,
            router_address,
            U256::from(1e18 as u32),
        ),
    )
    .send()
    .await?;
    Executor::new(
        &provider,
        erc20::approve(
            &provider,
            token1_address,
            router_address,
            U256::from(1e18 as u32),
        ),
    )
    .send()
    .await?;

    // Fetch token0 and token1 balances
    let token0_balance = Executor::new(
        &provider,
        erc20::balance_of(&provider, token0_address, client),
    )
    .call_return_uint()
    .await?;
    let token1_balance = Executor::new(
        &provider,
        erc20::balance_of(&provider, token1_address, client),
    )
    .call_return_uint()
    .await?;

    println!("Token0 balance: {:?}", token0_balance);
    println!("Token1 balance: {:?}", token1_balance);

    // Approve WETH to token0
    let weth_address = *addresses::WETH;
    Executor::new(
        &provider,
        erc20::approve(
            &provider,
            weth_address,
            router_address,
            pow!(BASE, DECIMALS),
        ),
    )
    .send()
    .await?;

    // Swap ETH for token0
    let amount_in = pow!(BASE, DECIMALS);
    let amount_out_min = U256::ZERO;

    Executor::new(
        &provider,
        router02::swap_exact_ethfor_tokens(
            &provider,
            router_address,
            token0_address,
            amount_in,
            amount_out_min,
            client,
            blockchain_utils::get_block_timestamp_future(&provider, 60).await?,
        ),
    )
    .send()
    .await?;
    Executor::new(
        &provider,
        router02::swap_exact_ethfor_tokens(
            &provider,
            router_address,
            token1_address,
            amount_in,
            amount_out_min,
            client,
            blockchain_utils::get_block_timestamp_future(&provider, 60).await?,
        ),
    )
    .send()
    .await?;

    let token0_balance_after = Executor::new(
        &provider,
        erc20::balance_of(&provider, token0_address, client),
    )
    .call_return_uint()
    .await?;
    let token1_balance_after = Executor::new(
        &provider,
        erc20::balance_of(&provider, token1_address, client),
    )
    .call_return_uint()
    .await?;
    println!("Token0 balance after swap: {:?}", token0_balance_after);
    println!("Token1 balance after swap: {:?}", token1_balance_after);
    Ok(())
}
