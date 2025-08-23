//! EVM formatting utilities

use alloy::primitives::U256;

/// Format wei amount to ETH with specified decimal places
pub fn format_wei_to_eth(wei_amount: U256, decimals: usize) -> String {
    // Convert U256 to string first, then parse as f64
    let wei_str = wei_amount.to_string();
    let eth_value = wei_str.parse::<f64>().unwrap_or(0.0) / 1e18;
    format!("{:.prec$} ETH", eth_value, prec = decimals)
}

/// Format token amount with decimals
pub fn format_token_amount(amount: U256, token_decimals: u8, symbol: &str) -> String {
    let divisor = U256::from(10).pow(U256::from(token_decimals));
    let amount_str = amount.to_string();
    let divisor_str = divisor.to_string();
    let formatted_amount =
        amount_str.parse::<f64>().unwrap_or(0.0) / divisor_str.parse::<f64>().unwrap_or(1.0);
    format!("{:.6} {}", formatted_amount, symbol)
}

/// Format gas price in gwei
pub fn format_gas_price_gwei(gas_price_wei: U256) -> String {
    let wei_str = gas_price_wei.to_string();
    let gwei = wei_str.parse::<f64>().unwrap_or(0.0) / 1e9;
    format!("{:.2} gwei", gwei)
}

/// Truncate address for display
pub fn truncate_address(address: &str, prefix_len: usize, suffix_len: usize) -> String {
    if address.len() <= prefix_len + suffix_len {
        return address.to_string();
    }

    let prefix = &address[..prefix_len];
    let suffix = &address[address.len() - suffix_len..];
    format!("{}...{}", prefix, suffix)
}
