use std::sync::Arc;

use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, JsonRpcClient, Middleware, Provider},
    signers::LocalWallet,
    types::{Address, U256},
};

use crate::erc20;
use eyre::Result;

// Get block timestamp
pub async fn get_block_timestamp_future<C: JsonRpcClient>(
    provider: &Provider<C>,
    seconds: U256,
) -> U256 {
    let block_number = provider.get_block_number().await.unwrap();
    let block = provider.get_block(block_number).await.unwrap().unwrap();

    block.timestamp + seconds
}

// Check that approval is over a limit
pub async fn check_approval_limit(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
    owner: Address,
    spender: Address,
    desired: U256,
) -> bool {
    let allowance = erc20::allowance(client, token, owner, spender)
        .await
        .expect("ALLOWANCE failed");
    allowance >= desired
}

pub async fn buy_tokens_with_eth(
    provider: &Provider<Http>,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    tokens: Vec<ethers::types::Address>,
    amounts: Vec<ethers::types::U256>,
) -> Result<()> {
    use ethers::types::U256;

    use crate::{addresses, erc20, router02, utils::get_block_timestamp_future};

    let router = addresses::get_address(addresses::UNISWAP_V2_ROUTER);
    let deadline = get_block_timestamp_future(&provider, U256::from(600)).await;

    for (token, amount) in tokens.into_iter().zip(amounts) {
        let _ = router02::swap_eth_for_exact_tokens(
            &client,
            router,
            token,
            amount,
            U256::exp10(18),
            deadline,
        )
        .await
        .expect("SWAP_EXACT_ETH_FOR_TOKENS failed");
        let _ = erc20::approve(&client, token, router, U256::max_value())
            .await
            .expect("APPROVE failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::setup;
    use crate::testconfig;

    use super::*;

    #[tokio::test]
    async fn check_timestamp() {
        let config = testconfig::TestConfig::load();
        let (provider, _client) = setup::setup(
            config
                .anvil_endpoint
                .expect("ANVIL_ENDPOINT does not exist")
                .as_str(),
            config.priv_key.as_str(),
        )
        .await
        .expect("UTILS_SETUP failed");

        let timestamp = get_block_timestamp_future(&provider, U256::zero())
            .await
            .as_u64();
        assert_eq!(
            get_block_timestamp_future(&provider, U256::from(10))
                .await
                .as_u64(),
            timestamp + 10
        );
    }
}
