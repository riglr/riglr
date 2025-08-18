//! EVM conversion utilities

use alloy::primitives::U256;

/// Convert ETH to wei
pub fn eth_to_wei(eth_amount: f64) -> U256 {
    let wei_amount = eth_amount * 1e18;
    U256::from(wei_amount as u64)
}

/// Convert wei to ETH
pub fn wei_to_eth(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    wei_str.parse::<f64>().unwrap_or(0.0) / 1e18
}

/// Convert gwei to wei
pub fn gwei_to_wei(gwei_amount: f64) -> U256 {
    let wei_amount = gwei_amount * 1e9;
    U256::from(wei_amount as u64)
}

/// Convert wei to gwei
pub fn wei_to_gwei(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    wei_str.parse::<f64>().unwrap_or(0.0) / 1e9
}

/// Convert token amount to smallest unit based on decimals
pub fn token_to_smallest_unit(amount: f64, decimals: u8) -> U256 {
    let multiplier = 10_f64.powi(decimals as i32);
    let smallest_unit = amount * multiplier;
    U256::from(smallest_unit as u64)
}

/// Convert smallest unit to token amount based on decimals
pub fn smallest_unit_to_token(amount: U256, decimals: u8) -> f64 {
    let divisor = 10_f64.powi(decimals as i32);
    let amount_str = amount.to_string();
    amount_str.parse::<f64>().unwrap_or(0.0) / divisor
}
