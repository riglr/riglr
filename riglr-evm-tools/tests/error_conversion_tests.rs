use riglr_evm_tools::error::EvmToolError;
use riglr_core::error::ToolError;

#[test]
fn test_rpc_error_conversion() {
    let err = EvmToolError::Rpc("RPC connection failed".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable(_) => {},
        _ => panic!("Expected Retriable variant"),
    }
}

#[test]
fn test_invalid_address_conversion() {
    let err = EvmToolError::InvalidAddress("0xinvalid".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent(_) => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_generic_error_conversion() {
    let err = EvmToolError::Generic("Need 1 ETH".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent(_) => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_rpc_error_is_retriable() {
    // Test that RPC errors are classified as retriable
    let err = EvmToolError::Rpc("Connection timeout".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Retriable(_) => {},
        _ => panic!("Expected Retriable variant for RPC errors"),
    }
}

#[test]
fn test_transaction_error_conversion() {
    let err = EvmToolError::Transaction("Transaction reverted".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent(_) => {},
        _ => panic!("Expected Permanent variant"),
    }
}

#[test]
fn test_contract_error_conversion() {
    let err = EvmToolError::Contract("Contract call failed".to_string());
    let tool_err: ToolError = err.into();
    
    match tool_err {
        ToolError::Permanent(_) => {},
        _ => panic!("Expected Permanent variant"),
    }
}