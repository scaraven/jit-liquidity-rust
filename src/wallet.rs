use ethers::signers::{LocalWallet, Signer};
use ethers::providers::{Http, Provider};
use ethers::middleware::SignerMiddleware;
use std::sync::Arc;


fn setup_wallet(private_key: &str, chain_id: u64) -> LocalWallet {
    let wallet = private_key.parse::<LocalWallet>()
    .expect("Cannot parse private key")
    .with_chain_id(chain_id);
    wallet
}

pub fn create_signer(provider: Provider<Http>, private_key: &str, chain_id: u64) -> Arc<SignerMiddleware<Provider<Http>, LocalWallet>> {
    let wallet = setup_wallet(private_key, chain_id);
    Arc::new(SignerMiddleware::new(provider, wallet))
}

pub fn create_provider(rpc_url: &str) -> Provider<Http> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .expect("Failed to connect to Ethereum node");
    provider
}