use alloy::primitives::{Address, U256};

pub struct DecreaseLiquidityArgs {
    pub token_a: Address,
    pub token_b: Address,
    pub liquidity: U256,
    pub amount_a_min: U256,
    pub amount_b_min: U256,
    pub to: Address,
}

impl DecreaseLiquidityArgs {
    pub fn new(
        token_a: Address,
        token_b: Address,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
    ) -> Self {
        DecreaseLiquidityArgs {
            token_a,
            token_b,
            liquidity,
            amount_a_min,
            amount_b_min,
            to,
        }
    }
}

pub struct IncreaseLiquidityArgs {
    pub token_a: Address,
    pub token_b: Address,
    pub amount_a_desired: U256,
    pub amount_b_desired: U256,
    pub amount_a_min: U256,
    pub amount_b_min: U256,
    pub to: Address,
}

impl IncreaseLiquidityArgs {
    pub fn new(
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
    ) -> Self {
        IncreaseLiquidityArgs {
            token_a,
            token_b,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
            to,
        }
    }
}
