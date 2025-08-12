//! Generic smart contract interaction tools

// use crate::error::Result;
use riglr_macros::tool;
use alloy::primitives::{self, Address, Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;
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

    // Encode calldata = 4-byte selector + encoded params (static types or pre-encoded words)
    let calldata = encode_selector_and_params(&function_selector, &params)?;

    let tx = TransactionRequest::default()
        .to(contract_addr)
        .input(calldata.into());

    let result = client.call(&tx).await
        .map_err(|e| format!("Contract call failed: {}", e))?;

    info!(
        "Contract read successful: {}::{}",
        contract_address, function_selector
    );

    // Best-effort decode: if exactly 32 bytes, treat as uint256 decimal; otherwise hex
    if result.len() == 32 {
        if let Some(v) = U256::try_from_be_slice(&result) {
            return Ok(v.to_string());
        }
    }
    Ok(format!("0x{}", primitives::hex::encode(result)))
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

    // Build transaction with encoded calldata
    let calldata = encode_selector_and_params(&function_selector, &params)?;

    let tx = TransactionRequest::default()
        .from(from_addr)
        .to(contract_addr)
        .input(calldata.into())
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

    // totalSupply()
    let total_supply = {
        // 0x18160ddd = keccak256("totalSupply()") first 4 bytes
        let selector = Bytes::from(primitives::hex::decode("18160ddd").map_err(|e| format!("selector decode: {}", e))?);
        let tx = TransactionRequest::default()
            .to(token_addr)
            .input(selector.into());
        match client.call(&tx).await {
            Ok(bytes) => U256::try_from_be_slice(&bytes).map(|v| v.to_string()).unwrap_or_default(),
            Err(_) => String::new(),
        }
    };

    Ok(serde_json::json!({
        "address": token_address,
        "name": name,
        "symbol": symbol,
        "decimals": decimals,
        "totalSupply": total_supply
    }))
}

/// Encode a 4-byte function selector and parameters into calldata.
/// Params are heuristically encoded:
/// - If starts with 0x and length is 42 (address), left-padded to 32 bytes
/// - If decimal number, encoded as uint256
/// - If starts with 0x and length is 66 (32 bytes), treated as one ABI word
/// - Otherwise, if starts with 0x, raw bytes are appended (advanced use: pre-encoded tail)
fn encode_selector_and_params(
    selector_hex: &str,
    params: &Vec<String>,
) -> Result<Bytes, Box<dyn std::error::Error + Send + Sync>> {
    let mut data: Vec<u8> = Vec::new();
    let sel = selector_hex.strip_prefix("0x").unwrap_or(selector_hex);
    let selector_bytes = primitives::hex::decode(sel)
        .map_err(|e| format!("Invalid function selector: {}", e))?;
    if selector_bytes.len() != 4 {
        return Err(format!("Function selector must be 4 bytes (8 hex chars), got {} bytes", selector_bytes.len()).into());
    }
    data.extend_from_slice(&selector_bytes);

    for p in params {
        let encoded = encode_param_word(p)?;
        data.extend_from_slice(&encoded);
    }

    Ok(Bytes::from(data))
}

fn encode_param_word(p: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Address hex
    if p.starts_with("0x") && p.len() == 42 {
        let addr = Address::from_str(p)?;
        let mut word = vec![0u8; 32];
        let be = addr.as_slice();
        // address is 20 bytes, left pad to 32
        word[12..].copy_from_slice(be);
        return Ok(word);
    }

    // 32-byte word hex
    if p.starts_with("0x") && p.len() == 66 {
        let bytes = primitives::hex::decode(&p[2..])?;
        return Ok(bytes);
    }

    // Hex arbitrary length: treat as already-encoded bytes (advanced use)
    if p.starts_with("0x") {
        let bytes = primitives::hex::decode(&p[2..])?;
        return Ok(bytes);
    }

    // Decimal number -> uint256
    if p.chars().all(|c| c.is_ascii_digit()) {
        let v = U256::from_str_radix(p, 10)?;
    let buf: [u8; 32] = v.to_be_bytes();
    return Ok(buf.to_vec());
    }

    Err(format!("Unsupported param format: {}. Use 0x-prefixed hex (address or 32-byte word) or decimal uint.", p).into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // This test just ensures the module compiles
        // No assertion needed - compilation itself is the test
    }
}
