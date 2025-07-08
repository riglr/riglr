//! Generic smart contract interaction tools

use crate::{client::EvmClient, error::{EvmToolError, Result}};
use alloy::contract::{ContractInstance, Interface};
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::json_abi::{Function, JsonAbi};
use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolCall;
use alloy::transports::http::{Client, Http};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

/// Standard ERC20 ABI for common operations
const ERC20_ABI: &str = r#"[
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

/// Parse ABI from JSON string or use default ERC20 ABI
fn parse_abi(abi_json: Option<&str>) -> Result<JsonAbi> {
    let abi_str = abi_json.unwrap_or(ERC20_ABI);
    serde_json::from_str(abi_str)
        .map_err(|e| EvmToolError::Generic(format!("Failed to parse ABI: {}", e)))
}

/// Parse parameters based on expected types
fn parse_params(params: Vec<String>, param_types: &[DynSolType]) -> Result<Vec<DynSolValue>> {
    if params.len() != param_types.len() {
        return Err(EvmToolError::Generic(format!(
            "Expected {} parameters, got {}",
            param_types.len(),
            params.len()
        )));
    }

    params
        .iter()
        .zip(param_types.iter())
        .map(|(param, param_type)| {
            match param_type {
                DynSolType::Address => {
                    let addr = Address::from_str(param)
                        .map_err(|e| EvmToolError::Generic(format!("Invalid address: {}", e)))?;
                    Ok(DynSolValue::Address(addr))
                }
                DynSolType::Uint(bits) => {
                    let value = U256::from_str(param)
                        .map_err(|e| EvmToolError::Generic(format!("Invalid uint: {}", e)))?;
                    Ok(DynSolValue::Uint(value, *bits))
                }
                DynSolType::Int(bits) => {
                    let value = param.parse::<i128>()
                        .map_err(|e| EvmToolError::Generic(format!("Invalid int: {}", e)))?;
                    Ok(DynSolValue::Int(U256::from(value as u128), *bits))
                }
                DynSolType::Bool => {
                    let value = param.parse::<bool>()
                        .map_err(|e| EvmToolError::Generic(format!("Invalid bool: {}", e)))?;
                    Ok(DynSolValue::Bool(value))
                }
                DynSolType::String => Ok(DynSolValue::String(param.clone())),
                DynSolType::Bytes => {
                    let bytes = hex::decode(param.trim_start_matches("0x"))
                        .map_err(|e| EvmToolError::Generic(format!("Invalid bytes: {}", e)))?;
                    Ok(DynSolValue::Bytes(bytes))
                }
                DynSolType::FixedBytes(size) => {
                    let bytes = hex::decode(param.trim_start_matches("0x"))
                        .map_err(|e| EvmToolError::Generic(format!("Invalid fixed bytes: {}", e)))?;
                    if bytes.len() != *size {
                        return Err(EvmToolError::Generic(format!(
                            "Expected {} bytes, got {}",
                            size,
                            bytes.len()
                        )));
                    }
                    let mut fixed_bytes = vec![0u8; *size];
                    fixed_bytes.copy_from_slice(&bytes);
                    Ok(DynSolValue::FixedBytes(FixedBytes::from_slice(&fixed_bytes), *size))
                }
                _ => Err(EvmToolError::Generic(format!(
                    "Unsupported parameter type: {:?}",
                    param_type
                )))
            }
        })
        .collect()
}

/// Convert DynSolValue to JSON
fn dyn_sol_value_to_json(value: &DynSolValue) -> Value {
    match value {
        DynSolValue::Address(addr) => Value::String(format!("0x{:x}", addr)),
        DynSolValue::Uint(val, _) => Value::String(val.to_string()),
        DynSolValue::Int(val, _) => Value::String(val.to_string()),
        DynSolValue::Bool(b) => Value::Bool(*b),
        DynSolValue::String(s) => Value::String(s.clone()),
        DynSolValue::Bytes(b) => Value::String(format!("0x{}", hex::encode(b))),
        DynSolValue::FixedBytes(b, _) => Value::String(format!("0x{}", hex::encode(b.as_slice()))),
        DynSolValue::Array(arr) => Value::Array(arr.iter().map(dyn_sol_value_to_json).collect()),
        DynSolValue::Tuple(tuple) => Value::Array(tuple.iter().map(dyn_sol_value_to_json).collect()),
        _ => Value::Null,
    }
}

/// Call a contract read function (view/pure function)
pub async fn call_contract_read(
    client: &EvmClient,
    contract_address: &str,
    function: &str,
    params: Vec<String>,
    abi_json: Option<&str>,
) -> Result<serde_json::Value> {
    debug!(
        "Calling contract read function {} at {} with params: {:?}",
        function, contract_address, params
    );

    // Parse contract address
    let address = Address::from_str(contract_address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid contract address: {}", e)))?;

    // Parse ABI
    let abi = parse_abi(abi_json)?;
    
    // Find the function in the ABI
    let func = abi
        .functions()
        .find(|f| f.name == function)
        .ok_or_else(|| EvmToolError::Generic(format!("Function {} not found in ABI", function)))?
        .clone();

    // Parse parameters
    let param_types: Vec<DynSolType> = func
        .inputs
        .iter()
        .map(|input| DynSolType::parse(&input.ty))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| EvmToolError::Generic(format!("Failed to parse parameter types: {}", e)))?;
    
    let parsed_params = parse_params(params, &param_types)?;

    // Encode function call
    let selector = func.selector();
    let encoded_params = alloy::dyn_abi::encode_packed(&parsed_params)
        .map_err(|e| EvmToolError::Generic(format!("Failed to encode parameters: {}", e)))?;
    
    let mut call_data = Vec::with_capacity(4 + encoded_params.len());
    call_data.extend_from_slice(&selector);
    call_data.extend_from_slice(&encoded_params);

    // Create transaction request for the call
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(call_data.into());

    // Call the contract
    let result = client
        .provider()
        .call(&tx)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Contract call failed: {}", e)))?;

    // Decode the result
    let return_types: Vec<DynSolType> = func
        .outputs
        .iter()
        .map(|output| DynSolType::parse(&output.ty))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| EvmToolError::Generic(format!("Failed to parse return types: {}", e)))?;

    if return_types.is_empty() {
        return Ok(Value::Null);
    }

    let decoded = alloy::dyn_abi::decode(&return_types, &result)
        .map_err(|e| EvmToolError::Generic(format!("Failed to decode result: {}", e)))?;

    // Convert to JSON
    let json_result = if decoded.len() == 1 {
        dyn_sol_value_to_json(&decoded[0])
    } else {
        Value::Array(decoded.iter().map(dyn_sol_value_to_json).collect())
    };

    info!(
        "Contract read successful: {}::{}",
        contract_address, function
    );

    Ok(json_result)
}

/// Call a contract write function (state-mutating function)
pub async fn call_contract_write(
    client: &EvmClient,
    contract_address: &str,
    function: &str,
    params: Vec<String>,
    abi_json: Option<&str>,
    gas_limit: Option<u128>,
) -> Result<String> {
    debug!(
        "Calling contract write function {} at {} with params: {:?}",
        function, contract_address, params
    );

    // Get signer from client
    let signer = client.require_signer()?;
    
    // Parse contract address
    let address = Address::from_str(contract_address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid contract address: {}", e)))?;

    // Parse ABI
    let abi = parse_abi(abi_json)?;
    
    // Find the function in the ABI
    let func = abi
        .functions()
        .find(|f| f.name == function)
        .ok_or_else(|| EvmToolError::Generic(format!("Function {} not found in ABI", function)))?
        .clone();

    // Parse parameters
    let param_types: Vec<DynSolType> = func
        .inputs
        .iter()
        .map(|input| DynSolType::parse(&input.ty))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| EvmToolError::Generic(format!("Failed to parse parameter types: {}", e)))?;
    
    let parsed_params = parse_params(params, &param_types)?;

    // Encode function call
    let selector = func.selector();
    let encoded_params = alloy::dyn_abi::encode_packed(&parsed_params)
        .map_err(|e| EvmToolError::Generic(format!("Failed to encode parameters: {}", e)))?;
    
    let mut call_data = Vec::with_capacity(4 + encoded_params.len());
    call_data.extend_from_slice(&selector);
    call_data.extend_from_slice(&encoded_params);

    // Create wallet from signer
    let wallet = EthereumWallet::from(signer);
    
    // Create a new provider with the wallet
    let provider_with_wallet = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(client.rpc_url.parse().unwrap());

    // Create transaction request
    let mut tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(call_data.into());

    // Set gas limit if provided
    if let Some(gas) = gas_limit {
        tx = tx.gas_limit(gas);
    }

    // Estimate gas if not provided
    if gas_limit.is_none() {
        let estimated_gas = provider_with_wallet
            .estimate_gas(&tx)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to estimate gas: {}", e)))?;
        
        // Add 20% buffer to gas estimate
        let gas_with_buffer = estimated_gas * 120 / 100;
        tx = tx.gas_limit(gas_with_buffer);
    }

    // Send transaction
    let pending_tx = provider_with_wallet
        .send_transaction(tx)
        .await
        .map_err(|e| EvmToolError::Transaction(format!("Failed to send transaction: {}", e)))?;

    let tx_hash = pending_tx.tx_hash();
    info!(
        "Contract write transaction sent: {}::{} - Hash: 0x{:x}",
        contract_address, function, tx_hash
    );

    // Wait for confirmation
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| EvmToolError::Transaction(format!("Failed to get receipt: {}", e)))?;

    if !receipt.status() {
        return Err(EvmToolError::Transaction(
            "Transaction failed on-chain".to_string(),
        ));
    }

    Ok(format!("0x{:x}", tx_hash))
}

/// Read ERC20 token information
pub async fn read_erc20_info(
    client: &EvmClient,
    token_address: &str,
) -> Result<serde_json::Value> {
    let name = call_contract_read(client, token_address, "name", vec![], Some(ERC20_ABI)).await?;
    let symbol = call_contract_read(client, token_address, "symbol", vec![], Some(ERC20_ABI)).await?;
    let decimals = call_contract_read(client, token_address, "decimals", vec![], Some(ERC20_ABI)).await?;
    let total_supply = call_contract_read(client, token_address, "totalSupply", vec![], Some(ERC20_ABI)).await?;

    Ok(serde_json::json!({
        "address": token_address,
        "name": name,
        "symbol": symbol,
        "decimals": decimals,
        "totalSupply": total_supply
    }))
}
