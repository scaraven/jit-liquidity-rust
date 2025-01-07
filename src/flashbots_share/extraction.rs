use alloy::{primitives::Address, sol, sol_types::SolEvent};
use eyre::Result;
use revm::primitives::{ExecutionResult, Log};

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
fn decode_uniswapv3_logs(tx: ExecutionResult) -> Vec<Log<Swap>> {
    // Assert that we are using Uniswap V3 with the corret function signatures
    let result = tx.into_logs();

    result
        .into_iter()
        .filter_map(|res| Swap::decode_log(&res, true).ok())
        .collect()
}

pub fn extract(tx: ExecutionResult) -> Vec<UniswapV3SwapInfo> {
    let logs = decode_uniswapv3_logs(tx);

    logs.into_iter()
        .map(extract_uniswapv3_info)
        .collect::<Result<Vec<UniswapV3SwapInfo>>>()
        .unwrap()
}

fn extract_uniswapv3_info(log: Log<Swap>) -> Result<UniswapV3SwapInfo> {
    let pool = log.address;

    Ok(UniswapV3SwapInfo { pool })
}
