use alloy::{
    eips::BlockNumberOrTag,
    primitives::U256,
    providers::Provider,
    rpc::types::BlockTransactionsKind,
    transports::http::{reqwest, Http},
};
use eyre::{eyre, Result};

macro_rules! pow {
    ($base:expr, $exp:expr) => {
        U256::pow(U256::from($base), U256::from($exp))
    };
    () => {};
}

// Get block timestamp
pub async fn get_block_timestamp_future(
    provider: &impl Provider<Http<reqwest::Client>>,
    seconds: u64,
) -> Result<U256> {
    let block = provider
        .get_block_by_number(BlockNumberOrTag::Latest, BlockTransactionsKind::Hashes)
        .await
        .expect("GET_BLOCK_BY_NUMBER failed");

    block
        .ok_or(eyre!("Block not found"))
        .map(|block| U256::from(block.header.timestamp + seconds))
}

#[cfg(test)]
mod tests {
    use crate::setup;

    use super::*;

    #[tokio::test]
    async fn check_timestamp() {
        const DELAY: u64 = 10;
        let (provider, _) = setup::test_setup().await;

        let timestamp_result = get_block_timestamp_future(&provider, 0).await;

        assert!(timestamp_result.is_ok());

        let timestamp = timestamp_result.unwrap();

        assert_eq!(
            get_block_timestamp_future(&provider, DELAY)
                .await
                .expect("GET_BLOCK_TIMESTAMP failed"),
            timestamp + U256::from(DELAY)
        );
    }
}
