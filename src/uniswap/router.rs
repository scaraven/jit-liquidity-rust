use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use eyre::Result;

abigen!(
    UniswapV2Router,
    r#"[
        removeLiquidity(address tokenA,address tokenB, uint liquidity,uint amountAMin, uint amountBMin, address to, uint ) external returns (uint amountA, uint amountB)
    ]"#,
);

abigen!(
    UniswapV2Pair,
    r#"[
        approve(address,uint256)(bool)
        getReserves()(uint112,uint112,uint32)
        token0()(address)
        token1()(address)
    ]"#
);

async pub fn add_liquidity(
    provider: &Arc<Provider<Http>>,
    wallet: &LocalWallet,
    tokenA: Address,
    tokenB: address,
    amountADesired: U256,
    recipient: Address
) -> Result<TxHash> {

}