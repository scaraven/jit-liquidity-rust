use alloy::{hex::FromHex, primitives::Address};

use eyre::Result;

use std::sync::LazyLock;

pub static UNISWAP_V2_ROUTER: LazyLock<Address> = LazyLock::new(|| {
    "0x7a250d5630b4cf539739df2c5dacb4c659f2488d"
        .parse()
        .unwrap()
});
pub static UNISWAP_V3_ROUTER: LazyLock<Address> = LazyLock::new(|| {
    "0xe592427a0aece92de3edee1f18e0157c05861564"
        .parse()
        .unwrap()
});

pub static WETH: LazyLock<Address> = LazyLock::new(|| {
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
        .parse()
        .unwrap()
});
pub static USDC_WBTC_PAIR: LazyLock<Address> = LazyLock::new(|| {
    "0x004375dff511095cc5a197a54140a24efef3a416"
        .parse()
        .unwrap()
});
pub static WBTC: LazyLock<Address> = LazyLock::new(|| {
    "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
        .parse()
        .unwrap()
});
pub static WETH_USDC_PAIR: LazyLock<Address> = LazyLock::new(|| {
    "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc"
        .parse()
        .unwrap()
});
pub static USDC_ADDR: LazyLock<Address> = LazyLock::new(|| {
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        .parse()
        .unwrap()
});

// Should not panic
pub fn get_address(address: &str) -> Result<Address> {
    Address::from_hex(address).map_err(|e| e.into())
}
