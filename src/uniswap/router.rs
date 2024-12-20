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

use crate::addresses;

abigen!(
    UniswapV2Router,
    r"[
        swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline) external payable returns (uint[] memory amounts)
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

fn create_uniswap_v2_router(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
) -> UniswapV2Router<SignerMiddleware<Provider<Http>, LocalWallet>> {
    UniswapV2Router::new(router, client.clone())
}

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

pub async fn swap_exact_ethfor_tokens(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
    token_b: Address,
    amount_eth: U256,
    amount_out_min: U256,
    deadline: U256,
) -> Result<TransactionReceipt> {
    // Fetch contract
    let contract = create_uniswap_v2_router(&client, router);

    let path = vec![addresses::get_address(addresses::WETH), token_b];

    let swap_call = contract
        .swap_exact_eth_for_tokens(amount_out_min, path, client.address(), deadline)
        .value(amount_eth);

    let pending_tx = swap_call.send().await.or_else(|e| {
        Err(eyre::eyre!(
            "Error swapping ETH for tokens: {:?}",
            e.to_string()
        ))
    });

    match pending_tx {
        Ok(tx) => Ok(tx.await?.unwrap_or(TransactionReceipt::default())),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testconfig;

    #[tokio::test]
    async fn test_token_fetch() {
        let config = testconfig::TestConfig::load();
        let (_provider, client) = crate::utils::setup(
            config
                .anvil_endpoint
                .expect("ANVIL_ENDPOINT does not exist")
                .as_str(),
            config.priv_key.as_str(),
        )
        .await
        .expect("UTILS_SETUP failed");

        let pair = addresses::get_address(addresses::WETH_USDC_PAIR);
        let token0 = addresses::get_address(addresses::USDC_ADDR);
        let token1 = addresses::get_address(addresses::WETH);

        assert_eq!(fetch_token0(&client, pair).await.unwrap(), token0);
        assert_eq!(fetch_token1(&client, pair).await.unwrap(), token1);
    }
}
