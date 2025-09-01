/// Raydium CLMM instruction discriminators and constants.
pub mod discriminators;
/// Raydium CLMM event definitions and structures.
pub mod events;
/// Raydium CLMM transaction and log parsing functionality.
pub mod parser;

pub use events::*;
pub use parser::{RaydiumClmmEventParser, RAYDIUM_CLMM_PROGRAM_ID};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::parser_types::ProtocolParser;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_re_exports_available() {
        // Test that the re-exported items are accessible
        // This ensures the pub use statements work correctly
        use crate::events::protocols::raydium_clmm::{
            RaydiumClmmEventParser, RAYDIUM_CLMM_PROGRAM_ID,
        };

        // Verify the program ID constant is accessible and has the expected type
        let _program_id: Pubkey = RAYDIUM_CLMM_PROGRAM_ID;

        // Verify the parser type is accessible
        let _parser_type = std::marker::PhantomData::<RaydiumClmmEventParser>;
    }

    #[test]
    fn test_module_exports_parser() {
        // Test that RaydiumClmmEventParser is properly exported
        let parser = RaydiumClmmEventParser::new();

        // Verify parser can be created and has the expected program ID support
        let supported_ids = parser.supported_program_ids();
        assert!(!supported_ids.is_empty());
        assert!(supported_ids.contains(&RAYDIUM_CLMM_PROGRAM_ID));
    }

    #[test]
    fn test_module_exports_program_id_constant() {
        // Test that RAYDIUM_CLMM_PROGRAM_ID is properly exported
        // Verify it's not the default pubkey
        assert_ne!(RAYDIUM_CLMM_PROGRAM_ID, Pubkey::default());
    }

    #[test]
    fn test_parser_should_handle_correct_program_id() {
        let parser = RaydiumClmmEventParser::new();

        // Should handle the Raydium CLMM program ID
        assert!(parser.should_handle(&RAYDIUM_CLMM_PROGRAM_ID));

        // Should not handle other program IDs
        assert!(!parser.should_handle(&Pubkey::default()));
        assert!(!parser.should_handle(&solana_sdk::pubkey!("11111111111111111111111111111112")));
    }

    #[test]
    fn test_module_structure_integrity() {
        // This test ensures that the discriminators module is accessible
        // by testing some constant values it should contain
        use discriminators::*;

        // Test that we can access discriminators - verify they're defined correctly
        assert_eq!(SWAP.len(), 8);
        assert_eq!(SWAP_V2.len(), 8);
        assert_eq!(CLOSE_POSITION.len(), 8);
        assert_eq!(DECREASE_LIQUIDITY_V2.len(), 8);
        assert_eq!(CREATE_POOL.len(), 8);
        assert_eq!(INCREASE_LIQUIDITY_V2.len(), 8);
        assert_eq!(OPEN_POSITION_WITH_TOKEN_22_NFT.len(), 8);
        assert_eq!(OPEN_POSITION_V2.len(), 8);

        // Verify discriminators are unique
        let discriminators = vec![
            SWAP,
            SWAP_V2,
            CLOSE_POSITION,
            DECREASE_LIQUIDITY_V2,
            CREATE_POOL,
            INCREASE_LIQUIDITY_V2,
            OPEN_POSITION_WITH_TOKEN_22_NFT,
            OPEN_POSITION_V2,
        ];

        for (i, &disc1) in discriminators.iter().enumerate() {
            for (j, &disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2);
                }
            }
        }
    }
}
