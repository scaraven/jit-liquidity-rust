pub struct Config {
    pub rpc_url: String,
    pub priv_key: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        Self {
            rpc_url: std::env::var("RPC_URL").expect("RPC_URL not set"),
            priv_key: std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"),
        }
    }
}
