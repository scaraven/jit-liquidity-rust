use alloy::{
    network::Network,
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    sol,
    transports::{
        http::{reqwest, Http},
        Transport,
    },
};

use IERC20Token::IERC20TokenInstance;

use super::executor::Executor;

sol!(
    #[sol(rpc)]
    "contracts/src/interfaces/IERC20Token.sol"
);

fn create_erc20_token<P: Provider<T, N>, T: Transport + Clone, N: Network>(
    provider: P,
    address: Address,
) -> IERC20TokenInstance<T, P, N> {
    IERC20Token::new(address, provider)
}

/// Check if the approval limit is met.
///
/// # Arguments
///
/// * `provider` - A reference to the provider.
/// * `token_addr` - The token address.
/// * `owner` - The owner address.
/// * `spender` - The spender address.
/// * `desired` - The desired approval amount.
///
/// # Returns
///
/// * `bool` - Whether the approval limit is met.
pub async fn check_approval_limit<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    owner: Address,
    spender: Address,
    desired: U256,
) -> bool {
    let allowance_tx = allowance(&provider, token_addr, owner, spender);
    let allowance = Executor::new(provider, allowance_tx)
        .call_return_uint()
        .await
        .unwrap_or(U256::ZERO);

    allowance >= desired
}

pub fn approve<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    spender: Address,
    amount: U256,
) -> TransactionRequest {
    let contract = create_erc20_token(provider, token_addr);

    contract.approve(spender, amount).into_transaction_request()
}

pub fn balance_of<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    spender: Address,
) -> TransactionRequest {
    let contract = create_erc20_token(provider, token_addr);

    contract.balanceOf(spender).into_transaction_request()
}

pub fn allowance<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    owner: Address,
    spender: Address,
) -> TransactionRequest {
    let contract = create_erc20_token(provider, token_addr);

    contract
        .allowance(owner, spender)
        .into_transaction_request()
}

pub fn transfer<P: Provider<Http<reqwest::Client>>>(
    provider: &P,
    token_addr: Address,
    to: Address,
    amount: U256,
) -> TransactionRequest {
    let contract = create_erc20_token(provider, token_addr);

    contract.transfer(to, amount).into_transaction_request()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::{addresses, setup};

    #[tokio::test]
    async fn check_balance_zero() {
        let random_addr =
            addresses::get_address("0x8BB0080aC1006D407dfe84D29013964aCC1b9C00").unwrap();

        let (provider, _) = setup::test_setup().await;

        let balance_random = Executor::new(
            &provider,
            balance_of(&provider, *addresses::WETH, random_addr),
        )
        .call_return_uint()
        .await
        .expect("BALANCE_OF failed");

        assert_eq!(balance_random, U256::from(0));
    }

    #[tokio::test]
    async fn check_balance_whale() {
        let whale_addr =
            addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3").unwrap();
        const EXPECTED: u64 = 236000000000000;

        let (provider, _) = setup::test_setup().await;

        let balance_whale = Executor::new(
            &provider,
            balance_of(&provider, *addresses::USDC_ADDR, whale_addr),
        )
        .call_return_uint()
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_whale, U256::from(EXPECTED));
    }

    #[tokio::test]
    async fn check_approve() {
        let whale_addr =
            addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3").unwrap();
        let amount = U256::from(1e18 as u32);

        let (provider, address) = setup::test_setup().await;

        let approve = Executor::new(
            &provider,
            approve(&provider, *addresses::WETH, whale_addr, amount),
        )
        .send()
        .await;

        assert!(approve.is_ok(), "APPROVE failed");

        // Ensure that allowance is now 1e18
        let allowance = Executor::new(
            &provider,
            allowance(&provider, *addresses::WETH, address, whale_addr),
        )
        .call_return_uint()
        .await
        .expect("ALLOWANCE failed");

        assert_eq!(allowance, amount);
    }
}
