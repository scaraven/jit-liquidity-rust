use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::TransactionReceipt,
};

use eyre::Result;

abigen!(
    UniswapV2Router,
    r"[
        addLiquidity(address tokenA,address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB, uint liquidity)
    ]"
);

abigen!(
    UniswapV2Pair,
    r#"[
        approve(address,uint256)(bool)
        getReserves()(uint112,uint112,uint32)
        token0()(address)
        token1()(address)
    ]"#
);

pub async fn fetch_token0(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pair: Address,
) -> Result<Address> {
    // Fetch contract
    let contract = UniswapV2Pair::new(pair, client.clone());

    let token0 = contract.token_0().call().await?;
    let token0_address = Address::from(token0);

    Ok(token0_address)
}

pub async fn fetch_token1(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pair: Address,
) -> Result<Address> {
    // Fetch contract
    let contract = UniswapV2Pair::new(pair, client.clone());

    let token0 = contract.token_1().call().await?;
    let token0_address = Address::from(token0);

    Ok(token0_address)
}

pub async fn increase_liquidity(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
    token_a: Address,
    token_b: Address,
    amount_adesired: U256,
    amount_bdesired: U256,
    amount_amin: U256,
    amount_bmin: U256,
    to: Address,
    deadline: U256,
) -> Result<TransactionReceipt> {
    // Fetch contract
    let contract = UniswapV2Router::new(router, client.clone());

    let receipt = contract
        .add_liquidity(
            token_a,
            token_b,
            amount_adesired,
            amount_bdesired,
            amount_amin,
            amount_bmin,
            to,
            deadline,
        )
        .send()
        .await?
        .await?;

    Ok(receipt.unwrap())
}
