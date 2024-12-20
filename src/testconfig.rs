pub struct TestConfig {
    pub anvil_endpoint: Option<String>,
    pub priv_key: String,
}

impl TestConfig {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self {
            anvil_endpoint: std::env::var("ANVIL_ENDPOINT").ok(),
            priv_key: std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"),
        }
    }
}
