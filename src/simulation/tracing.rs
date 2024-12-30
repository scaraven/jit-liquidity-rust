use eyre::Result;
use std::sync::Arc;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::Network,
    providers::Provider,
    transports::Transport,
};
use revm::{
    db::{AlloyDB, CacheDB},
    Database, Evm, InMemoryDB,
};

pub async fn revm_setup<'a, T: Transport + Clone, N: Network, P: Provider<T, N> + 'a>(
    provider: Arc<P>,
) -> Result<Evm<'a, (), CacheDB<AlloyDB<T, N, Arc<P>>>>> {
    let block = provider.get_block_number().await.unwrap();
    let alloydb = CacheDB::new(
        AlloyDB::new(
            provider.clone(),
            BlockId::Number(BlockNumberOrTag::Number(block)),
        )
        .unwrap(),
    );

    Ok(Evm::builder().with_db(alloydb).build())
}
