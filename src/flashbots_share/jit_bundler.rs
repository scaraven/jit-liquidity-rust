use std::sync::Arc;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolEvent,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

use revm::primitives::{Address, ExecutionResult, Log};
use sandwich_bundler::SandwichBundler;
use IExecutor::IExecutorInstance;

use crate::engine::EngineTask;

use super::sandwich_bundler;

sol!(
    #[sol(rpc)]
    "contracts/src/interfaces/IExecutor.sol"
);

sol! {
    #[sol(rpc)]
    event Swap(
    address sender,
    address recipient,
    int256 amount0,
    int256 amount1,
    uint160 sqrtPriceX96,
    uint128 liquidity,
    int24 tick);
}

pub struct UniswapV3SwapInfo {
    pub pool: Address,
}

/// Extract key information from UniswapV3 logs.
///
/// # Arguments
///
/// * `tx` - The transaction result which potentially contains logs.
///
/// # Returns
///
/// * `Vec<Log<Swap>` - The resulting swap logs.
fn decode_uniswapv3_logs(tx: ExecutionResult) -> Result<Vec<UniswapV3SwapInfo>> {
    // Assert that we are using Uniswap V3 with the corret function signatures
    let result = tx.into_logs();

    result
        .into_iter()
        .filter_map(|res| Swap::decode_log(&res, true).ok())
        .map(extract_uniswapv3_info)
        .collect::<Result<Vec<UniswapV3SwapInfo>>>()
}

fn extract_uniswapv3_info(log: Log<Swap>) -> Result<UniswapV3SwapInfo> {
    let pool = log.address;

    Ok(UniswapV3SwapInfo { pool })
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
        let logs = decode_uniswapv3_logs(result.as_ref().unwrap().result.clone())?;

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
