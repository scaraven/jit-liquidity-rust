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
    use crate::config;
    use crate::utils;

    use super::*;

    #[tokio::test]
    async fn check_timestamp() {
        let config = config::Config::load();
        let (provider, _client, _anvil) = utils::setup(config).await.expect("UTILS_SETUP failed");

        assert_eq!(
            get_block_timestamp_future(&provider, U256::zero())
                .await
                .as_u64(),
            1734545255
        );
        assert_eq!(
            get_block_timestamp_future(&provider, U256::from(10))
                .await
                .as_u64(),
            1734545265
        );
    }
}
