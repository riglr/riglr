use riglr_solana_tools::utils::{create_token_with_mint_keypair, generate_mint_keypair};
use riglr_core::{
    signer::{SignerContext, TransactionSigner, SignerError},
};
use std::sync::Arc;
#[allow(deprecated)]
use solana_sdk::{
    signature::Keypair,
    signer::Signer as SolanaSigner,
    pubkey::Pubkey,
    instruction::Instruction,
    system_instruction,
    transaction::Transaction,
};
use std::str::FromStr;

/// Mock Solana signer for testing
struct MockSolanaSigner {
    keypair: Keypair,
    rpc_url: String,
}

impl MockSolanaSigner {
    fn new(keypair: Keypair) -> Self {
        Self {
            keypair,
            rpc_url: "https://api.devnet.solana.com".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl TransactionSigner for MockSolanaSigner {
    fn pubkey(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }

    fn address(&self) -> Option<String> {
        None
    }

    async fn sign_and_send_solana_transaction(&self, _tx: &mut Transaction) -> Result<String, SignerError> {
        // Mock transaction submission - return a mock signature
        Ok("5VfYmGC52L5CuxpGtDVsgQs3T1NHDzaAhF28UwKnztcyEJhwjdEu6rkJG5qNt1WyKmkqFq8v2wQeJnj5zV1sXNkL".to_string())
    }

    async fn sign_and_send_evm_transaction(&self, _tx: alloy::rpc::types::TransactionRequest) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("Solana signer cannot sign EVM transactions".to_string()))
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(solana_client::rpc_client::RpcClient::new(self.rpc_url.clone())))
    }

    fn evm_client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Solana signer does not provide EVM client".to_string()))
    }
}

impl std::fmt::Debug for MockSolanaSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockSolanaSigner")
            .field("pubkey", &self.keypair.pubkey().to_string())
            .field("rpc_url", &self.rpc_url)
            .finish()
    }
}

#[tokio::test]
async fn test_mint_keypair_generation() {
    // Test that we can generate mint keypairs
    let mint_keypair1 = generate_mint_keypair();
    let mint_keypair2 = generate_mint_keypair();
    
    // Verify they are different
    assert_ne!(mint_keypair1.pubkey(), mint_keypair2.pubkey());
    
    // Verify they are valid keypairs
    assert_eq!(mint_keypair1.pubkey().to_string().len(), 44); // Base58 encoded pubkey length
    assert_eq!(mint_keypair2.pubkey().to_string().len(), 44);
    
    // Verify they can sign
    let message = b"test message";
    let signature1 = mint_keypair1.sign_message(message);
    let signature2 = mint_keypair2.sign_message(message);
    
    assert_ne!(signature1.to_string(), signature2.to_string());
}

#[tokio::test]
async fn test_deterministic_addressing() {
    // Test that mint keypairs produce deterministic addresses
    let mint_keypair = generate_mint_keypair();
    let mint_address = mint_keypair.pubkey();
    
    // The same keypair should always produce the same address
    assert_eq!(mint_keypair.pubkey(), mint_address);
    
    // Test derivation of program-derived addresses (PDAs)
    let program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let seeds = &[b"metadata", program_id.as_ref(), mint_address.as_ref()];
    
    let (pda1, bump1) = Pubkey::find_program_address(seeds, &program_id);
    let (pda2, bump2) = Pubkey::find_program_address(seeds, &program_id);
    
    // Same seeds should produce same PDA
    assert_eq!(pda1, pda2);
    assert_eq!(bump1, bump2);
    
    // Different mint should produce different PDA
    let different_mint = Keypair::new().pubkey();
    let different_seeds = &[b"metadata", program_id.as_ref(), different_mint.as_ref()];
    let (different_pda, _) = Pubkey::find_program_address(different_seeds, &program_id);
    
    assert_ne!(pda1, different_pda);
}

#[tokio::test]
async fn test_create_token_with_mint_keypair() {
    let payer_keypair = Keypair::new();
    let payer_pubkey = payer_keypair.pubkey();
    let signer = Arc::new(MockSolanaSigner::new(payer_keypair));
    
    let result = SignerContext::with_signer(signer, async {
        let mint_keypair = generate_mint_keypair();
        
        // Create mock instructions for token creation
        let instructions = create_mock_token_instructions(&payer_pubkey, &mint_keypair.pubkey());
        
        create_token_with_mint_keypair(instructions, &mint_keypair).await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    }).await;
    
    assert!(result.is_ok());
    let signature = result.unwrap();
    
    // Verify signature format (Solana signatures are base58 encoded)
    assert!(signature.len() >= 86 && signature.len() <= 88); // Typical base58 signature length
    assert!(signature.chars().all(|c| c.is_alphanumeric()));
}

#[tokio::test]
async fn test_utility_functions() {
    // Test utility functions that would be used in token operations
    
    // Test pubkey validation
    let valid_pubkey = "11111111111111111111111111111112";
    let parsed_pubkey = Pubkey::from_str(valid_pubkey);
    assert!(parsed_pubkey.is_ok());
    
    // Test invalid pubkey
    let invalid_pubkey = "invalid_pubkey";
    let parsed_invalid = Pubkey::from_str(invalid_pubkey);
    assert!(parsed_invalid.is_err());
    
    // Test rent calculation simulation
    let account_size = 165; // Size of a mint account
    let rent_minimum = calculate_mock_rent_exemption(account_size);
    assert!(rent_minimum > 0);
    
    // Test lamports conversion
    let sol_amount = 0.001;
    let lamports = sol_to_lamports(sol_amount);
    assert_eq!(lamports, 1_000_000);
}

#[tokio::test]
async fn test_solana_error_handling() {
    let payer_keypair = Keypair::new();
    let signer = Arc::new(MockSolanaSigner::new(payer_keypair));
    
    // Test with empty instructions
    let result = SignerContext::with_signer(signer.clone(), async {
        let mint_keypair = generate_mint_keypair();
        let empty_instructions = vec![];
        
        create_token_with_mint_keypair(empty_instructions, &mint_keypair).await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    }).await;
    
    // This should still work (empty transaction)
    assert!(result.is_ok());
    
    // Test with invalid instruction
    let result = SignerContext::with_signer(signer, async {
        let mint_keypair = generate_mint_keypair();
        let invalid_instructions = create_invalid_instructions();
        
        create_token_with_mint_keypair(invalid_instructions, &mint_keypair).await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    }).await;
    
    // Should still return OK since we're mocking the transaction submission
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_mint_keypair_generation() {
    let handles = (0..10).map(|_| {
        tokio::spawn(async {
            generate_mint_keypair()
        })
    }).collect::<Vec<_>>();
    
    let keypairs = futures_util::future::join_all(handles).await;
    let mut pubkeys = std::collections::HashSet::new();
    
    for keypair_result in keypairs {
        let keypair = keypair_result.unwrap();
        let pubkey = keypair.pubkey();
        
        // Ensure all generated keypairs are unique
        assert!(pubkeys.insert(pubkey), "Duplicate keypair generated");
    }
    
    assert_eq!(pubkeys.len(), 10);
}

#[tokio::test]
async fn test_transaction_serialization() {
    let payer_keypair = Keypair::new();
    let mint_keypair = generate_mint_keypair();
    
    // Create a transaction
    let instructions = create_mock_token_instructions(&payer_keypair.pubkey(), &mint_keypair.pubkey());
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer_keypair.pubkey()));
    
    // Set a mock recent blockhash
    let recent_blockhash = solana_sdk::hash::Hash::new_unique();
    transaction.message.recent_blockhash = recent_blockhash;
    
    // Sign the transaction
    transaction.partial_sign(&[&payer_keypair, &mint_keypair], recent_blockhash);
    
    // Verify transaction structure
    assert!(!transaction.signatures.is_empty());
    assert_eq!(transaction.signatures.len(), 2); // Payer + mint keypair
    assert_eq!(transaction.message.account_keys.len(), instructions.iter().map(|ix| ix.accounts.len()).sum::<usize>() + 1); // +1 for payer
}

#[tokio::test]
async fn test_signer_context_with_solana_operations() {
    let keypair1 = Keypair::new();
    let keypair2 = Keypair::new();
    
    let keypair1_pubkey = keypair1.pubkey();
    let keypair2_pubkey = keypair2.pubkey();
    let signer1 = MockSolanaSigner::new(keypair1);
    let signer2 = MockSolanaSigner::new(keypair2);
    
    // Test concurrent operations with different signers
    let handle1 = tokio::spawn(async move {
        SignerContext::with_signer(Arc::new(signer1), async {
            let current = SignerContext::current().await?;
            let pubkey = current.pubkey().unwrap();
            assert_eq!(pubkey, keypair1_pubkey.to_string());
            
            // Perform token creation
            let mint_keypair = generate_mint_keypair();
            let instructions = create_mock_token_instructions(&keypair1_pubkey, &mint_keypair.pubkey());
            create_token_with_mint_keypair(instructions, &mint_keypair).await
                .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
        }).await
    });
    
    let handle2 = tokio::spawn(async move {
        SignerContext::with_signer(Arc::new(signer2), async {
            let current = SignerContext::current().await?;
            let pubkey = current.pubkey().unwrap();
            assert_eq!(pubkey, keypair2_pubkey.to_string());
            
            // Perform different operation
            let mint_keypair = generate_mint_keypair();
            let instructions = create_mock_token_instructions(&keypair2_pubkey, &mint_keypair.pubkey());
            create_token_with_mint_keypair(instructions, &mint_keypair).await
                .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
        }).await
    });
    
    let results = futures_util::future::join_all(vec![handle1, handle2]).await;
    
    for result in results {
        let tx_result = result.unwrap();
        assert!(tx_result.is_ok());
    }
}

// Helper functions for testing

fn create_mock_token_instructions(payer: &Pubkey, mint: &Pubkey) -> Vec<Instruction> {
    vec![
        // Create mint account
        system_instruction::create_account(
            payer,
            mint,
            1_000_000, // Mock rent exemption amount
            82,        // Mint account size
            &spl_token::id(),
        ),
        // Initialize mint
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint,
            payer,
            Some(payer),
            9, // decimals
        ).unwrap(),
    ]
}

fn create_invalid_instructions() -> Vec<Instruction> {
    vec![
        Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![],
            data: vec![255, 255, 255], // Invalid instruction data
        }
    ]
}

fn calculate_mock_rent_exemption(data_size: usize) -> u64 {
    // Mock rent calculation - in real implementation this would query the RPC
    let base_rent = 1_000_000; // ~0.001 SOL
    let size_rent = data_size as u64 * 6_960; // Mock rate per byte
    base_rent + size_rent
}

fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}