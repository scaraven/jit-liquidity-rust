use std::{marker::PhantomData, sync::Arc};

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::{BoxTransport, Transport},
};

trait FlashBotBundler {
    fn execute(self);
}

struct UniswapV3LiquidityBundler<
    P: Provider<T, N>,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
> {
    flashbot_provider: Arc<P>,
    sandwich_transaction: TransactionRequest,
    _marker: PhantomData<(T, N)>,
}

impl<P, T, N> UniswapV3LiquidityBundler<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network,
{
    pub fn new(flashbot_provider: Arc<P>, sandwich_transaction: TransactionRequest) -> Self {
        Self {
            flashbot_provider,
            sandwich_transaction,
            _marker: PhantomData,
        }
    }

    pub fn execute(self) {}
}
