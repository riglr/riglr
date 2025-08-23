pub mod jupiter;
pub mod metaplex;
pub mod pump_fun;
pub mod raydium_v4;

pub use jupiter::*;
pub use metaplex::*;
pub use pump_fun::*;
pub use raydium_v4::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_jupiter_reexports_compile() {
        // Test that jupiter module re-exports are accessible
        use super::*;

        // Test that jupiter types can be referenced (compile-time check)
        let _discriminator: Option<JupiterDiscriminator> = None;
        let _parser: Option<JupiterParser> = None;
        let _factory: Option<JupiterParserFactory> = None;
        let _program_id = JUPITER_PROGRAM_ID;
    }

    #[test]
    fn test_pump_fun_reexports_compile() {
        // Test that pump_fun module re-exports are accessible
        use super::*;

        // Test that pump_fun types can be referenced (compile-time check)
        let _discriminator: Option<PumpFunDiscriminator> = None;
        let _parser: Option<PumpFunParser> = None;
        let _factory: Option<PumpFunParserFactory> = None;
        let _program_id = PUMP_FUN_PROGRAM_ID;
    }

    #[test]
    fn test_metaplex_reexports_compile() {
        // Test that metaplex module re-exports are accessible
        use super::*;

        // Test that metaplex types can be referenced (compile-time check)
        let _metadata_discriminator: Option<MetaplexTokenMetadataDiscriminator> = None;
        let _auction_discriminator: Option<MetaplexAuctionHouseDiscriminator> = None;
        let _metadata_program_id = METAPLEX_TOKEN_METADATA_PROGRAM_ID;
        let _auction_program_id = METAPLEX_AUCTION_HOUSE_PROGRAM_ID;
    }

    #[test]
    fn test_raydium_v4_reexports_compile() {
        // Test that raydium_v4 module re-exports are accessible
        use super::*;

        // Test that raydium_v4 types can be referenced (compile-time check)
        let _discriminator: Option<RaydiumV4Discriminator> = None;
        let _program_id = RAYDIUM_AMM_V4_PROGRAM_ID;
    }
}
