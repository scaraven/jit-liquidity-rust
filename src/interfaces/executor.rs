use alloy::{
    primitives::{FixedBytes, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::http::{Client, Http},
};

use eyre::Result;

pub struct Executor<'a, P: Provider<Http<Client>>> {
    provider: &'a P,
    tx: TransactionRequest,
}

impl<'a, P: Provider<Http<Client>>> Executor<'a, P> {
    pub fn new(provider: &'a P, tx: TransactionRequest) -> Self {
        Self { provider, tx }
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
