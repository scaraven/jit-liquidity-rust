use std::marker::PhantomData;

use alloy::{
    network::{Ethereum, Network},
    primitives::{FixedBytes, U256},
    providers::Provider,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

pub struct Executor<
    'a,
    P: Provider<T, N>,
    T: Transport + Clone = BoxTransport,
    N: Network = Ethereum,
> {
    provider: &'a P,
    tx: <N as Network>::TransactionRequest,
    _marker: std::marker::PhantomData<(T, N)>,
}

impl<'a, P: Provider<T, N>, T: Transport + Clone, N: Network> Executor<'a, P, T, N> {
    pub fn new(provider: &'a P, tx: <N as Network>::TransactionRequest) -> Self {
        Self {
            provider,
            tx,
            _marker: PhantomData,
        }
    }

    pub async fn send(self) -> Result<FixedBytes<32>> {
        let result = self.provider.send_transaction(self.tx).await?.watch().await;
        Ok(result?)
    }

    pub async fn call_return_uint(self) -> Result<U256> {
        let result = self.provider.call(&self.tx).await;
        Ok(result.map(|res| U256::from_be_slice(res.as_ref()))?)
    }
}
