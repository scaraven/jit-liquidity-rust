#![allow(dead_code)]
use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::Provider,
    sol,
    transports::http::{reqwest, Http},
};
use eyre::Result;
use IUniswapV2Router::IUniswapV2RouterInstance;

use crate::{addresses, erc20, utils};

mod router02interface;

const DELAY: u64 = 600;
const MIN_ETH_DECIMALS: u64 = 18;
const BASE: u64 = 10;

sol!(
    #[sol(rpc)]
    "contracts/src/IUniswapV2Router.sol"
);

sol!(
    #[sol(rpc)]
    "contracts/src/IUniswapV2Pair.sol"
);

fn create_uniswap_v2_router<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    router: Address,
) -> IUniswapV2RouterInstance<Http<reqwest::Client>, P> {
    IUniswapV2Router::new(router, provider)
}

pub async fn buy_tokens_with_eth<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    tokens: Vec<Address>,
    amounts: Vec<U256>,
    to: Address,
) -> Result<()> {
    let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);
    let deadline = utils::get_block_timestamp_future(provider, DELAY)
        .await
        .expect("GET_BLOCK_TIMESTAMP failed");

    for (token, amount) in tokens.into_iter().zip(amounts) {
        let _ = swap_eth_for_exact_tokens(
            provider,
            router,
            token,
            amount,
            U256::pow(U256::from(BASE), U256::from(MIN_ETH_DECIMALS)),
            to,
            deadline,
        )
        .await
        .expect("SWAP_EXACT_ETH_FOR_TOKENS failed");

        let _ = erc20::approve(&provider, token, router, U256::MAX)
            .await
            .expect("APPROVE failed");
    }

    Ok(())
}

pub async fn fetch_token0<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    pair: Address,
) -> Result<Address> {
    // Fetch contract
    let contract = IUniswapV2Pair::new(pair, provider);

    let token0 = contract.token0().call().await?._0;

    Ok(token0)
}

pub async fn fetch_token1<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    pair: Address,
) -> Result<Address> {
    // Fetch contract
    let contract = IUniswapV2Pair::new(pair, provider);

    let token0 = contract.token1().call().await?._0;

    Ok(token0)
}

pub async fn swap_exact_ethfor_tokens<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    router_addr: Address,
    token_b: Address,
    amount_eth: U256,
    amount_out_min: U256,
    to: Address,
    deadline: U256,
) -> Result<FixedBytes<32>> {
    let router = create_uniswap_v2_router(provider, router_addr);
    let path = vec![addresses::get_address(addresses::WETH), token_b];

    let swap_call = router
        .swapExactETHForTokens(amount_out_min, path, to, deadline)
        .value(amount_eth);

    let pending_tx = swap_call.send().await;

    match pending_tx {
        Ok(tx) => Ok(tx.watch().await?),
        Err(e) => Err(e.into()),
    }
}

pub async fn swap_eth_for_exact_tokens<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    router_addr: Address,
    token_b: Address,
    token_out_amount: U256,
    amount_eth: U256,
    to: Address,
    deadline: U256,
) -> Result<FixedBytes<32>> {
    let router = create_uniswap_v2_router(provider, router_addr);
    let path = vec![addresses::get_address(addresses::WETH), token_b];

    let swap_call = router
        .swapETHForExactTokens(token_out_amount, path, to, deadline)
        .value(amount_eth);

    let pending_tx = swap_call.send().await;

    match pending_tx {
        Ok(tx) => Ok(tx.watch().await?),
        Err(e) => Err(e.into()),
    }
}

pub async fn increase_liquidity<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    router_addr: Address,
    args: router02interface::IncreaseLiquidityArgs,
    owner: Address,
    deadline: U256,
) -> Result<FixedBytes<32>> {
    let router = create_uniswap_v2_router(&provider, router_addr);

    let approval_one = {
        erc20::check_approval_limit(
            &provider,
            args.token_a,
            owner,
            router_addr,
            args.amount_a_desired,
        )
        .await
    };
    let approval_two = {
        erc20::check_approval_limit(
            &provider,
            args.token_b,
            owner,
            router_addr,
            args.amount_b_desired,
        )
        .await
    };

    if !(approval_one && approval_two) {
        return Err(eyre::eyre!("APPROVAL_LIMIT not met"));
    }

    let add_liquidity_call = router.addLiquidity(
        args.token_a,
        args.token_b,
        args.amount_a_desired,
        args.amount_b_desired,
        args.amount_a_min,
        args.amount_b_min,
        args.to,
        deadline,
    );

    let pending_tx: std::result::Result<_, _> = add_liquidity_call.send().await;

    match pending_tx {
        Ok(tx) => Ok(tx.watch().await?),
        Err(e) => Err(e.into()),
    }
}

pub async fn remove_liquidity<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    router_addr: Address,
    args: router02interface::DecreaseLiquidityArgs,
    deadline: U256,
) -> Result<FixedBytes<32>> {
    let router = create_uniswap_v2_router(&provider, router_addr);

    let remove_liquidity_call = router.removeLiquidity(
        args.token_a,
        args.token_b,
        args.liquidity,
        args.amount_a_min,
        args.amount_b_min,
        args.to,
        deadline,
    );

    let pending_tx: std::result::Result<_, _> = remove_liquidity_call.send().await;

    match pending_tx {
        Ok(tx) => Ok(tx.watch().await?),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{erc20, setup, utils};

    const AMOUNT_DESIRED: u64 = 100000;

    #[tokio::test]
    async fn test_token_fetch() {
        let (provider, _client) = setup::test_setup().await;

        let pair = addresses::get_address(addresses::WETH_USDC_PAIR);
        let token0 = addresses::get_address(addresses::USDC_ADDR);
        let token1 = addresses::get_address(addresses::WETH);

        assert_eq!(fetch_token0(&provider, pair).await.unwrap(), token0);
        assert_eq!(fetch_token1(&provider, pair).await.unwrap(), token1);
    }

    #[tokio::test]
    async fn test_swap_exact_eth_for_tokens_positive() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        // Assert that we currently have enough ETH
        let eth_balance = provider.get_balance(client).await.unwrap();
        assert!(eth_balance > U256::from(1e18 as u32));

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let amount_out_min = U256::ZERO;
        let deadline = utils::get_block_timestamp_future(&provider, DELAY)
            .await
            .expect("GET_BLOCK_TIMESTAMP failed");

        // Make sure we do not hold any USDC
        let balance = erc20::balance_of(&provider, token_b, client).await.unwrap();

        let receipt = swap_exact_ethfor_tokens(
            &provider,
            router,
            token_b,
            U256::from(1e18 as u32),
            amount_out_min,
            client,
            deadline,
        )
        .await;

        assert!(receipt.is_ok(), "SWAP_EXACT_ETH_FOR_TOKENS failed");
        assert!(
            erc20::balance_of(&provider, token_b, client).await.unwrap() > balance + amount_out_min
        );
    }

    #[tokio::test]
    async fn test_swap_exact_eth_for_tokens_with_invalid_deadline() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        // Assert that we currently have enough ETH
        let eth_balance = provider.get_balance(client).await.unwrap();
        assert!(eth_balance >= U256::pow(U256::from(BASE), U256::from(MIN_ETH_DECIMALS)));

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let amount_out_min = U256::ZERO;
        let mut deadline = utils::get_block_timestamp_future(&provider, 0)
            .await
            .expect("GET_BLOCK_TIMESTAMP failed");

        deadline -= U256::from(10);

        let receipt = swap_exact_ethfor_tokens(
            &provider,
            router,
            token_b,
            U256::from(1e18 as u32),
            amount_out_min,
            client,
            deadline,
        )
        .await;

        assert!(receipt.is_err());
    }

    #[tokio::test]
    async fn test_swap_eth_for_exact_tokens() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let deadline = utils::get_block_timestamp_future(&provider, DELAY)
            .await
            .expect("GET_BLOCK_TIMESTAMP failed");
        let desired = U256::from(AMOUNT_DESIRED);

        let _ = erc20::approve(&provider, token_b, router, U256::MAX)
            .await
            .unwrap();

        let receipt = swap_eth_for_exact_tokens(
            &provider,
            router,
            token_b,
            desired,
            U256::pow(U256::from(BASE), U256::from(MIN_ETH_DECIMALS)),
            client,
            deadline,
        )
        .await;

        assert!(receipt.is_ok(), "SWAP_ETH_FOR_EXACT_TOKENS failed");
        assert!(
            erc20::balance_of(&provider, token_b, client).await.unwrap() >= desired,
            "TOKEN BALANCE is less than desired"
        );
    }

    #[tokio::test]
    async fn test_increase_liquidity() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);
        let usdc = addresses::get_address(addresses::USDC_ADDR);
        let wbtc = addresses::get_address(addresses::WBTC);
        let pair = addresses::get_address(addresses::USDC_WBTC_PAIR);
        let desired = U256::from(AMOUNT_DESIRED);

        buy_tokens_with_eth(&provider, vec![usdc, wbtc], vec![desired, desired], client)
            .await
            .unwrap();
        let deadline = utils::get_block_timestamp_future(&provider, DELAY)
            .await
            .expect("GET_BLOCK_TIMESTAMP failed");

        assert_eq!(
            erc20::balance_of(&provider, pair, client).await.unwrap(),
            U256::ZERO
        );

        let args = router02interface::IncreaseLiquidityArgs::new(
            usdc,
            wbtc,
            desired,
            desired,
            U256::ZERO,
            U256::ZERO,
            client,
        );

        // Deposit liquidity
        let receipt = increase_liquidity(&provider, router, args, client, deadline).await;

        assert!(receipt.is_ok(), "INCREASE_LIQUIDITY failed");
        assert!(
            erc20::balance_of(&provider, pair, client).await.unwrap() > U256::ZERO,
            "LIQUIDITY BALANCE is zero"
        );
    }

    #[tokio::test]
    async fn test_remove_liquidity() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);
        let usdc = addresses::get_address(addresses::USDC_ADDR);
        let wbtc = addresses::get_address(addresses::WBTC);
        let desired = U256::from(AMOUNT_DESIRED);

        buy_tokens_with_eth(&provider, vec![usdc, wbtc], vec![desired, desired], client)
            .await
            .unwrap();
        let deadline = utils::get_block_timestamp_future(&provider, DELAY)
            .await
            .expect("GET_BLOCK_TIMESTAMP failed");

        let args = router02interface::IncreaseLiquidityArgs::new(
            usdc,
            wbtc,
            desired,
            desired,
            U256::ZERO,
            U256::ZERO,
            client,
        );

        // Deposit liquidity
        let receipt = increase_liquidity(&provider, router, args, client, deadline).await;

        println!("{:#?}", receipt);

        // Get amount of tokenA and tokenB used to deposit liquidity

        assert!(receipt.is_ok(), "INCREASE_LIQUIDITY failed");

        let pair = addresses::get_address(addresses::USDC_WBTC_PAIR);
        let liquidity = erc20::balance_of(&provider, pair, client).await.unwrap();

        assert!(liquidity > U256::ZERO, "LIQUIDITY BALANCE is zero");

        // Approve liquidity
        let _ = erc20::approve(&provider, pair, router, liquidity)
            .await
            .unwrap();

        let args = router02interface::DecreaseLiquidityArgs::new(
            usdc,
            wbtc,
            liquidity,
            U256::from(1),
            U256::from(1),
            client,
        );

        let receipt = remove_liquidity(&provider, router, args, deadline).await;

        println!("{:?}", receipt);
        assert!(receipt.is_ok(), "DECREASE_LIQUIDITY failed");
    }
}
