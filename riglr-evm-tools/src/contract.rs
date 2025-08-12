//! Generic smart contract interaction tools

// use crate::error::Result;
use riglr_macros::tool;
use tracing::{debug, info};

/// Standard ERC20 ABI for common operations
const _ERC20_ABI: &str = r#"[
    {"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"type":"function"},
    {"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"type":"function"},
    {"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"type":"function"},
    {"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"","type":"uint256"}],"type":"function"},
    {"constant":true,"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"type":"function"},
    {"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"","type":"uint256"}],"type":"function"},
    {"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"","type":"bool"}],"type":"function"},
    {"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"","type":"bool"}],"type":"function"},
    {"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"","type":"bool"}],"type":"function"}
]"#;

/// Call a contract read function (view/pure function)
#[tool]
pub async fn call_contract_read(
    contract_address: String,
    function_selector: String,
    params: Vec<String>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "Calling contract read function {} at {} with params: {:?}",
        function_selector, contract_address, params
    );

    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current().await
        .map_err(|e| format!("No signer context: {}", e))?;
    let client = signer.evm_client()
        .map_err(|e| format!("Failed to get EVM client: {}", e))?;

    // Validate contract address
    let contract_addr = contract_address.parse::<alloy::primitives::Address>()
        .map_err(|e| format!("Invalid contract address: {}", e))?;

    // For now, return a simple call without full ABI parsing
    // In a full implementation, this would parse the ABI and encode parameters
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(contract_addr)
        .input(alloy::primitives::hex::decode(&function_selector)
            .map_err(|e| format!("Invalid function selector: {}", e))?.into());

    let result = client.call(&tx).await
        .map_err(|e| format!("Contract call failed: {}", e))?;

    info!(
        "Contract read successful: {}::{}",
        contract_address, function_selector
    );

    Ok(format!("0x{}", alloy::primitives::hex::encode(result)))
}

/// Call a contract write function (state-mutating function)
#[tool]
pub async fn call_contract_write(
    contract_address: String,
    function_selector: String,
    params: Vec<String>,
    gas_limit: Option<u64>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    debug!(
        "Calling contract write function {} at {} with params: {:?}",
        function_selector, contract_address, params
    );

    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current().await
        .map_err(|e| format!("No signer context: {}", e))?;
    let _client = signer.evm_client()
        .map_err(|e| format!("Failed to get EVM client: {}", e))?;

    // Get signer address
    let from_addr_str = signer.address()
        .ok_or_else(|| "Signer has no address")?;
    let from_addr = from_addr_str.parse::<alloy::primitives::Address>()
        .map_err(|e| format!("Invalid signer address: {}", e))?;

    // Validate contract address
    let contract_addr = contract_address.parse::<alloy::primitives::Address>()
        .map_err(|e| format!("Invalid contract address: {}", e))?;

    // Build transaction
    let tx = alloy::rpc::types::TransactionRequest::default()
        .from(from_addr)
        .to(contract_addr)
        .input(alloy::primitives::hex::decode(&function_selector)
            .map_err(|e| format!("Invalid function selector: {}", e))?.into())
        .gas_limit(gas_limit.unwrap_or(200000));

    // Sign and send transaction
    let tx_hash = signer.sign_and_send_evm_transaction(tx).await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;

    info!(
        "Contract write transaction sent: {}::{} - Hash: {}",
        contract_address, function_selector, tx_hash
    );

    Ok(tx_hash)
}

/// Read ERC20 token information
#[tool]
pub async fn read_erc20_info(
    token_address: String,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    // Get signer context and EVM client
    let signer = riglr_core::SignerContext::current().await
        .map_err(|e| format!("No signer context: {}", e))?;
    let client = signer.evm_client()
        .map_err(|e| format!("Failed to get EVM client: {}", e))?;

    let token_addr = token_address.parse::<alloy::primitives::Address>()
        .map_err(|e| format!("Invalid token address: {}", e))?;

    // Use helper functions from balance module
    use crate::balance::{get_token_symbol, get_token_name, get_token_decimals};
    
    let symbol = get_token_symbol(&*client, token_addr).await.ok();
    let name = get_token_name(&*client, token_addr).await.ok();
    let decimals = get_token_decimals(&*client, token_addr).await.unwrap_or(18);

    Ok(serde_json::json!({
        "address": token_address,
        "name": name,
        "symbol": symbol,
        "decimals": decimals,
        "totalSupply": "unknown" // Would need additional implementation
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // This test just ensures the module compiles
        // No assertion needed - compilation itself is the test
    }
}
