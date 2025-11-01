use std::sync::Arc;

use async_trait::async_trait;

use alloy::{
    network::{Ethereum, Network},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolEvent,
    transports::{BoxTransport, Transport},
};

use eyre::Result;

use revm::primitives::{Address, Log};
use sandwich_bundler::SandwichBundler;
use IExecutor::IExecutorInstance;

use crate::simulation::engine::EngineTask;

use super::sandwich_bundler;

sol!(
    #[sol(rpc)]
    "contracts/src/interfaces/IExecutor.sol"
);

sol! {
    #[derive(Debug)]
    #[sol(rpc)]
    event Swap(
    address indexed sender,
    address indexed recipient,
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
fn decode_uniswapv3_logs(tx: Vec<Log>) -> Result<Vec<UniswapV3SwapInfo>> {
    let decoded = tx
        .into_iter()
        .filter_map(|res| Swap::decode_log(&res, true).ok())
        .collect::<Vec<_>>();

    decoded
        .into_iter()
        .map(extract_uniswapv3_info)
        .collect::<Result<Vec<UniswapV3SwapInfo>>>()
}

fn extract_uniswapv3_info(log: Log<Swap>) -> Result<UniswapV3SwapInfo> {
    let pool = log.address;

    Ok(UniswapV3SwapInfo { pool })
}

pub struct UniswapV3LiquidityBundler<
    P: Provider<T, N>,
    T: Clone + Transport = BoxTransport,
    N: Network = Ethereum,
> {
    executor: IExecutorInstance<T, Arc<P>, N>,
}

impl<P, T, N> UniswapV3LiquidityBundler<P, T, N>
where
    P: Provider<T, N>,
    T: Transport + Clone,
    N: Network<TransactionRequest = TransactionRequest>,
{
    pub fn new(executor: IExecutorInstance<T, Arc<P>, N>) -> Self {
        Self { executor }
    }
}

#[async_trait]
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

        let logs = decode_uniswapv3_logs(result.as_ref().unwrap().result.logs().to_vec())?;

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

#[cfg(test)]
mod tests {
    use alloy::hex;
    use revm::primitives::{Log, LogData};

    use crate::{flashbots_share::jit_bundler::decode_uniswapv3_logs, utils::addresses};

    #[tokio::test]
    async fn test_decode_uniswapv3_logs() {
        let log_str = "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffd73def1000000000000000000000000000000000000000000000000000009184e72a00000000000000000000000000000000000000001e30ae01f89852dd01a164cdfe200000000000000000000000000000000000000000000000000005b91bcab7c87000000000000000000000000000000000000000000000000000000000001e2d8";
        let log_data = hex::decode(log_str).unwrap();

        let log = Log {
            address: "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad"
                .parse()
                .unwrap(),
            data: LogData::new(
                vec![
                    "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"
                        .parse()
                        .unwrap(),
                    "0x0000000000000000000000003fc91a3afd70395cd496c647d5a6cc9d4b2b7fad"
                        .parse()
                        .unwrap(),
                    "0x00000000000000000000000046bb6bb1b27069c652aa40ddbf47854b1c426428"
                        .parse()
                        .unwrap(),
                ],
                log_data.into(),
            )
            .unwrap(),
        };

        let logs = vec![log];
        let output = decode_uniswapv3_logs(logs);

        assert!(output.is_ok());
        let output = output.unwrap();

        assert_eq!(output.len(), 1);
        assert!(output.first().is_some());
        let decoded_log = output.first().unwrap();
        assert_eq!(
            decoded_log.pool,
            addresses::get_address("0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").unwrap()
        );
    }
}
