use ethers::providers::{Provider, Http};

pub fn setup_provider(rpc_url: &str) -> Provider<Http> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .expect("Failed to connect to Ethereum node");
}