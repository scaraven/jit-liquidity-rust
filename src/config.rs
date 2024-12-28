use std::str::FromStr;

use alloy::{primitives::Address, signers::local::PrivateKeySigner};

pub struct Config {
    pub rpc_url: Option<String>,
    pub signer: PrivateKeySigner,
    pub address: Address,
    pub anvil_path: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        let key = PrivateKeySigner::from_str(
            &std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"),
        )
        .expect("Could not parse private key");

        let addr = key.address();

        Self {
            rpc_url: std::env::var("INFURA_URL").ok(),
            signer: key,
            address: addr,
            anvil_path: std::env::var("ANVIL_PATH").ok(),
        }
    }
}
