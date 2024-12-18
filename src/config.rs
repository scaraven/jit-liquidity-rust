pub struct Config {
    pub rpc_url: Option<String>,
    pub priv_key: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self {
            rpc_url: std::env::var("INFURA_URL").ok(),
            priv_key: std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"),
        }
    }
}
