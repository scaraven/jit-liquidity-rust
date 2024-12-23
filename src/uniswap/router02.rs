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
use tokio::spawn;

use crate::{addresses, utils};

abigen!(
    UniswapV2Router,
    r"[
        swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline) external payable returns (uint[] memory amounts)
        addLiquidity(address tokenA,address tokenB, uint amountADesired, uint amountBDesired, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB, uint liquidity)
        swapETHForExactTokens(uint amountOut, address[] calldata path, address to, uint deadline) external payable returns (uint[] memory amounts)
        decreaseLiquidity(address tokenA, address tokenB, uint liquidity, uint amountAMin, uint amountBMin, address to, uint deadline) external returns (uint amountA, uint amountB)
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

    let pending_tx = swap_call.send().await.or_else(|e| Err(e));

    match pending_tx {
        Ok(tx) => Ok(tx.await?.unwrap_or(TransactionReceipt::default())),
        Err(e) => Err(e.into()),
    }
}

pub async fn swap_eth_for_exact_tokens(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
    token_b: Address,
    token_out: U256,
    amount_eth: U256,
    deadline: U256,
) -> Result<TransactionReceipt> {
    // Fetch contract
    let contract = create_uniswap_v2_router(&client, router);

    let path = vec![addresses::get_address(addresses::WETH), token_b];

    let swap_call = contract
        .swap_eth_for_exact_tokens(token_out, path, client.address(), deadline)
        .value(amount_eth);

    let pending_tx = swap_call.send().await.or_else(|e| Err(e));

    match pending_tx {
        Ok(tx) => Ok(tx.await?.unwrap_or(TransactionReceipt::default())),
        Err(e) => Err(e.into()),
    }
}

pub async fn increase_liquidity(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
    token_a: Address,
    token_b: Address,
    amount_a_desired: U256,
    amount_b_desired: U256,
    amount_a_min: U256,
    amount_b_min: U256,
    to: Address,
    deadline: U256,
) -> Result<TransactionReceipt> {
    let contract = create_uniswap_v2_router(&client, router);

    let addr = client.address();
    let task_one = spawn({
        let client = client.clone();
        async move {
            utils::check_approval_limit(&client, token_a, addr, router, amount_a_desired).await
        }
    });
    let task_two = spawn({
        let client = client.clone();
        async move {
            utils::check_approval_limit(&client, token_b, addr, router, amount_b_desired).await
        }
    });

    let (approval_one, approval_two) = tokio::try_join!(task_one, task_two)?;
    if (approval_one && approval_two) == false {
        return Err(eyre::eyre!("APPROVAL_LIMIT not met"));
    }

    let add_liquidity_call = contract.add_liquidity(
        token_a,
        token_b,
        amount_a_desired,
        amount_b_desired,
        amount_a_min,
        amount_b_min,
        to,
        deadline,
    );

    let pending_tx = add_liquidity_call.send().await.or_else(|e| Err(e));

    match pending_tx {
        Ok(tx) => Ok(tx.await?.unwrap_or(TransactionReceipt::default())),
        Err(e) => Err(e.into()),
    }
}

pub async fn decrease_liquidity(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    router: Address,
    token_a: Address,
    token_b: Address,
    liquidity: U256,
    amount_a_min: U256,
    amount_b_min: U256,
    to: Address,
    deadline: U256,
) -> Result<TransactionReceipt> {
    let contract = create_uniswap_v2_router(&client, router);

    let add_liquidity_call = contract.decrease_liquidity(
        token_a,
        token_b,
        liquidity,
        amount_a_min,
        amount_b_min,
        to,
        deadline,
    );

    let pending_tx = add_liquidity_call.send().await.or_else(|e| Err(e));

    match pending_tx {
        Ok(tx) => Ok(tx.await?.unwrap_or(TransactionReceipt::default())),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {

    use ethers::{
        contract::ContractError, core::k256::ecdsa, providers::Middleware, signers::Wallet,
    };

    use super::*;
    use crate::{erc20, setup, utils};

    use std::sync::LazyLock;

    static AMOUNT_DESIRED: LazyLock<ethers::types::U256> = LazyLock::new(|| U256::from(10000));

    #[tokio::test]
    async fn test_token_fetch() {
        let (_provider, client) = setup::test_setup().await;

        let pair = addresses::get_address(addresses::WETH_USDC_PAIR);
        let token0 = addresses::get_address(addresses::USDC_ADDR);
        let token1 = addresses::get_address(addresses::WETH);

        assert_eq!(fetch_token0(&client, pair).await.unwrap(), token0);
        assert_eq!(fetch_token1(&client, pair).await.unwrap(), token1);
    }

    #[tokio::test]
    async fn test_swap_exact_eth_for_tokens_positive() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        // Assert that we currently have enough ETH
        let eth_balance = provider.get_balance(client.address(), None).await.unwrap();
        assert!(eth_balance > U256::from(1e18 as u32));

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let amount_out_min = U256::zero();
        let deadline = utils::get_block_timestamp_future(&provider, U256::from(600)).await;

        // Make sure we do not hold any USDC
        let balance = erc20::balance_of(&client, token_b, client.address())
            .await
            .unwrap();

        let receipt = swap_exact_ethfor_tokens(
            &client,
            router,
            token_b,
            U256::from(1e18 as u32),
            amount_out_min,
            deadline,
        )
        .await
        .expect("SWAP_EXACT_ETH_FOR_TOKENS failed");

        assert_eq!(receipt.status.unwrap().as_u64(), 1);
        assert!(
            erc20::balance_of(&client, token_b, client.address())
                .await
                .unwrap()
                > balance + amount_out_min
        );
    }

    #[tokio::test]
    async fn test_swap_exact_eth_for_tokens_with_invalid_deadline() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        // Assert that we currently have enough ETH
        let eth_balance = provider.get_balance(client.address(), None).await.unwrap();
        assert!(eth_balance >= U256::exp10(18));

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let amount_out_min = U256::from(0);
        let mut deadline = utils::get_block_timestamp_future(&provider, U256::from(0)).await;

        deadline = deadline - U256::from(10);

        let receipt = swap_exact_ethfor_tokens(
            &client,
            router,
            token_b,
            U256::exp10(18),
            amount_out_min,
            deadline,
        )
        .await;

        assert!(receipt.is_err());
        let err = receipt.unwrap_err();
        let root = err.root_cause();

        // Check if the root cause is specifically a ContractError::Revert
        if let Some(contract_error) = root.downcast_ref::<ContractError<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>>>() {
            assert!(matches!(contract_error, ContractError::Revert(_)), "Expected a ContractError::Revert, but got a different error type.");
        } else {
            assert!(false, "Expected a ContractError, but got a different error type.");
        }
    }

    #[tokio::test]
    async fn test_swap_eth_for_exact_tokens() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);

        let token_b = addresses::get_address(addresses::USDC_ADDR);
        let deadline = utils::get_block_timestamp_future(&provider, U256::from(600)).await;
        let desired = *AMOUNT_DESIRED;

        let _ = erc20::approve(&client, token_b, router, U256::max_value())
            .await
            .unwrap();

        let receipt =
            swap_eth_for_exact_tokens(&client, router, token_b, desired, U256::exp10(18), deadline)
                .await;

        assert!(receipt.is_ok(), "SWAP_ETH_FOR_EXACT_TOKENS failed");
        assert!(
            erc20::balance_of(&client, token_b, client.address())
                .await
                .unwrap()
                >= desired,
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
        crate::utils::buy_tokens_with_eth(
            &provider,
            client.clone(),
            vec![usdc, wbtc],
            vec![*AMOUNT_DESIRED, *AMOUNT_DESIRED],
        )
        .await
        .unwrap();
        let deadline = utils::get_block_timestamp_future(&provider, U256::from(600)).await;

        assert_eq!(
            erc20::balance_of(&client, pair, client.address())
                .await
                .unwrap(),
            U256::zero()
        );

        let receipt = increase_liquidity(
            &client,
            router,
            usdc,
            wbtc,
            *AMOUNT_DESIRED,
            *AMOUNT_DESIRED,
            U256::zero(),
            U256::zero(),
            client.address(),
            deadline,
        )
        .await;

        assert!(receipt.is_ok(), "INCREASE_LIQUIDITY failed");
        assert!(
            erc20::balance_of(&client, pair, client.address())
                .await
                .unwrap()
                > U256::zero(),
            "LIQUIDITY BALANCE is zero"
        );
    }

    #[tokio::test]
    async fn test_decrease_liquidity() {
        let (provider, client) = setup::test_setup().await;

        let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);
        let usdc = addresses::get_address(addresses::USDC_ADDR);
        let wbtc = addresses::get_address(addresses::WBTC);
        crate::utils::buy_tokens_with_eth(
            &provider,
            client.clone(),
            vec![usdc, wbtc],
            vec![*AMOUNT_DESIRED, *AMOUNT_DESIRED],
        )
        .await
        .unwrap();
        let deadline = utils::get_block_timestamp_future(&provider, U256::from(600)).await;

        // Deposit liquidity
        let receipt = increase_liquidity(
            &client,
            router,
            usdc,
            wbtc,
            *AMOUNT_DESIRED,
            *AMOUNT_DESIRED,
            U256::zero(),
            U256::zero(),
            client.address(),
            deadline,
        )
        .await;

        println!("{:#?}", receipt);

        // Get amount of tokenA and tokenB used to deposit liquidity

        assert!(receipt.is_ok(), "INCREASE_LIQUIDITY failed");

        let pair = addresses::get_address(addresses::USDC_WBTC_PAIR);
        let liquidity = erc20::balance_of(&client, pair, client.address())
            .await
            .unwrap();

        println!("{:#?}", liquidity);
        // Approve liquidity
        let _ = erc20::approve(&client, pair, router, U256::max_value())
            .await
            .unwrap();

        let receipt = decrease_liquidity(
            &client,
            router,
            usdc,
            wbtc,
            liquidity,
            U256::from(1),
            U256::from(1),
            client.address(),
            deadline,
        )
        .await;

        println!("{:#?}", receipt);
        assert!(receipt.is_ok(), "DECREASE_LIQUIDITY failed");
    }
}
