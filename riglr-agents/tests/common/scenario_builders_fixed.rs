//! Simple scenario builder implementation for Phase 4 E2E tests.
//!
//! This is a simplified version to avoid compilation issues with the complex
//! scenario builder. It provides basic functionality for E2E tests.

use crate::common::BlockchainTestHarness;

/// Simple scenario builder for E2E testing.
#[derive(Debug)]
pub struct SimpleScenarioBuilder {
    /// The name of the scenario being built.
    pub name: String,
}

impl SimpleScenarioBuilder {
    /// Creates a new scenario builder with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Create a simple SOL transfer scenario for testing.
    pub async fn execute_sol_transfer_test(
        harness: &BlockchainTestHarness,
        from_index: usize,
        to_index: usize,
        amount_sol: f64,
    ) -> std::result::Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let to_keypair = harness
            .get_funded_keypair(to_index)
            .ok_or("Failed to get recipient keypair")?;

        use solana_sdk::signer::Signer;
        let to_pubkey = to_keypair.pubkey();

        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

        let tx_info = harness
            .transfer_sol(from_index, &to_pubkey, amount_lamports)
            .await?;

        Ok(tx_info.signature.to_string())
    }
}
