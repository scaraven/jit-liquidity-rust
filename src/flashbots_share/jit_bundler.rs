use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

use IExecutor::IExecutorInstance;

use crate::{bundle_extraction, engine::EngineTask};

sol!(
    #[sol(rpc)]
    "contracts/src/interfaces/IExecutor.sol"
);
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

struct UniswapV3LiquidityBundler<
    P: Provider<T, N>,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
> {
    executor: IExecutorInstance<T, P, N>,
}

impl<P, T, N> UniswapV3LiquidityBundler<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    pub fn new(executor: IExecutorInstance<T, P, N>) -> Self {
        Self { executor }
    }
}

impl<P, T, N> SandwichBundler<P, T, N> for UniswapV3LiquidityBundler<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    async fn build(
        &self,
        provider: Arc<P>,
        tx: TransactionRequest,
    ) -> Result<(Vec<TransactionRequest>, Vec<TransactionRequest>)> {
        // Extract pool address
        // Create an engine task and execute
        let task = EngineTask::new(provider, vec![tx]);
        let result = task.consume();

        // Extract ResultAndState and assert we have no errors
        let result = result
            .first()
            .ok_or_else(|| eyre::eyre!("No result found"))?;
        let logs = bundle_extraction::extract(result.as_ref().unwrap().result.clone());

        // For now assert that we only have one Swap log
        // TODO: Handle multiple logs
        if logs.len() != 1 {
            return Err(eyre::eyre!("Expected 1 log, got {}", logs.len()));
        }

        let log = logs.first().ok_or_else(|| eyre::eyre!("No log found"))?;
        let frontrun = self.executor.execute(log.pool).into_transaction_request();
        let backrun = self.executor.finish().into_transaction_request();

        Ok((vec![frontrun], vec![backrun]))
    }
}
