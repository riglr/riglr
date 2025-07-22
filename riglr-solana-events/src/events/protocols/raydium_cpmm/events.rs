use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::types::EventMetadata;
use crate::impl_unified_event;

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

// Custom implementation to handle merge logic properly
impl crate::events::core::traits::UnifiedEvent for RaydiumCpmmSwapEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn event_type(&self) -> crate::types::EventType {
        self.metadata.event_type.clone()
    }

    fn signature(&self) -> &str {
        &self.metadata.signature
    }

    fn slot(&self) -> u64 {
        self.metadata.slot
    }

    fn program_received_time_ms(&self) -> i64 {
        self.metadata.program_received_time_ms
    }
    
    fn program_handle_time_consuming_ms(&self) -> i64 {
        self.metadata.program_handle_time_consuming_ms
    }

    fn set_program_handle_time_consuming_ms(&mut self, time: i64) {
        self.metadata.program_handle_time_consuming_ms = time;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn crate::events::core::traits::UnifiedEvent> {
        Box::new(self.clone())
    }

    /// Custom merge implementation that only fills missing values
    fn merge(&mut self, other: Box<dyn crate::events::core::traits::UnifiedEvent>) {
        if let Some(other_event) = other.as_any().downcast_ref::<RaydiumCpmmSwapEvent>() {
            // Only update amount_out if it's currently 0 and the other has a value
            if self.amount_out == 0 && other_event.amount_out > 0 {
                self.amount_out = other_event.amount_out;
            }
            // Only update amount_in if it's currently 0 and the other has a value
            if self.amount_in == 0 && other_event.amount_in > 0 {
                self.amount_in = other_event.amount_in;
            }
            // Always update fees from log data if available
            if other_event.trade_fee > 0 {
                self.trade_fee = other_event.trade_fee;
            }
            if other_event.transfer_fee > 0 {
                self.transfer_fee = other_event.transfer_fee;
            }
        }
    }

    fn set_transfer_data(&mut self, _transfer_data: Vec<crate::types::TransferData>, _swap_data: Option<crate::types::SwapData>) {
        // Implementation can be extended as needed
    }

    fn index(&self) -> String {
        self.metadata.index.clone()
    }

    fn protocol_type(&self) -> crate::types::ProtocolType {
        self.metadata.protocol_type.clone()
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

impl_unified_event!(RaydiumCpmmDepositEvent);

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