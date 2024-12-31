use std::{marker::PhantomData, sync::Arc};

use alloy::{
    network::{Ethereum, Network},
    providers::{ext::DebugApi, Provider},
    transports::{BoxTransport, Transport},
};

struct EngineTask<P, T = BoxTransport, N = Ethereum>
where
    P: Provider<T, N> + DebugApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    provider: Arc<P>,
    bundle: <N as Network>::TransactionRequest,
    // Cursed!
    _marker: PhantomData<T>,
}

impl<P, T, N> EngineTask<P, T, N>
where
    P: Provider<T, N> + DebugApi<N, T>,
    T: Transport + Clone,
    N: Network,
{
    pub fn new(provider: Arc<P>, bundle: <N as Network>::TransactionRequest) -> Self {
        Self {
            provider,
            bundle,
            _marker: PhantomData,
        }
    }
}
