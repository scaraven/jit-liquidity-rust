use ethers::core::types::Address;

pub const UNISWAP_V2_ROUTER: &str = "0x7a250d5630b4cf539739df2c5dacb4c659f2488d";
pub const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

pub fn get_address(address: &str) -> Address {
    address.parse::<Address>().unwrap()
}
