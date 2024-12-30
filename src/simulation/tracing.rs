use eyre::Result;
use std::sync::Arc;

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::{Network, TransactionBuilder},
    primitives::Bytes,
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::Transport,
};
use revm::{
    db::{AlloyDB, CacheDB},
    primitives::{ExecutionResult, Output, TxEnv},
    Database, Evm, InMemoryDB,
};

pub async fn revm_call_single<'a, T: Transport + Clone, N: Network, P: Provider<T, N> + 'a>(
    provider: Arc<P>,
    cache_db: CacheDB<AlloyDB<T, N, Arc<P>>>,
    desired_tx: TransactionRequest,
) -> Result<Bytes> {
    let block = provider.get_block_number().await.unwrap();

    let mut evm = Evm::builder()
        .with_db(cache_db)
        .modify_tx_env(|tx| {
            tx.caller = desired_tx.from.unwrap_or(tx.caller);
            tx.value = desired_tx.value.unwrap_or(tx.value);
            tx.gas_limit = desired_tx.gas.unwrap_or(tx.gas_limit);
            tx.nonce = desired_tx.nonce;
            tx.data = desired_tx.input.input.unwrap_or(tx.data.clone());
            tx.transact_to = desired_tx.to.unwrap_or(tx.transact_to);
        })
        .build();

    let ref_tx = evm.transact().unwrap();
    let result = ref_tx.result;

    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        result => {
            return Err(eyre::eyre!("execution failed: {result:?}"));
        }
    };

    Ok(value)
}

pub fn init_cache_db<T: Transport + Clone, N: Network, P: Provider<T, N>>(
    provider: Arc<P>,
    block: BlockId,
) -> CacheDB<AlloyDB<T, N, Arc<P>>> {
    CacheDB::new(AlloyDB::new(provider, block).expect("Failed to create AlloyDB"))
}
