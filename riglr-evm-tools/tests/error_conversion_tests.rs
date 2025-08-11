use riglr_evm_tools::error::EvmToolError;
use riglr_core::error::ToolError;

#[test]
fn test_rpc_error_conversion() {
    let err = EvmToolError::Rpc("RPC connection failed".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_invalid_address_conversion() {
    let err = EvmToolError::InvalidAddress("0xinvalid".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_insufficient_balance_conversion() {
    let err = EvmToolError::InsufficientBalance("Need 1 ETH".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_gas_estimation_conversion() {
    let err = EvmToolError::GasEstimation("Failed to estimate gas".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_transaction_failed_conversion() {
    let err = EvmToolError::TransactionFailed("Transaction reverted".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable { .. } => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_contract_error_conversion() {
    let err = EvmToolError::Contract("Contract call failed".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent { .. } => {},
        _ => panic!("Expected Permanent variant"),
    }
}