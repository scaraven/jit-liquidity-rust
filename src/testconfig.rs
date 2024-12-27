use alloy::signers::local::PrivateKeySigner;
use std::str::FromStr;

const DEFAULT_ENDPOINT: &str = "http://localhost:8545";

pub struct TestConfig {
    pub anvil_endpoint: String,
    pub priv_key: PrivateKeySigner,
}

impl TestConfig {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self {
            anvil_endpoint: std::env::var("ANVIL_ENDPOINT")
                .map_or(DEFAULT_ENDPOINT.to_owned(), |v| v),
            priv_key: PrivateKeySigner::from_str(
                &std::env::var("TEST_PRIVATE_KEY").expect("TEST_PRIVATE_KEY not set"),
            )
            .expect("Could not parse private key"),
        }
    }
}
