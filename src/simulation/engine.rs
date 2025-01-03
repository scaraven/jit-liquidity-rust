use std::{marker::PhantomData, sync::Arc};

use alloy::{
    eips::BlockNumberOrTag,
    network::{Ethereum, Network},
    providers::{ext::DebugApi, Provider},
    rpc::types::TransactionRequest,
    transports::{BoxTransport, Transport},
};
use revm::primitives::ResultAndState;

use eyre::Result;

use crate::tracing;

type TransactionBundle = Vec<TransactionRequest>;
pub type EngineResultBundle = Vec<Result<ResultAndState>>;

struct EngineTask<P, T = BoxTransport, N = Ethereum>
where
    P: Provider<T, N> + DebugApi<N, T>,
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
    P: Provider<T, N> + DebugApi<N, T>,
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
