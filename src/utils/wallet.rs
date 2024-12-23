use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, JsonRpcClient, Provider};
use ethers::signers::{LocalWallet, Signer};
use std::sync::Arc;
use std::time::Duration;

pub fn setup_wallet(private_key: &str, chain_id: u64) -> LocalWallet {
    private_key
        .parse::<LocalWallet>()
        .expect("Cannot parse private key")
        .with_chain_id(chain_id)
}

pub fn create_signer<C: JsonRpcClient>(
    provider: Provider<C>,
    wallet: LocalWallet,
) -> Arc<SignerMiddleware<Provider<C>, LocalWallet>> {
    Arc::new(SignerMiddleware::new(provider, wallet))
}

pub fn create_provider(rpc_url: &str) -> Provider<Http> {
    Provider::try_from(rpc_url)
        .expect("Failed to connect to Ethereum node")
        .interval(Duration::from_millis(10u64))
}
