use std::{marker::PhantomData, sync::Arc};

use alloy::{
    eips::BlockNumberOrTag,
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::{BoxTransport, Transport},
};
use revm::primitives::ResultAndState;

use eyre::Result;

use super::tracing;

type TransactionBundle = Vec<TransactionRequest>;
pub type EngineResultBundle = Vec<Result<ResultAndState>>;

pub struct EngineTask<P, T = BoxTransport, N = Ethereum>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    provider: Arc<P>,
    bundle: TransactionBundle,
    // Cursed!
    _marker: PhantomData<(T, N)>,
}

impl<P, T, N> EngineTask<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    pub fn new(provider: Arc<P>, bundle: TransactionBundle) -> Self {
        Self {
            provider,
            bundle,
            _marker: PhantomData,
        }
    }

    pub fn consume(self) -> EngineResultBundle {
        let mut cache_db =
            tracing::init_cache_db(self.provider.clone(), BlockNumberOrTag::Latest.into());
        let mut results = Vec::new();
        for tx in self.bundle {
            let result = tracing::revm_call_write(&mut cache_db, tx);
            results.push(result);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use revm::primitives::{TxKind, U256};

    use crate::utils::{addresses, setup};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    pub async fn test_engine_task() {
        let (provider, client) = setup::test_setup().await;
        let provider = Arc::new(provider);

        let alice = addresses::get_address("0x390e206254c9777C01d017B22eBDC7E2959fE3E8").unwrap();
        const VALUE: i32 = 100;

        // Send 200 wei to alice and then 100 wei back
        let bundle = vec![
            TransactionRequest {
                from: Some(client),
                to: Some(TxKind::Call(alice)),
                value: Some(U256::from(2 * VALUE)),
                ..Default::default()
            },
            TransactionRequest {
                from: Some(alice),
                to: Some(TxKind::Call(client)),
                value: Some(U256::from(VALUE)),
                ..Default::default()
            },
        ];

        let task = EngineTask::new(provider, bundle);
        let results = task.consume();

        assert_eq!(results.len(), 2);
        for result in &results {
            assert!(result.is_ok());
        }

        // Check that the state changes are as expected
        let res1 = results[0].as_ref().unwrap();
        assert!(res1.state.contains_key(&alice));
        assert!(res1.state.contains_key(&client));
        assert_eq!(
            res1.state.get(&alice).unwrap().info.balance,
            U256::from(2 * VALUE)
        );

        let res2 = results[1].as_ref().unwrap();
        assert!(res2.state.contains_key(&alice));
        assert!(res2.state.contains_key(&client));
        assert_eq!(
            res2.state.get(&alice).unwrap().info.balance,
            U256::from(VALUE)
        );
    }
}
