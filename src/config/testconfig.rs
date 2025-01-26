use alloy::signers::local::PrivateKeySigner;
use std::str::FromStr;

const DEFAULT_ENDPOINT: &str = "http://localhost:8545";
const DEFAULT_WS_ENDPOINT: &str = "ws://localhost:8545";

pub struct TestConfig {
    pub alchemy_ws_endpoint: Option<String>,
    pub anvil_endpoint: String,
    pub anvil_ws_endpoint: String,
    pub priv_key: PrivateKeySigner,
}

impl TestConfig {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self {
            alchemy_ws_endpoint: std::env::var("RPC_WS_TEST_URL").ok(),
            anvil_endpoint: std::env::var("ANVIL_ENDPOINT")
                .map_or(DEFAULT_ENDPOINT.to_owned(), |v| v),
            anvil_ws_endpoint: std::env::var("ANVIL_WS_ENDPOINT")
                .map_or(DEFAULT_WS_ENDPOINT.to_owned(), |v| v),
            priv_key: PrivateKeySigner::from_str(
                &std::env::var("TEST_PRIVATE_KEY").expect("TEST_PRIVATE_KEY not set"),
            )
            .expect("Could not parse private key"),
        }
    }
}
