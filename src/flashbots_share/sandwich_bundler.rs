use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

// TODO! Create a struct which takes in a single sandwich Transaction, verifies, bundles it
// Use a builder design pattern to create the struct
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
