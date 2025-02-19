use std::str::FromStr;

use alloy::{primitives::Address, signers::local::PrivateKeySigner};

pub struct Config {
    pub rpc_url: Option<String>,
    pub rpc_url_ws: Option<String>,
    pub signer: PrivateKeySigner,
    pub address: Address,
    pub flashbot_signer: Option<PrivateKeySigner>,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();

        // Get private key, error if not set
        let key = PrivateKeySigner::from_str(
            &std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"),
        )
        .expect("Could not parse private key");

        // Get flashbot key for signing bundles
        let flashbot_key = std::env::var("FLASHBOT_PRIVATE_KEY").ok().map(|key| {
            PrivateKeySigner::from_str(&key).expect("Could not parse flashbot private key")
        });

        let addr = key.address();

        Self {
            rpc_url: std::env::var("SEPOLIA_RPC_URL").ok(),
            rpc_url_ws: std::env::var("SEPOLIA_RPC_WS_URL").ok(),
            signer: key,
            address: addr,
            flashbot_signer: flashbot_key,
        }
    }
}
