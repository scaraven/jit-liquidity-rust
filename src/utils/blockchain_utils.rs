use ethers::{
    providers::{JsonRpcClient, Middleware, Provider},
    types::U256,
};

// Get block timestamp
pub async fn get_block_timestamp_future<C: JsonRpcClient>(
    provider: &Provider<C>,
    seconds: U256,
) -> U256 {
    let block_number = provider.get_block_number().await.unwrap();
    let block = provider.get_block(block_number).await.unwrap().unwrap();

    block.timestamp + seconds
}

#[cfg(test)]
mod tests {
    use crate::testconfig;
    use crate::utils;

    use super::*;

    #[tokio::test]
    async fn check_timestamp() {
        let config = testconfig::TestConfig::load();
        let (provider, _client) = utils::setup(
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
