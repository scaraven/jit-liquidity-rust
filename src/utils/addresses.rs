use ethers::core::types::Address;

pub const UNISWAP_V2_ROUTER: &str = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d";
pub const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub const USDC_WBTC_PAIR: &str = "0x004375dff511095cc5a197a54140a24efef3a416";
pub const WBTC: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
pub const WETH_USDC_PAIR: &str = "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc";
pub const USDC_ADDR: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

pub fn get_address(address: &str) -> Address {
    address.parse::<Address>().unwrap()
}
