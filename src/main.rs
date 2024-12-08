use ethers::providers::Middleware;
use ethers::types::Address;

use eyre::Result;

#[path = "config.rs"] mod config;
#[path = "uniswap/router.rs"] mod router;
#[path = "wallet.rs"] mod wallet;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup config file
    let config = config::Config::load();

    // Setup provider
    let provider = wallet::create_provider(&config.rpc_url);
    let chain_id = provider.get_chainid().await?;
    
    // provider is now consumed
    let client = wallet::create_signer(provider, &config.priv_key, chain_id.as_u64());

    // Fetch pair address
    let pair = "0xa2107fa5b38d9bbd2c461d6edf11b11a50f6b974".parse::<Address>()?;

    let _ = router::fetch_token0(&client, pair).await?;

    Ok(())
}
