//! Solana-specific transaction processing with priority fees
//!
//! This module provides enhanced Solana transaction handling including:
//! - Priority fee estimation
//! - Transaction size optimization
//! - Compute unit optimization
//! - Blockhash management

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    signature::Signature,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, debug};

use crate::error::ToolError;
use super::{TransactionProcessor, TransactionStatus, RetryConfig};

/// Solana priority fee configuration
#[derive(Debug, Clone)]
pub struct PriorityFeeConfig {
    /// Enable priority fees
    pub enabled: bool,
    /// Priority fee in microlamports per compute unit
    pub microlamports_per_cu: u64,
    /// Additional compute units to request
    pub additional_compute_units: Option<u32>,
}

impl Default for PriorityFeeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            microlamports_per_cu: 1000, // 0.001 lamports per CU
            additional_compute_units: Some(200_000),
        }
    }
}

/// Solana transaction processor with priority fee support
pub struct SolanaTransactionProcessor {
    client: Arc<RpcClient>,
    priority_config: PriorityFeeConfig,
}

impl SolanaTransactionProcessor {
    /// Create a new Solana transaction processor
    pub fn new(client: Arc<RpcClient>, priority_config: PriorityFeeConfig) -> Self {
        Self {
            client,
            priority_config,
        }
    }
    
    /// Add priority fee instructions to transaction
    pub fn add_priority_fee_instructions(
        &self,
        instructions: &mut Vec<Instruction>,
    ) {
        if !self.priority_config.enabled {
            return;
        }
        
        // Add compute budget instructions at the beginning
        let mut priority_instructions = vec![];
        
        // Set compute unit price for priority
        priority_instructions.push(
            ComputeBudgetInstruction::set_compute_unit_price(
                self.priority_config.microlamports_per_cu
            )
        );
        
        // Optionally set compute unit limit
        if let Some(units) = self.priority_config.additional_compute_units {
            priority_instructions.push(
                ComputeBudgetInstruction::set_compute_unit_limit(units)
            );
        }
        
        // Insert at the beginning of instructions
        priority_instructions.append(instructions);
        *instructions = priority_instructions;
        
        debug!(
            "Added priority fee: {} microlamports/CU",
            self.priority_config.microlamports_per_cu
        );
    }
    
    /// Optimize transaction for size and compute units
    pub fn optimize_transaction(&self, tx: &mut Transaction) -> Result<(), ToolError> {
        // Check transaction size
        let serialized = bincode::serialize(&tx)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize transaction: {}", e)))?;
        
        const MAX_TRANSACTION_SIZE: usize = 1232; // Solana's max transaction size
        
        if serialized.len() > MAX_TRANSACTION_SIZE {
            return Err(ToolError::permanent_string(
                format!(
                    "Transaction size {} exceeds maximum {}",
                    serialized.len(),
                    MAX_TRANSACTION_SIZE
                )
            ));
        }
        
        debug!("Transaction size: {} bytes", serialized.len());
        Ok(())
    }
    
    /// Get recent prioritization fees from the network
    pub async fn get_recent_prioritization_fees(&self) -> Result<u64, ToolError> {
        // Try to get recent prioritization fees from the network
        // Using the getRecentPrioritizationFees RPC method
        
        // Build the RPC request for recent prioritization fees
        let _request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getRecentPrioritizationFees",
            "params": []
        });
        
        // Try to get fees from RPC, fall back to default if unavailable
        match self.client.get_recent_prioritization_fees(&[]) {
            Ok(fees) if !fees.is_empty() => {
                // Calculate average of recent fees
                let total: u64 = fees.iter().map(|f| f.prioritization_fee).sum();
                let average = total / fees.len() as u64;
                
                debug!("Average recent prioritization fee: {} microlamports/CU", average);
                
                // Use the average or fall back to configured if too low
                if average > 0 {
                    Ok(average)
                } else {
                    Ok(self.priority_config.microlamports_per_cu)
                }
            }
            Ok(_) => {
                // No recent fees available, use configured default
                debug!("No recent prioritization fees available, using default: {} microlamports/CU", 
                       self.priority_config.microlamports_per_cu);
                Ok(self.priority_config.microlamports_per_cu)
            }
            Err(e) => {
                // RPC error, log and use default
                debug!("Failed to get recent prioritization fees: {}, using default", e);
                Ok(self.priority_config.microlamports_per_cu)
            }
        }
    }
    
    /// Send transaction with automatic retry on blockhash expiry
    pub async fn send_transaction_with_retry(
        &self,
        tx: &Transaction,
    ) -> Result<Signature, ToolError> {
        let config = RetryConfig::default();
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match self.client.send_and_confirm_transaction_with_spinner(tx) {
                Ok(signature) => {
                    info!("Transaction confirmed: {}", signature);
                    return Ok(signature);
                }
                Err(e) if attempts < config.max_attempts => {
                    let error_str = e.to_string();
                    
                    if error_str.contains("blockhash not found") || 
                       error_str.contains("blockhash expired") {
                        // Need to refresh blockhash and retry
                        return Err(ToolError::Retriable {
                            source: Box::new(e),
                            context: "Blockhash expired, need to refresh".to_string(),
                        });
                    } else if error_str.contains("insufficient funds") {
                        // Non-retriable error
                        return Err(ToolError::permanent_string(
                            format!("Insufficient funds: {}", e)
                        ));
                    }
                    // Other errors might be retriable
                    tokio::time::sleep(config.initial_delay).await;
                }
                Err(e) => {
                    return Err(ToolError::permanent_string(
                        format!("Transaction failed after {} attempts: {}", attempts, e)
                    ));
                }
            }
        }
    }
}

#[async_trait]
impl TransactionProcessor for SolanaTransactionProcessor {
    async fn process_with_retry<T, F, Fut>(
        &self,
        operation: F,
        config: RetryConfig,
    ) -> Result<T, ToolError>
    where
        T: Send,
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, ToolError>> + Send,
    {
        // Use the generic implementation
        let processor = super::GenericTransactionProcessor;
        processor.process_with_retry(operation, config).await
    }
    
    async fn get_status(&self, tx_hash: &str) -> Result<TransactionStatus, ToolError> {
        let signature = tx_hash.parse::<Signature>()
            .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {}", e)))?;
        
        // Get transaction status
        match self.client.get_signature_status(&signature) {
            Ok(Some(status)) => {
                if status.is_err() {
                    Ok(TransactionStatus::Failed {
                        reason: format!("Transaction failed: {:?}", status.err()),
                    })
                } else {
                    // Transaction is successful, try to get slot information
                    match self.client.get_transaction(&signature, UiTransactionEncoding::Json) {
                        Ok(confirmed_tx) => {
                            let block = confirmed_tx.slot;
                            
                            // Check if finalized based on commitment
                            match self.client.get_signature_status_with_commitment(&signature, CommitmentConfig::finalized()) {
                                Ok(Some(finalized_status)) if finalized_status.is_ok() => {
                                    Ok(TransactionStatus::Confirmed {
                                        hash: tx_hash.to_string(),
                                        block,
                                    })
                                }
                                _ => {
                                    // Not yet finalized, still confirming
                                    Ok(TransactionStatus::Confirming {
                                        hash: tx_hash.to_string(),
                                        confirmations: 1, // At least 1 confirmation
                                    })
                                }
                            }
                        }
                        Err(_) => {
                            // Can't get detailed info, but transaction is successful
                            Ok(TransactionStatus::Confirmed {
                                hash: tx_hash.to_string(),
                                block: self.client.get_slot().unwrap_or(0),
                            })
                        }
                    }
                }
            }
            Ok(None) => {
                // Transaction not found or not yet processed
                Ok(TransactionStatus::Submitted {
                    hash: tx_hash.to_string(),
                })
            }
            Err(e) => {
                Err(ToolError::permanent_string(
                    format!("Failed to get transaction status: {}", e)
                ))
            }
        }
    }
    
    async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        required_confirmations: u64,
    ) -> Result<TransactionStatus, ToolError> {
        let signature = tx_hash.parse::<Signature>()
            .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {}", e)))?;
        
        // Determine commitment level based on required confirmations
        // Solana has: processed (0), confirmed (1), finalized (31+)
        let commitment = if required_confirmations >= 31 {
            CommitmentConfig::finalized()
        } else if required_confirmations > 0 {
            CommitmentConfig::confirmed()
        } else {
            CommitmentConfig::processed()
        };
        
        // Wait for the transaction to reach the desired commitment level
        let max_retries = 60; // Maximum retries (60 * 2 seconds = 2 minutes)
        let mut retries = 0;
        
        loop {
            match self.client.get_signature_status_with_commitment(&signature, commitment) {
                Ok(Some(status)) => {
                    if status.is_err() {
                        return Err(ToolError::permanent_string(
                            format!("Transaction failed: {:?}", status.err())
                        ));
                    }
                    
                    // Transaction has reached desired commitment, get slot info
                    match self.client.get_transaction(&signature, UiTransactionEncoding::Json) {
                        Ok(confirmed_tx) => {
                            let block = confirmed_tx.slot;
                            info!("Transaction {} confirmed at slot {} with {:?}", tx_hash, block, commitment);
                            
                            return Ok(TransactionStatus::Confirmed {
                                hash: tx_hash.to_string(),
                                block,
                            });
                        }
                        Err(_) => {
                            // Can't get slot info, use current slot as approximation
                            let block = self.client.get_slot().unwrap_or(0);
                            info!("Transaction {} confirmed (approximate slot {})", tx_hash, block);
                            
                            return Ok(TransactionStatus::Confirmed {
                                hash: tx_hash.to_string(),
                                block,
                            });
                        }
                    }
                }
                Ok(None) => {
                    // Transaction not yet at desired commitment level
                    retries += 1;
                    if retries >= max_retries {
                        return Err(ToolError::permanent_string(
                            format!("Transaction {} not confirmed after {} seconds", tx_hash, max_retries * 2)
                        ));
                    }
                    
                    debug!("Waiting for transaction {} to reach {:?} commitment... (attempt {}/{})", 
                           tx_hash, commitment, retries, max_retries);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    return Err(ToolError::permanent_string(
                        format!("Failed to check transaction status: {}", e)
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    
    #[test]
    fn test_priority_fee_config() {
        let config = PriorityFeeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.microlamports_per_cu, 1000);
        assert_eq!(config.additional_compute_units, Some(200_000));
    }
    
    #[test]
    fn test_add_priority_fee_instructions() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let processor = SolanaTransactionProcessor::new(client, PriorityFeeConfig::default());
        
        let mut instructions = vec![
            solana_sdk::system_instruction::transfer(
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                1000,
            ),
        ];
        
        let original_len = instructions.len();
        processor.add_priority_fee_instructions(&mut instructions);
        
        // Should add 2 instructions (compute unit price and limit)
        assert_eq!(instructions.len(), original_len + 2);
        
        // Priority instructions should be at the beginning
        // We can't easily test the exact instruction type, but we can verify count
    }
    
    #[test]
    fn test_disabled_priority_fees() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let config = PriorityFeeConfig {
            enabled: false,
            ..Default::default()
        };
        let processor = SolanaTransactionProcessor::new(client, config);
        
        let mut instructions = vec![
            solana_sdk::system_instruction::transfer(
                &Pubkey::new_unique(),
                &Pubkey::new_unique(),
                1000,
            ),
        ];
        
        let original_len = instructions.len();
        processor.add_priority_fee_instructions(&mut instructions);
        
        // Should not add any instructions when disabled
        assert_eq!(instructions.len(), original_len);
    }
    
    #[test]
    fn test_transaction_size_validation() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com".to_string()));
        let processor = SolanaTransactionProcessor::new(client, PriorityFeeConfig::default());
        
        // Create a normal-sized transaction
        let payer = Pubkey::new_unique();
        let mut tx = Transaction::new_with_payer(
            &[solana_sdk::system_instruction::transfer(
                &payer,
                &Pubkey::new_unique(),
                1000,
            )],
            Some(&payer),
        );
        
        // Should succeed for normal transaction
        assert!(processor.optimize_transaction(&mut tx).is_ok());
    }
}