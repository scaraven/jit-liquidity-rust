use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::TransactionReceipt,
};

use eyre::Result;

abigen!(
    ERC20Token,
    r"[
        approve(address spender, uint256 amount) external returns (bool)
        transfer(address recipient, uint256 amount) external returns (bool)
        transferFrom(address sender, address recipient, uint256 amount) external returns (bool)
        balanceOf(address account) external view returns (uint256)
        allowance(address owner, address spender) external view returns (uint256)
    ]"
);

fn create_erc20(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
) -> Result<ERC20Token<SignerMiddleware<Provider<Http>, LocalWallet>>> {
    Ok(ERC20Token::new(token, client.clone()))
}

pub async fn approve(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
    spender: Address,
    amount: U256,
) -> Result<TransactionReceipt> {
    let contract = create_erc20(client, token).unwrap();

    let receipt_option = contract
        .approve(spender, amount)
        .send()
        .await?
        .await
        .unwrap();

    // Checke whether we got an approve event
    Ok(receipt_option.unwrap())
}

pub async fn balance_of(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
    spender: Address,
) -> Result<U256> {
    let contract = create_erc20(client, token).unwrap();

    let balance = contract.balance_of(spender).call().await?;

    Ok(U256::from(balance))
}

pub async fn allowance(
    client: &Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<U256> {
    let contract = create_erc20(client, token).unwrap();

    let allowance = contract.allowance(owner, spender).call().await?;

    Ok(U256::from(allowance))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::addresses;
    use crate::setup;

    #[tokio::test]
    async fn check_balance_zero() {
        let random_addr = addresses::get_address("0x8BB0080aC1006D407dfe84D29013964aCC1b9C00");

        let (_provider, client) = setup::test_setup().await;

        let balance_random = balance_of(
            &client,
            addresses::get_address(addresses::WETH),
            random_addr,
        )
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_random.as_u64(), 0);
    }

    #[tokio::test]
    async fn check_balance_whale() {
        let whale_addr = addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3");

        let (_provider, client) = setup::test_setup().await;

        let balance_whale = balance_of(
            &client,
            addresses::get_address(addresses::USDC_ADDR),
            whale_addr,
        )
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_whale.as_u64(), 236000000000000);
    }

    #[tokio::test]
    async fn check_approve() {
        let whale_addr = addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3");
        let amount = U256::from(1e18 as u32);

        let (_provider, client) = setup::test_setup().await;

        let receipt = approve(
            &client,
            addresses::get_address(addresses::WETH),
            whale_addr,
            amount,
        )
        .await
        .expect("APPROVE failed");

        // Fetch approval
        assert_eq!(receipt.status.expect("APPROVE reverted"), (1 as u64).into());

        // Ensure that allowance is now 1e18
        let allowance = allowance(
            &client,
            addresses::get_address(addresses::WETH),
            client.address(),
            whale_addr,
        )
        .await
        .expect("ALLOWANCE failed");

        assert_eq!(allowance, amount);
    }
}
