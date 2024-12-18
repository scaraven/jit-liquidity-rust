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
