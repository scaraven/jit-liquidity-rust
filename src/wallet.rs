use ethers::signers::{LocalWallet, Signer, SignerMiddleware};
use ethers::providers::Provider;
use std::sync::Arc;

pub fn setup_wallet(private_key: &str, chain_id: i64) -> LocalWallet {
    let wallet = LocalWallet::from_str(private_key)
        .expect("Failed to create wallet from private key")
        .with_chain_id(chain_id.as_u64());
    wallet
}

pub fn create_signer(provider: Provider, private_key: &str, chain_id: i64) -> Arc<Signer> {
    let wallet = setup_wallet(private_key, chain_id);
    Arc::new(SignerMiddlware::new(provider, wallet))
}