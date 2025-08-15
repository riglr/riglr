use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use riglr_events_core::{Event, EventKind, EventMetadata as CoreEventMetadata};
use std::any::Any;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::types::EventMetadata;
// UnifiedEvent trait removed - events now implement Event trait directly

/// Raydium CPMM Swap event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmSwapEvent {
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub payer: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub trade_fee: u64,
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

    fn metadata(&self) -> &CoreEventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<CoreEventMetadata> = Lazy::new(|| {
            CoreEventMetadata {
                id: String::new(),
                kind: EventKind::Swap,
                timestamp: DateTime::<Utc>::MIN_UTC,
                received_at: DateTime::<Utc>::MIN_UTC,
                source: String::from("solana"),
                chain_data: None,
                custom: HashMap::new(),
            }
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct RaydiumCpmmDepositEvent {
    #[serde(skip)]
    #[borsh(skip)]
    pub metadata: EventMetadata,
    pub pool_state: Pubkey,
    pub user: Pubkey,
    pub lp_token_amount: u64,
    pub token_0_amount: u64,
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

    fn metadata(&self) -> &CoreEventMetadata {
        use once_cell::sync::Lazy;
        static DEFAULT_METADATA: Lazy<CoreEventMetadata> = Lazy::new(|| {
            CoreEventMetadata {
                id: String::new(),
                kind: EventKind::Liquidity,
                timestamp: DateTime::<Utc>::MIN_UTC,
                received_at: DateTime::<Utc>::MIN_UTC,
                source: String::from("solana"),
                chain_data: None,
                custom: HashMap::new(),
            }
        });
        &DEFAULT_METADATA
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
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
    // Event discriminators - more efficient string identifiers
    pub const SWAP_EVENT: &str = "raydium_cpmm_swap_event";
    pub const DEPOSIT_EVENT: &str = "raydium_cpmm_deposit_event";
    
    // Raw event discriminators as byte arrays for efficient parsing
    pub const SWAP_EVENT_BYTES: &[u8] = &[0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0a, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72, 0xb1];
    pub const DEPOSIT_EVENT_BYTES: &[u8] = &[0xe4, 0x45, 0xa5, 0x2e, 0x51, 0xcb, 0x9a, 0x1d, 0x0b, 0x11, 0xa9, 0xd2, 0xbe, 0x8b, 0x72, 0xb2];
    
    // Instruction discriminators
    pub const SWAP_BASE_INPUT_IX: &[u8] = &[143, 190, 90, 218, 196, 30, 51, 222];
    pub const SWAP_BASE_OUTPUT_IX: &[u8] = &[55, 217, 98, 86, 163, 74, 180, 175];
    pub const DEPOSIT_IX: &[u8] = &[242, 35, 198, 137, 82, 225, 242, 182];
    pub const INITIALIZE_IX: &[u8] = &[175, 175, 109, 31, 13, 152, 155, 237];
    pub const WITHDRAW_IX: &[u8] = &[183, 18, 70, 156, 148, 109, 161, 34];
}