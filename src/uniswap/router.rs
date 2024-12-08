use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::types::Address,
    middleware::SignerMiddleware,
    providers::{Provider, Http},
    signers::LocalWallet,
};

use eyre::Result;

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA,address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint ) external returns (uint amountA, uint amountB)
    ]"#,
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
    pair: Address
) -> Result<()> {
    // Fetch contract
    let contract = UniswapV2Pair::new(pair, client.clone());

    let token0 = contract.token_0().call().await?;
    let token0_address = Address::from(token0);

    println!("Token0 address {}", token0_address);

    Ok(())
}