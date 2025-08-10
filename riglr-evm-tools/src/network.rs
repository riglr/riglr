//! Network and blockchain query tools for EVM chains

use crate::{client::EvmClient, error::{EvmToolError, Result}};
use alloy::primitives::{Address, FixedBytes, B256};
use alloy::providers::Provider;
use alloy::rpc::types::{Block, BlockId, BlockNumberOrTag, Transaction, TransactionReceipt};
use std::str::FromStr;
use tracing::{debug, info};

/// Get the current block number
pub async fn get_block_number(client: &EvmClient) -> Result<u64> {
    debug!("Getting current block number");
    
    let block_number = client
        .provider()
        .get_block_number()
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get block number: {}", e)))?;
    
    info!("Current block number: {}", block_number);
    Ok(block_number)
}

/// Get a specific block by number or tag
pub async fn get_block(
    client: &EvmClient,
    block: BlockNumberOrTag,
    include_txs: bool,
) -> Result<serde_json::Value> {
    debug!("Getting block: {:?}", block);
    
    let block_data = if include_txs {
        client
            .provider()
            .get_block_by_number(block, true)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get block: {}", e)))?
    } else {
        client
            .provider()
            .get_block_by_number(block, false)
            .await
            .map_err(|e| EvmToolError::Rpc(format!("Failed to get block: {}", e)))?
    };
    
    match block_data {
        Some(block) => {
            let json = serde_json::json!({
                "number": block.header.number,
                "hash": format!("0x{:x}", block.header.hash),
                "parentHash": format!("0x{:x}", block.header.parent_hash),
                "timestamp": block.header.timestamp,
                "gasLimit": block.header.gas_limit.to_string(),
                "gasUsed": block.header.gas_used.to_string(),
                "baseFeePerGas": block.header.base_fee_per_gas.map(|v| v.to_string()),
                "difficulty": block.header.difficulty.to_string(),
                "totalDifficulty": block.header.total_difficulty.map(|v| v.to_string()),
                "miner": format!("0x{:x}", block.header.miner),
                "transactionCount": block.transactions.len(),
            });
            Ok(json)
        }
        None => Err(EvmToolError::Generic("Block not found".to_string())),
    }
}

/// Get a transaction receipt by hash
pub async fn get_transaction_receipt(
    client: &EvmClient,
    tx_hash: &str,
) -> Result<serde_json::Value> {
    debug!("Getting transaction receipt for: {}", tx_hash);
    
    // Parse transaction hash
    let hash = B256::from_str(tx_hash.trim_start_matches("0x"))
        .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash: {}", e)))?;
    
    // Get receipt
    let receipt = client
        .provider()
        .get_transaction_receipt(hash)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get receipt: {}", e)))?;
    
    match receipt {
        Some(receipt) => {
            let json = serde_json::json!({
                "transactionHash": format!("0x{:x}", receipt.transaction_hash),
                "blockHash": receipt.block_hash.map(|h| format!("0x{:x}", h)),
                "blockNumber": receipt.block_number,
                "transactionIndex": receipt.transaction_index,
                "from": format!("0x{:x}", receipt.from),
                "to": receipt.to.map(|a| format!("0x{:x}", a)),
                "contractAddress": receipt.contract_address.map(|a| format!("0x{:x}", a)),
                "gasUsed": receipt.gas_used.to_string(),
                "effectiveGasPrice": receipt.effective_gas_price.to_string(),
                "cumulativeGasUsed": receipt.cumulative_gas_used.to_string(),
                "status": receipt.status(),
                "logsBloom": format!("0x{}", hex::encode(receipt.logs_bloom.as_bytes())),
                "logs": receipt.inner.logs().iter().map(|log| {
                    serde_json::json!({
                        "address": format!("0x{:x}", log.address()),
                        "topics": log.topics().iter().map(|t| format!("0x{:x}", t)).collect::<Vec<_>>(),
                        "data": format!("0x{}", hex::encode(log.data().data.as_ref())),
                        "blockNumber": log.block_number,
                        "transactionHash": log.transaction_hash.map(|h| format!("0x{:x}", h)),
                        "transactionIndex": log.transaction_index,
                        "logIndex": log.log_index,
                        "removed": log.removed,
                    })
                }).collect::<Vec<_>>(),
            });
            
            info!("Transaction receipt retrieved for: {}", tx_hash);
            Ok(json)
        }
        None => Err(EvmToolError::Generic(format!(
            "Transaction receipt not found for: {}",
            tx_hash
        ))),
    }
}

/// Get a transaction by hash
pub async fn get_transaction(
    client: &EvmClient,
    tx_hash: &str,
) -> Result<serde_json::Value> {
    debug!("Getting transaction: {}", tx_hash);
    
    // Parse transaction hash
    let hash = B256::from_str(tx_hash.trim_start_matches("0x"))
        .map_err(|e| EvmToolError::Generic(format!("Invalid transaction hash: {}", e)))?;
    
    // Get transaction
    let tx = client
        .provider()
        .get_transaction_by_hash(hash)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get transaction: {}", e)))?;
    
    match tx {
        Some(tx) => {
            let json = serde_json::json!({
                "hash": format!("0x{:x}", tx.hash),
                "from": format!("0x{:x}", tx.from),
                "to": tx.to.map(|a| format!("0x{:x}", a)),
                "value": tx.value.to_string(),
                "gas": tx.gas.to_string(),
                "gasPrice": tx.gas_price.map(|p| p.to_string()),
                "maxFeePerGas": tx.max_fee_per_gas.map(|f| f.to_string()),
                "maxPriorityFeePerGas": tx.max_priority_fee_per_gas.map(|f| f.to_string()),
                "nonce": tx.nonce,
                "blockHash": tx.block_hash.map(|h| format!("0x{:x}", h)),
                "blockNumber": tx.block_number,
                "transactionIndex": tx.transaction_index,
                "input": format!("0x{}", hex::encode(&tx.input)),
                "chainId": tx.chain_id,
            });
            
            info!("Transaction retrieved: {}", tx_hash);
            Ok(json)
        }
        None => Err(EvmToolError::Generic(format!(
            "Transaction not found: {}",
            tx_hash
        ))),
    }
}

/// Get the current gas price
pub async fn get_gas_price(client: &EvmClient) -> Result<serde_json::Value> {
    debug!("Getting current gas price");
    
    let gas_price = client
        .provider()
        .get_gas_price()
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get gas price: {}", e)))?;
    
    // Also get base fee if available (EIP-1559)
    let latest_block = client
        .provider()
        .get_block_by_number(BlockNumberOrTag::Latest, false)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get latest block: {}", e)))?;
    
    let base_fee = latest_block
        .and_then(|b| b.header.base_fee_per_gas)
        .map(|f| f.to_string());
    
    let json = serde_json::json!({
        "gasPrice": gas_price.to_string(),
        "gasPriceGwei": format!("{:.2}", gas_price as f64 / 1e9),
        "baseFeePerGas": base_fee,
        "chain": client.chain_name(client.chain_id),
    });
    
    info!("Gas price: {} gwei", gas_price as f64 / 1e9);
    Ok(json)
}

/// Get account nonce
pub async fn get_nonce(client: &EvmClient, address: &str) -> Result<u64> {
    debug!("Getting nonce for address: {}", address);
    
    let addr = Address::from_str(address)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address: {}", e)))?;
    
    let nonce = client
        .provider()
        .get_transaction_count(addr)
        .await
        .map_err(|e| EvmToolError::Rpc(format!("Failed to get nonce: {}", e)))?;
    
    info!("Nonce for {}: {}", address, nonce);
    Ok(nonce)
}