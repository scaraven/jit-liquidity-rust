use alloy::{
    hex::FromHex,
    primitives::{Address, Bytes, TxKind, U256},
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolEvent,
};
use eyre::{eyre, OptionExt, Result};
use revm::primitives::{ExecutionResult, Log, ResultAndState};

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

struct UniswapV3SwapInfo {
    sender: Address,
    token0: Address,
    token1: Address,
    token0_amount: U256,
    token1_amount: U256,
    fee: U256,
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
pub fn decode_uniswapv3_logs(tx: ExecutionResult) -> Vec<Log<Swap>> {
    // Assert that we are using Uniswap V3 with the corret function signatures
    let result = tx.into_logs();

    result
        .into_iter()
        .filter_map(|res| Swap::decode_log(&res, true).ok())
        .collect()
}
