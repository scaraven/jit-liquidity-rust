use alloy::{
    primitives::{Address, FixedBytes, U256},
    providers::Provider,
    sol,
    transports::http::{reqwest, Http},
};

use eyre::Result;
use IERC20Token::IERC20TokenInstance;

sol!(
    #[sol(rpc)]
    "contracts/src/IERC20Token.sol"
);

fn create_erc20_token<P: Provider<Http<reqwest::Client>>>(
    provider: P,
    address: Address,
) -> IERC20TokenInstance<Http<reqwest::Client>, P> {
    IERC20Token::new(address, provider)
}

// Check that approval is over a limit
pub async fn check_approval_limit<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    owner: Address,
    spender: Address,
    desired: U256,
) -> bool {
    let allowance = allowance(&provider, token_addr, owner, spender)
        .await
        .expect("ALLOWANCE failed");
    allowance >= desired
}

pub async fn approve<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    spender: Address,
    amount: U256,
) -> Result<FixedBytes<32>> {
    let contract = create_erc20_token(provider, token_addr);

    let builder = contract.approve(spender, amount);

    let tx_hash = builder.send().await.unwrap().watch().await;

    tx_hash.map_err(|e| e.into())
}

pub async fn balance_of<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    spender: Address,
) -> Result<U256> {
    let contract = create_erc20_token(provider, token_addr);

    let balance = contract.balanceOf(spender).call().await.unwrap().amount;

    Ok(balance)
}

pub async fn allowance<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    owner: Address,
    spender: Address,
) -> Result<U256> {
    let contract = create_erc20_token(provider, token_addr);

    let allowance = contract
        .allowance(owner, spender)
        .call()
        .await
        .unwrap()
        .amount;

    Ok(allowance)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::addresses;
    use crate::setup;

    #[tokio::test]
    async fn check_balance_zero() {
        let random_addr = addresses::get_address("0x8BB0080aC1006D407dfe84D29013964aCC1b9C00");

        let (provider, _) = setup::test_setup().await;

        let balance_random = balance_of(
            &provider,
            addresses::get_address(addresses::WETH),
            random_addr,
        )
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_random, U256::from(0));
    }

    #[tokio::test]
    async fn check_balance_whale() {
        let whale_addr = addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3");
        const EXPECTED: u64 = 236000000000000;

        let (provider, _) = setup::test_setup().await;

        let balance_whale = balance_of(
            &provider,
            addresses::get_address(addresses::USDC_ADDR),
            whale_addr,
        )
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_whale, U256::from(EXPECTED));
    }

    #[tokio::test]
    async fn check_approve() {
        let whale_addr = addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3");
        let amount = U256::from(1e18 as u32);

        let (provider, address) = setup::test_setup().await;

        let approve = approve(
            &provider,
            addresses::get_address(addresses::WETH),
            whale_addr,
            amount,
        )
        .await;

        assert!(approve.is_ok(), "APPROVE failed");

        // Ensure that allowance is now 1e18
        let allowance = allowance(
            &provider,
            addresses::get_address(addresses::WETH),
            address,
            whale_addr,
        )
        .await
        .expect("ALLOWANCE failed");

        assert_eq!(allowance, amount);
    }
}
