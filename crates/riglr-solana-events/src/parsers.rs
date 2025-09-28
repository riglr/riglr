pub mod jupiter;
pub mod metaplex;
pub mod pump_fun;
pub mod raydium_v4;

// Explicit re-exports instead of glob imports
pub use jupiter::{
    Discriminator as JupiterDiscriminator,
    ExactOutRouteInstruction as JupiterExactOutRouteInstruction, Parser as JupiterParser,
    ParserFactory as JupiterParserFactory, RouteAnalysis as JupiterRouteAnalysis,
    RouteData as JupiterRouteData, RouteHop, RouteInstruction as JupiterRouteInstruction,
    JUPITER_PROGRAM_ID,
};

pub use metaplex::{
    AuctionHouseDiscriminator as MetaplexAuctionHouseDiscriminator, BurnNftInstruction,
    CreateMetadataAccountInstruction, EventAnalysis as MetaplexEventAnalysis,
    Parser as MetaplexParser, ParserFactory as MetaplexParserFactory,
    TokenMetadataDiscriminator as MetaplexTokenMetadataDiscriminator, TransferInstruction,
    METAPLEX_AUCTION_HOUSE_PROGRAM_ID, METAPLEX_TOKEN_METADATA_PROGRAM_ID,
};

pub use pump_fun::{
    Discriminator as PumpFunDiscriminator, ParserFactory as PumpFunParserFactory,
    PumpBuyInstruction, PumpCreatePoolInstruction, PumpDepositInstruction, PumpFunParser,
    PumpSellInstruction, PumpWithdrawInstruction, PUMP_FUN_PROGRAM_ID,
};

pub use raydium_v4::{
    DepositInstruction, Discriminator as RaydiumV4Discriminator, Parser as RaydiumV4Parser,
    ParserFactory as RaydiumV4ParserFactory, SwapBaseInInstruction, SwapBaseOutInstruction,
    WithdrawInstruction, RAYDIUM_AMM_V4_PROGRAM_ID,
};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn jupiter_reexports_compile() {
        // Test that jupiter module re-exports are accessible
        // Test that jupiter types can be referenced (compile-time check)
        let _: Option<JupiterDiscriminator> = None;
        let _: Option<JupiterParser> = None;
        let _: Option<JupiterParserFactory> = None;
        let _: &str = JUPITER_PROGRAM_ID;
    }

    #[test]
    fn pump_fun_reexports_compile() {
        // Test that pump_fun module re-exports are accessible
        // Test that pump_fun types can be referenced (compile-time check)
        let _: Option<PumpFunDiscriminator> = None;
        let _: Option<PumpFunParser> = None;
        let _: Option<PumpFunParserFactory> = None;
        let _: &str = PUMP_FUN_PROGRAM_ID;
    }

    #[test]
    fn metaplex_reexports_compile() {
        // Test that metaplex module re-exports are accessible
        // Test that metaplex types can be referenced (compile-time check)
        let _: Option<MetaplexTokenMetadataDiscriminator> = None;
        let _: Option<MetaplexAuctionHouseDiscriminator> = None;
        let _: &str = METAPLEX_TOKEN_METADATA_PROGRAM_ID;
        let _: &str = METAPLEX_AUCTION_HOUSE_PROGRAM_ID;
    }

    #[test]
    fn raydium_v4_reexports_compile() {
        // Test that raydium_v4 module re-exports are accessible
        // Test that raydium_v4 types can be referenced (compile-time check)
        let _: Option<RaydiumV4Discriminator> = None;
        let _: &str = RAYDIUM_AMM_V4_PROGRAM_ID;
    }
}
