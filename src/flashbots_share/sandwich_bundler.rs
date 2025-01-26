use async_trait::async_trait;
use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

#[async_trait]
pub trait SandwichBundler<
    P: Provider<T, N>,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
>
{
    async fn build(
        &self,
        provider: Arc<P>,
        tx: TransactionRequest,
    ) -> Result<(Vec<TransactionRequest>, Vec<TransactionRequest>)>;
}
