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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::addresses;
    use crate::config;
    use crate::utils;

    #[tokio::test]
    async fn check_balance_zero() {
        let random_addr = addresses::get_address("0x8BB0080aC1006D407dfe84D29013964aCC1b9C00");
        let whale_addr = addresses::get_address("0xD6c32E35D6A169C77786430ac7b257fF6bb480C3");

        let config = config::Config::load();
        let (_provider, client, _anvil) = utils::setup(config).await.expect("UTIL_SETUP failed");

        let balance_random = balance_of(
            &client,
            addresses::get_address(addresses::WETH),
            random_addr,
        )
        .await
        .expect("BALANCE_OF failed");
        assert_eq!(balance_random.as_u64(), 0);

        let balance_check = balance_of(
            &client,
            addresses::get_address(addresses::USDC_ADDR),
            whale_addr,
        )
        .await
        .expect("BALANCE_OF failed");

        // Balance of the whale
        assert_eq!(balance_check.as_u64(), 236000000000000);
    }
}
