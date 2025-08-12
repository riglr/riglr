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
            .map_err(|e| ToolError::permanent(format!("Failed to serialize transaction: {}", e)))?;
        
        const MAX_TRANSACTION_SIZE: usize = 1232; // Solana's max transaction size
        
        if serialized.len() > MAX_TRANSACTION_SIZE {
            return Err(ToolError::permanent(
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
        // This would typically call RPC method to get recent fees
        // For now, return the configured default
        Ok(self.priority_config.microlamports_per_cu)
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
                        return Err(ToolError::permanent(
                            format!("Insufficient funds: {}", e)
                        ));
                    } else {
                        // Other errors might be retriable
                        tokio::time::sleep(config.initial_delay).await;
                    }
                }
                Err(e) => {
                    return Err(ToolError::permanent(
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
            .map_err(|e| ToolError::permanent(format!("Invalid signature: {}", e)))?;
        
        // Get transaction status
        match self.client.get_signature_status(&signature) {
            Ok(Some(status)) => {
                if status.is_err() {
                    Ok(TransactionStatus::Failed {
                        reason: format!("Transaction failed: {:?}", status.err()),
                    })
                } else {
                    // Transaction is successful
                    Ok(TransactionStatus::Confirmed {
                        hash: tx_hash.to_string(),
                        block: 0, // Solana doesn't easily provide block number here
                    })
                }
            }
            Ok(None) => {
                // Transaction not found or not yet processed
                Ok(TransactionStatus::Submitted {
                    hash: tx_hash.to_string(),
                })
            }
            Err(e) => {
                Err(ToolError::permanent(
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
            .map_err(|e| ToolError::permanent(format!("Invalid signature: {}", e)))?;
        
        // Use confirmed commitment for waiting
        let commitment = if required_confirmations > 1 {
            CommitmentConfig::finalized()
        } else {
            CommitmentConfig::confirmed()
        };
        
        match self.client.confirm_transaction_with_commitment(&signature, commitment) {
            Ok(_) => {
                info!("Transaction {} confirmed with {:?}", tx_hash, commitment);
                Ok(TransactionStatus::Confirmed {
                    hash: tx_hash.to_string(),
                    block: 0,
                })
            }
            Err(e) => {
                Err(ToolError::permanent(
                    format!("Failed to confirm transaction: {}", e)
                ))
            }
        }
    }
}