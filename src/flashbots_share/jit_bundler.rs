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

trait FlashBotBundler {
    fn execute(self);
    fn simulate(&self);
}

struct UniswapV3LiquidityBundler<
    P: Provider<T, N>,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
> {
    executor: IExecutorInstance<T, P, N>,
    sandwich_transaction: TransactionRequest,
}

impl<P, T, N> UniswapV3LiquidityBundler<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    pub fn new(
        executor: IExecutorInstance<T, P, N>,
        sandwich_transaction: TransactionRequest,
    ) -> Self {
        Self {
            executor,
            sandwich_transaction,
        }
    }

    pub async fn build(self, provider: Arc<P>) -> Result<(TransactionRequest, TransactionRequest)> {
        // Extract pool address
        // Create an engine task and execute
        let task = EngineTask::new(provider, vec![self.sandwich_transaction.clone()]);
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

        Ok((frontrun, backrun))
    }
}
