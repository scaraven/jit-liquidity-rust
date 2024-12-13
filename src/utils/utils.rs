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
