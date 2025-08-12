use borsh::BorshDeserialize;
use chrono::{DateTime, Utc};
use riglr_events_core::{Event, EventKind, EventMetadata};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;
use std::collections::HashMap;
// UnifiedEvent trait removed - events now implement Event trait directly

/// Raydium CPMM Swap event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
pub struct RaydiumCpmmSwapEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Public key of the account that initiated the swap
    pub payer: Pubkey,
    /// Public key of the input token account
    pub input_token_account: Pubkey,
    /// Public key of the output token account
    pub output_token_account: Pubkey,
    /// Public key of the input token vault
    pub input_vault: Pubkey,
    /// Public key of the output token vault
    pub output_vault: Pubkey,
    /// Public key of the input token mint
    pub input_token_mint: Pubkey,
    /// Public key of the output token mint
    pub output_token_mint: Pubkey,
    /// Amount of input tokens swapped
    pub amount_in: u64,
    /// Amount of output tokens received
    pub amount_out: u64,
    /// Trading fee amount
    pub trade_fee: u64,
    /// Transfer fee amount
    pub transfer_fee: u64,
}

// Event trait implementation for RaydiumCpmmSwapEvent
impl Event for RaydiumCpmmSwapEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &EventKind::Swap
    }

    fn metadata(&self) -> &EventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
            id: String::default(),
            kind: EventKind::Swap,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::default(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        // This would need proper implementation with actual mutable metadata storage
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

/// Raydium CPMM Deposit event
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
pub struct RaydiumCpmmDepositEvent {
    /// Event metadata (excluded from serialization)
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    /// Public key of the CPMM pool state account
    pub pool_state: Pubkey,
    /// Public key of the user depositing liquidity
    pub user: Pubkey,
    /// Amount of LP tokens minted
    pub lp_token_amount: u64,
    /// Amount of token 0 deposited
    pub token_0_amount: u64,
    /// Amount of token 1 deposited
    pub token_1_amount: u64,
}

// Event trait implementation for RaydiumCpmmDepositEvent
impl Event for RaydiumCpmmDepositEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &EventKind::Liquidity
    }

    fn metadata(&self) -> &EventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<EventMetadata> = Lazy::new(|| EventMetadata {
            id: String::default(),
            kind: EventKind::Liquidity,
            timestamp: DateTime::<Utc>::MIN_UTC,
            received_at: DateTime::<Utc>::MIN_UTC,
            source: String::from("solana"),
            chain_data: None,
            custom: HashMap::default(),
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        // This would need proper implementation with actual mutable metadata storage
        unimplemented!("Mutable metadata not yet implemented")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
}

/// Event discriminator constants
pub mod discriminators {
    /// String identifier for swap events
    pub const SWAP_EVENT: &str = "raydium_cpmm_swap_event";
    /// String identifier for deposit events
    pub const DEPOSIT_EVENT: &str = "raydium_cpmm_deposit_event";

    /// Byte array discriminator for swap events
    pub const SWAP_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb1,
    ];
    /// Byte array discriminator for deposit events
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[
        0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0b, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72,
        0xb2,
    ];

    /// Instruction discriminator for swap with base input
    pub const SWAP_BASE_INPUT_IX: &[u8] = &[143, 190, 90, 218, 196, 30, 51, 222];
    /// Instruction discriminator for swap with base output
    pub const SWAP_BASE_OUTPUT_IX: &[u8] = &[55, 217, 98, 86, 163, 74, 180, 175];
    /// Instruction discriminator for deposit operations
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    /// Instruction discriminator for pool initialization
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    /// Instruction discriminator for withdraw operations
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}
