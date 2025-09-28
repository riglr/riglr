//! `MarginFi` protocol event definitions, types, and parsers.
//!
//! This module provides comprehensive support for parsing and handling MarginFi protocol events
//! including deposits, withdrawals, borrows, repayments, and liquidations.

use crate::{
    error::{Error as ParseError, ParseResult},
    events::{
        common::{
            has_discriminator, parse_u64_le, safe_get_account, validate_account_count,
            validate_data_length,
        },
        core::EventParameters,
        factory::{InnerInstructionParseParams, InstructionParseParams, SolanaTransactionInput},
        parser_types::{GenericEventParseConfig, ProtocolParser},
    },
    metadata_helpers::{self, SolanaMetadataParams},
    solana_metadata::SolanaEventMetadata,
    types::{EventType, ProtocolType, TransferData},
};
use core::any::Any;
use riglr_events_core::{
    error::{EventError, EventResult},
    traits::{EventParser, ParserInfo},
    Event, EventKind, EventMetadata as CoreEventMetadata,
};
use serde::{Deserialize, Serialize};
use solana_message::compiled_instruction::CompiledInstruction;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::OnceLock};

// ==================== CONSTANTS ====================

/// `MarginFi` program ID
pub const MARGINFI_PROGRAM_ID: &str = "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA";

/// `MarginFi` Bank program ID (for lending pools)
pub const MARGINFI_BANK_PROGRAM_ID: &str = "4Be9aW2D8f3G2b3ZP8uo5kd9z8zwJ3FYD1tNKmQ9c9xA";

/// Discriminator for lending deposit instruction
pub const MARGINFI_DEPOSIT_DISCRIMINATOR: [u8; 8] =
    [0x13, 0x65, 0x32, 0x1f, 0x7a, 0x43, 0x2a, 0x9f];
/// Discriminator for lending withdraw instruction
pub const MARGINFI_WITHDRAW_DISCRIMINATOR: [u8; 8] =
    [0x4c, 0x1c, 0x9b, 0x2d, 0xe3, 0x7a, 0x8b, 0x12];
/// Discriminator for lending borrow instruction
pub const MARGINFI_BORROW_DISCRIMINATOR: [u8; 8] = [0xa2, 0xfd, 0x67, 0xe3, 0x45, 0x1b, 0x8c, 0x9a];
/// Discriminator for lending repay instruction
pub const MARGINFI_REPAY_DISCRIMINATOR: [u8; 8] = [0x85, 0x72, 0x1a, 0x5f, 0x9d, 0x4e, 0x23, 0x7c];
/// Discriminator for liquidation instruction
pub const MARGINFI_LIQUIDATE_DISCRIMINATOR: [u8; 8] =
    [0x6a, 0x8b, 0x47, 0x2e, 0x1c, 0x93, 0x5f, 0x4d];

// ==================== TYPES ====================

/// `MarginFi` account types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarginFiAccountType {
    /// Bank account representing a lending pool
    Bank,
    /// Individual user account for lending positions
    MarginfiAccount,
    /// `MarginFi` group account containing global settings
    MarginfiGroup,
    /// Unknown or unrecognized account type
    Unknown,
}

/// `MarginFi` bank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarginFiBankConfig {
    /// Bank account public key
    pub bank: Pubkey,
    /// Authority that can modify bank settings
    pub bank_authority: Pubkey,
    /// Maximum amount that can be borrowed
    pub borrow_limit: u64,
    /// Outstanding insurance fees collected
    pub collected_insurance_fees_outstanding: u64,
    /// Maximum amount that can be deposited
    pub deposit_limit: u64,
    /// Fee rate charged on operations
    pub fee_rate: u64,
    /// Insurance fee rate for risk coverage
    pub insurance_fee_rate: u64,
    /// Vault holding insurance funds
    pub insurance_vault: Pubkey,
    /// Token mint for this bank
    pub mint: Pubkey,
    /// Current operational state of the bank
    pub operational_state: u8,
    /// Price oracle for this token
    pub oracle: Pubkey,
    /// Array of oracle public keys
    pub oracle_keys: [Pubkey; 5],
    /// Oracle configuration setup
    pub oracle_setup: u8,
    /// Liquidity vault holding deposited tokens
    pub vault: Pubkey,
}

/// `MarginFi` bank state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarginFiBankState {
    /// Value per asset share
    pub asset_share_value: u128,
    /// Current borrowing interest rate
    pub borrowing_rate: u64,
    /// Outstanding group fees collected
    pub collected_group_fees_outstanding: u64,
    /// Fee collection vault
    pub fee_vault: Pubkey,
    /// Authority for the fee vault
    pub fee_vault_authority: Pubkey,
    /// Bump seed for fee vault authority
    pub fee_vault_authority_bump: u8,
    /// Authority for the insurance vault
    pub insurance_vault_authority: Pubkey,
    /// Bump seed for insurance vault authority
    pub insurance_vault_authority_bump: u8,
    /// Timestamp of last state update
    pub last_update: u64,
    /// Current lending interest rate
    pub lending_rate: u64,
    /// Value per liability share
    pub liability_share_value: u128,
    /// Authority for the liquidity vault
    pub liquidity_vault_authority: Pubkey,
    /// Bump seed for liquidity vault authority
    pub liquidity_vault_authority_bump: u8,
    /// Total shares representing assets in the bank
    pub total_asset_shares: u128,
    /// Total shares representing liabilities in the bank
    pub total_liability_shares: u128,
}

/// `MarginFi` account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarginFiBalance {
    /// Whether this balance is active
    pub active: bool,
    /// Shares representing deposited assets
    pub asset_shares: u128,
    /// Bank public key for this balance
    pub bank_pk: Pubkey,
    /// Outstanding emission rewards
    pub emissions_outstanding: u64,
    /// Timestamp of last balance update
    pub last_update: u64,
    /// Shares representing borrowed liabilities
    pub liability_shares: u128,
    /// Padding for future use
    pub padding: [u64; 1],
}

/// `MarginFi` lending account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarginFiLendingAccount {
    /// Array of balances for different tokens
    pub balances: [MarginFiBalance; 16],
    /// Padding for future use
    pub padding: [u64; 8],
}

/// `MarginFi` user account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarginFiAccount {
    /// Account configuration flags
    pub account_flags: u64,
    /// Authority that controls this account
    pub authority: Pubkey,
    /// `MarginFi` group this account belongs to
    pub group: Pubkey,
    /// Lending account with balance information
    pub lending_account: MarginFiLendingAccount,
    /// Padding for future use
    pub padding: [u128; 8],
}

/// `MarginFi` deposit data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiDepositData {
    /// Amount being deposited
    pub amount: u64,
    /// Bank receiving the deposit
    pub bank: Pubkey,
    /// Bank's liquidity vault being credited
    pub bank_liquidity_vault: Pubkey,
    /// User's `MarginFi` account
    pub marginfi_account: Pubkey,
    /// `MarginFi` group for the deposit
    pub marginfi_group: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// User's token account being debited
    pub token_account: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
}

/// `MarginFi` withdraw data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiWithdrawData {
    /// Amount being withdrawn
    pub amount: u64,
    /// Bank from which tokens are withdrawn
    pub bank: Pubkey,
    /// Bank's liquidity vault being debited
    pub bank_liquidity_vault: Pubkey,
    /// Authority for the bank's liquidity vault
    pub bank_liquidity_vault_authority: Pubkey,
    /// User's `MarginFi` account
    pub marginfi_account: Pubkey,
    /// `MarginFi` group for the withdrawal
    pub marginfi_group: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// User's token account being credited
    pub token_account: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
    /// Whether to withdraw all available tokens
    pub withdraw_all: bool,
}

/// `MarginFi` borrow data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiBorrowData {
    /// Amount being borrowed
    pub amount: u64,
    /// Bank from which tokens are borrowed
    pub bank: Pubkey,
    /// Bank's liquidity vault being debited
    pub bank_liquidity_vault: Pubkey,
    /// Authority for the bank's liquidity vault
    pub bank_liquidity_vault_authority: Pubkey,
    /// User's `MarginFi` account
    pub marginfi_account: Pubkey,
    /// `MarginFi` group for the borrow
    pub marginfi_group: Pubkey,
    /// Transaction signer
    pub signer: Pubkey,
    /// User's token account being credited
    pub token_account: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
}

/// `MarginFi` repay data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiRepayData {
    /// Amount being repaid
    pub amount: u64,
    /// Bank to which tokens are repaid
    pub bank: Pubkey,
    /// Bank's liquidity vault being credited
    pub bank_liquidity_vault: Pubkey,
    /// User's `MarginFi` account
    pub marginfi_account: Pubkey,
    /// `MarginFi` group for the repayment
    pub marginfi_group: Pubkey,
    /// Whether to repay all outstanding debt
    pub repay_all: bool,
    /// Transaction signer
    pub signer: Pubkey,
    /// User's token account being debited
    pub token_account: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
}

/// `MarginFi` liquidation data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiLiquidationData {
    /// Amount of asset being seized
    pub asset_amount: u64,
    /// Bank holding the asset being seized
    pub asset_bank: Pubkey,
    /// Asset bank's liquidity vault
    pub asset_bank_liquidity_vault: Pubkey,
    /// Amount of liability being repaid
    pub liab_amount: u64,
    /// Bank holding the liability being repaid
    pub liab_bank: Pubkey,
    /// Liability bank's liquidity vault
    pub liab_bank_liquidity_vault: Pubkey,
    /// Account being liquidated
    pub liquidatee_marginfi_account: Pubkey,
    /// Liquidator's wallet address
    pub liquidator: Pubkey,
    /// Liquidator's `MarginFi` account
    pub liquidator_marginfi_account: Pubkey,
    /// Liquidator's token account
    pub liquidator_token_account: Pubkey,
    /// `MarginFi` group for the liquidation
    pub marginfi_group: Pubkey,
    /// Token program ID
    pub token_program: Pubkey,
}

// ==================== UTILITY FUNCTIONS ====================

/// Extract `MarginFi` program ID as Pubkey
///
/// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
static MARGINFI_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

#[inline]
#[must_use]
/// Returns the `MarginFi` program ID.
///
/// If the constant is somehow invalid, returns a default pubkey,
/// though this should never happen as it's a hardcoded valid constant.
pub fn program_id() -> Pubkey {
    *MARGINFI_PUBKEY
        .get_or_init(|| Pubkey::try_from(MARGINFI_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default()))
}

/// Extract `MarginFi` bank program ID as Pubkey
///
/// This uses a static lazy-evaluated Pubkey to avoid repeated parsing.
static MARGINFI_BANK_PUBKEY: OnceLock<Pubkey> = OnceLock::new();

#[inline]
#[must_use]
/// Returns the `MarginFi` bank program ID.
///
/// If the constant is somehow invalid, returns a default pubkey,
/// though this should never happen as it's a hardcoded valid constant.
pub fn bank_program_id() -> Pubkey {
    *MARGINFI_BANK_PUBKEY.get_or_init(|| {
        Pubkey::try_from(MARGINFI_BANK_PROGRAM_ID).unwrap_or_else(|_| Pubkey::default())
    })
}

/// Check if the given pubkey is `MarginFi` program
#[inline]
#[must_use]
pub fn is_marginfi_program(prog_id: &Pubkey) -> bool {
    *prog_id == program_id() || *prog_id == bank_program_id()
}

/// Calculate health ratio for a `MarginFi` account
#[inline]
#[must_use]
pub fn calculate_health_ratio(
    total_asset_value: u128,
    total_liability_value: u128,
    maintenance_margin: u64,
) -> f64 {
    if total_liability_value == 0 {
        return f64::INFINITY;
    }

    #[expect(clippy::cast_precision_loss)]
    let asset_value = total_asset_value as f64; // Precision loss is acceptable for health ratio calculation
    #[expect(clippy::cast_precision_loss)]
    let liability_value = total_liability_value as f64; // Precision loss is acceptable for health ratio calculation
    #[expect(clippy::cast_precision_loss)]
    let margin_factor = (maintenance_margin as f64) / 10_000.0; // Basis points to decimal, precision loss is acceptable

    (asset_value * margin_factor) / liability_value
}

/// Calculate liquidation threshold
///
/// Returns `None` if multiplication or division operations overflow.
#[inline]
#[must_use]
pub const fn calculate_liquidation_threshold(
    total_asset_value: u128,
    liquidation_ltv: u64,
) -> Option<u128> {
    let ltv_factor = liquidation_ltv as u128;
    if let Some(multiplied) = total_asset_value.checked_mul(ltv_factor) {
        multiplied.checked_div(10_000) // Convert from basis points
    } else {
        None
    }
}

/// Convert shares to amount using share value
///
/// Returns `None` if multiplication or division operations overflow, or if the result cannot fit in a u64.
#[inline]
#[must_use]
pub fn shares_to_amount(shares: u128, share_value: u128) -> Option<u64> {
    shares
        .checked_mul(share_value)?
        .checked_div(1_u128 << 64)?
        .try_into()
        .ok()
}

/// Convert amount to shares using share value
///
/// Returns `None` if multiplication or division operations overflow.
#[inline]
#[must_use]
pub const fn amount_to_shares(amount: u64, share_value: u128) -> Option<u128> {
    if share_value == 0 {
        return Some(amount as u128);
    }
    if let Some(multiplied) = (amount as u128).checked_mul(1_u128 << 64) {
        multiplied.checked_div(share_value)
    } else {
        None
    }
}

// ==================== EVENT DEFINITIONS ====================

/// `MarginFi` deposit event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiDepositEvent {
    /// MarginFi-specific deposit operation data
    pub deposit_data: MarginFiDepositData,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Associated token transfer data for this deposit
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiDepositEvent {
    /// Creates a new `MarginFi` deposit event with the provided parameters and deposit data
    #[inline]
    #[must_use]
    pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self {
        let core_metadata = metadata_helpers::create_solana_metadata(SolanaMetadataParams {
            id: params.id.clone(),
            kind: riglr_events_core::EventKind::Transaction,
            source: "marginfi".to_string(),
            slot: params.slot,
            signature: Some(params.signature.clone()),
            program_id: Some(program_id()),
            instruction_index: params.index.parse().ok(),
            block_time: Some(params.block_time),
            protocol_type: &ProtocolType::MarginFi,
            event_type: &EventType::Deposit,
        });
        let metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Deposit,
            ProtocolType::MarginFi,
            params.index,
            params.program_received_time_ms,
            core_metadata,
        );

        Self {
            metadata,
            deposit_data,
            transfer_data: Vec::default(),
        }
    }

    /// Adds transfer data to the deposit event and returns the modified event
    #[inline]
    #[must_use]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Event for MarginFiDepositEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// `MarginFi` withdraw event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiWithdrawEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Associated token transfer data for this withdrawal
    pub transfer_data: Vec<TransferData>,
    /// MarginFi-specific withdraw operation data
    pub withdraw_data: MarginFiWithdrawData,
}

impl MarginFiWithdrawEvent {
    /// Creates a new `MarginFi` withdraw event with the provided parameters and withdraw data
    #[inline]
    #[must_use]
    pub fn new(params: EventParameters, withdraw_data: MarginFiWithdrawData) -> Self {
        let core_metadata = metadata_helpers::create_solana_metadata(SolanaMetadataParams {
            id: params.id.clone(),
            kind: riglr_events_core::EventKind::Transaction,
            source: "marginfi".to_string(),
            slot: params.slot,
            signature: Some(params.signature.clone()),
            program_id: Some(program_id()),
            instruction_index: params.index.parse().ok(),
            block_time: Some(params.block_time),
            protocol_type: &ProtocolType::MarginFi,
            event_type: &EventType::Withdraw,
        });
        let metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Withdraw,
            ProtocolType::MarginFi,
            params.index,
            params.program_received_time_ms,
            core_metadata,
        );

        Self {
            metadata,
            withdraw_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this withdraw event
    #[inline]
    #[must_use]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Event for MarginFiWithdrawEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// `MarginFi` borrow event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiBorrowEvent {
    /// MarginFi-specific borrow operation data
    pub borrow_data: MarginFiBorrowData,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Associated token transfer data for this borrow
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiBorrowEvent {
    /// Creates a new `MarginFi` borrow event with the provided parameters and borrow data
    #[inline]
    #[must_use]
    pub fn new(params: EventParameters, borrow_data: MarginFiBorrowData) -> Self {
        let core_metadata = metadata_helpers::create_solana_metadata(SolanaMetadataParams {
            id: params.id.clone(),
            kind: riglr_events_core::EventKind::Transaction,
            source: "marginfi".to_string(),
            slot: params.slot,
            signature: Some(params.signature.clone()),
            program_id: Some(program_id()),
            instruction_index: params.index.parse().ok(),
            block_time: Some(params.block_time),
            protocol_type: &ProtocolType::MarginFi,
            event_type: &EventType::Borrow,
        });
        let metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Borrow,
            ProtocolType::MarginFi,
            params.index,
            params.program_received_time_ms,
            core_metadata,
        );

        Self {
            metadata,
            borrow_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this borrow event
    #[inline]
    #[must_use]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Event for MarginFiBorrowEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// `MarginFi` repay event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiRepayEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific repay operation data
    pub repay_data: MarginFiRepayData,
    /// Associated token transfer data for this repayment
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiRepayEvent {
    /// Creates a new `MarginFi` repay event with the provided parameters and repay data
    #[inline]
    #[must_use]
    pub fn new(params: EventParameters, repay_data: MarginFiRepayData) -> Self {
        let core_metadata = metadata_helpers::create_solana_metadata(SolanaMetadataParams {
            id: params.id.clone(),
            kind: riglr_events_core::EventKind::Transaction,
            source: "marginfi".to_string(),
            slot: params.slot,
            signature: Some(params.signature.clone()),
            program_id: Some(program_id()),
            instruction_index: params.index.parse().ok(),
            block_time: Some(params.block_time),
            protocol_type: &ProtocolType::MarginFi,
            event_type: &EventType::Repay,
        });
        let metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Repay,
            ProtocolType::MarginFi,
            params.index,
            params.program_received_time_ms,
            core_metadata,
        );

        Self {
            metadata,
            repay_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this repay event
    #[inline]
    #[must_use]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Event for MarginFiRepayEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// `MarginFi` liquidation event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct MarginFiLiquidationEvent {
    /// MarginFi-specific liquidation operation data
    pub liquidation_data: MarginFiLiquidationData,
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Associated token transfer data for this liquidation
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiLiquidationEvent {
    /// Creates a new `MarginFi` liquidation event with the provided parameters and liquidation data
    #[inline]
    #[must_use]
    pub fn new(params: EventParameters, liquidation_data: MarginFiLiquidationData) -> Self {
        let core_metadata = metadata_helpers::create_solana_metadata(SolanaMetadataParams {
            id: params.id.clone(),
            kind: riglr_events_core::EventKind::Transaction,
            source: "marginfi".to_string(),
            slot: params.slot,
            signature: Some(params.signature.clone()),
            program_id: Some(program_id()),
            instruction_index: params.index.parse().ok(),
            block_time: Some(params.block_time),
            protocol_type: &ProtocolType::MarginFi,
            event_type: &EventType::Liquidate,
        });
        let metadata = SolanaEventMetadata::new(
            params.signature,
            params.slot,
            EventType::Liquidate,
            ProtocolType::MarginFi,
            params.index,
            params.program_received_time_ms,
            core_metadata,
        );

        Self {
            metadata,
            liquidation_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this liquidation event
    #[inline]
    #[must_use]
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

impl Event for MarginFiLiquidationEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.core.id
    }
    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }
    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }
    fn metadata_mut(&mut self) -> EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| EventError::generic(format!("Serialization failed: {e}")))
    }
}

/// Calculate interest rate based on utilization
///
/// Returns `None` if any arithmetic operations overflow.
#[inline]
#[must_use]
pub const fn calculate_interest_rate(
    utilization_rate: u64,
    base_rate: u64,
    slope1: u64,
    slope2: u64,
    optimal_utilization: u64,
) -> Option<u64> {
    if utilization_rate <= optimal_utilization {
        if optimal_utilization == 0 {
            return Some(base_rate);
        }
        if let Some(mul_result) = utilization_rate.checked_mul(slope1) {
            if let Some(slope_calc) = mul_result.checked_div(optimal_utilization) {
                return base_rate.checked_add(slope_calc);
            }
        }
        return None;
    }
    if let Some(excess_utilization) = utilization_rate.checked_sub(optimal_utilization) {
        if let Some(max_excess) = 10_000_u64.checked_sub(optimal_utilization) {
            if max_excess == 0 {
                return Some(base_rate);
            }
            if let Some(mul_result) = excess_utilization.checked_mul(slope2) {
                if let Some(slope_calc) = mul_result.checked_div(max_excess) {
                    if let Some(base_plus_slope1) = base_rate.checked_add(slope1) {
                        return base_plus_slope1.checked_add(slope_calc);
                    }
                }
            }
        }
    }
    None
}

// ==================== PARSER IMPLEMENTATION ====================

/// `MarginFi` event parser
#[derive(Debug)]
#[non_exhaustive]
pub struct MarginFiEventParser {
    info: ParserInfo,
    inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>,
    instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>,
    program_ids: Vec<Pubkey>,
}

impl MarginFiEventParser {
    /// Helper method to return inner instruction configs (for testing)
    #[inline]
    #[must_use]
    pub fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }

    /// Helper method to return instruction configs (for testing)
    #[inline]
    #[must_use]
    pub fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }
}

// Implement the new core EventParser trait
#[async_trait::async_trait]
impl EventParser for MarginFiEventParser {
    type Input = SolanaTransactionInput;
    fn can_parse(&self, input: &Self::Input) -> bool {
        match *input {
            SolanaTransactionInput::InnerInstruction(_) => true,
            SolanaTransactionInput::Instruction(ref params) => {
                // Check if the instruction matches our discriminators
                self.instruction_configs
                    .keys()
                    .any(|disc| params.instruction_data.starts_with(disc))
            }
        }
    }
    fn info(&self) -> &ParserInfo {
        &self.info
    }

    async fn parse(&self, input: Self::Input) -> EventResult<Vec<Box<dyn Event>>> {
        let events = match input {
            SolanaTransactionInput::InnerInstruction(params) => {
                let legacy_params = InnerInstructionParseParams {
                    inner_instruction: &solana_transaction_status::UiCompiledInstruction {
                        program_id_index: 0,
                        accounts: vec![],
                        data: params.inner_instruction_data.clone(),
                        stack_height: Some(1),
                    },
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.parse_events_from_inner_instruction_impl(&legacy_params)
            }
            SolanaTransactionInput::Instruction(params) => {
                let instruction = CompiledInstruction {
                    program_id_index: 0,
                    accounts: vec![],
                    data: params.instruction_data.clone(),
                };
                let legacy_params = InstructionParseParams {
                    instruction: &instruction,
                    accounts: &params.accounts,
                    signature: &params.signature,
                    slot: params.slot,
                    block_time: params.block_time,
                    program_received_time_ms: params.program_received_time_ms,
                    index: params.index.clone(),
                };
                self.parse_events_from_instruction_impl(&legacy_params)
            }
        };
        Ok(events)
    }
}

// Implement the legacy EventParser trait for backward compatibility
impl ProtocolParser for MarginFiEventParser {
    fn inner_instruction_configs(&self) -> HashMap<&'static str, Vec<GenericEventParseConfig>> {
        self.inner_instruction_configs.clone()
    }
    fn instruction_configs(&self) -> HashMap<Vec<u8>, Vec<GenericEventParseConfig>> {
        self.instruction_configs.clone()
    }
    fn parse_events_from_inner_instruction(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.parse_events_from_inner_instruction_impl(params)
    }
    fn parse_events_from_instruction(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        self.parse_events_from_instruction_impl(params)
    }
    fn should_handle(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }
    fn supported_program_ids(&self) -> Vec<Pubkey> {
        self.program_ids.clone()
    }
}

impl MarginFiEventParser {
    fn parse_events_from_inner_instruction_impl(
        &self,
        params: &InnerInstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // For inner instructions, we'll use the data to identify the instruction type
        if let Ok(data) = bs58::decode(&params.inner_instruction.data).into_vec() {
            for configs in self.inner_instruction_configs.values() {
                for config in configs {
                    let core_metadata =
                        metadata_helpers::create_solana_metadata(SolanaMetadataParams {
                            id: format!("{}_{}", params.signature, params.index),
                            kind: riglr_events_core::EventKind::Transaction,
                            source: "marginfi".to_string(),
                            slot: params.slot,
                            signature: Some(params.signature.to_owned()),
                            program_id: Some(config.program_id),
                            instruction_index: params.index.parse().ok(),
                            block_time: params.block_time,
                            protocol_type: &config.protocol_type,
                            event_type: &config.event_type,
                        });
                    let metadata = SolanaEventMetadata::new(
                        params.signature.to_owned(),
                        params.slot,
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                        params.index.clone(),
                        params.program_received_time_ms,
                        core_metadata,
                    );

                    if let Ok(event) = (config.inner_instruction_parser)(&data, metadata) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }
    fn parse_events_from_instruction_impl(
        &self,
        params: &InstructionParseParams<'_>,
    ) -> Vec<Box<dyn Event>> {
        let mut events = Vec::new();

        // Check each discriminator
        for (discriminator, configs) in &self.instruction_configs {
            if has_discriminator(&params.instruction.data, discriminator) {
                for config in configs {
                    let core_metadata =
                        metadata_helpers::create_solana_metadata(SolanaMetadataParams {
                            id: format!("{}_{}", params.signature, params.index),
                            kind: riglr_events_core::EventKind::Transaction,
                            source: "marginfi".to_string(),
                            slot: params.slot,
                            signature: Some(params.signature.to_owned()),
                            program_id: Some(config.program_id),
                            instruction_index: params.index.parse().ok(),
                            block_time: params.block_time,
                            protocol_type: &config.protocol_type,
                            event_type: &config.event_type,
                        });
                    let metadata = SolanaEventMetadata::new(
                        params.signature.to_owned(),
                        params.slot,
                        config.event_type.clone(),
                        config.protocol_type.clone(),
                        params.index.clone(),
                        params.program_received_time_ms,
                        core_metadata,
                    );

                    if let Ok(event) = (config.instruction_parser)(
                        &params.instruction.data,
                        params.accounts,
                        metadata,
                    ) {
                        events.push(event);
                    }
                }
            }
        }

        events
    }
}

impl Default for MarginFiEventParser {
    fn default() -> Self {
        let program_ids = vec![program_id(), bank_program_id()];

        let configs = vec![
            GenericEventParseConfig {
                program_id: program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountDeposit",
                instruction_discriminator: &MARGINFI_DEPOSIT_DISCRIMINATOR,
                event_type: EventType::AddLiquidity,
                inner_instruction_parser: |data, metadata| {
                    parse_marginfi_deposit_inner_instruction(data, &metadata)
                },
                instruction_parser: |data, accounts, metadata| {
                    parse_marginfi_deposit_instruction(data, accounts, &metadata)
                },
            },
            GenericEventParseConfig {
                program_id: program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountWithdraw",
                instruction_discriminator: &MARGINFI_WITHDRAW_DISCRIMINATOR,
                event_type: EventType::RemoveLiquidity,
                inner_instruction_parser: |data, metadata| {
                    parse_marginfi_withdraw_inner_instruction(data, &metadata)
                },
                instruction_parser: |data, accounts, metadata| {
                    parse_marginfi_withdraw_instruction(data, accounts, &metadata)
                },
            },
            GenericEventParseConfig {
                program_id: program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountBorrow",
                instruction_discriminator: &MARGINFI_BORROW_DISCRIMINATOR,
                event_type: EventType::Borrow,
                inner_instruction_parser: |data, metadata| {
                    parse_marginfi_borrow_inner_instruction(data, &metadata)
                },
                instruction_parser: |data, accounts, metadata| {
                    parse_marginfi_borrow_instruction(data, accounts, &metadata)
                },
            },
            GenericEventParseConfig {
                program_id: program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountRepay",
                instruction_discriminator: &MARGINFI_REPAY_DISCRIMINATOR,
                event_type: EventType::Repay,
                inner_instruction_parser: |data, metadata| {
                    parse_marginfi_repay_inner_instruction(data, &metadata)
                },
                instruction_parser: |data, accounts, metadata| {
                    parse_marginfi_repay_instruction(data, accounts, &metadata)
                },
            },
            GenericEventParseConfig {
                program_id: program_id(),
                protocol_type: ProtocolType::MarginFi,
                inner_instruction_discriminator: "lendingAccountLiquidate",
                instruction_discriminator: &MARGINFI_LIQUIDATE_DISCRIMINATOR,
                event_type: EventType::Liquidate,
                inner_instruction_parser: |data, metadata| {
                    parse_marginfi_liquidate_inner_instruction(data, &metadata)
                },
                instruction_parser: |data, accounts, metadata| {
                    parse_marginfi_liquidate_instruction(data, accounts, &metadata)
                },
            },
        ];

        let mut inner_instruction_configs = HashMap::new();
        let mut instruction_configs = HashMap::new();

        for config in configs {
            inner_instruction_configs
                .entry(config.inner_instruction_discriminator)
                .or_insert_with(Vec::new)
                .push(config.clone());
            instruction_configs
                .entry(config.instruction_discriminator.to_vec())
                .or_insert_with(Vec::new)
                .push(config);
        }

        let info = ParserInfo::new("marginfi_parser".to_owned(), "1.0.0".to_owned())
            .with_kind(EventKind::Transaction)
            .with_format("solana_instruction".to_owned());

        Self {
            info,
            inner_instruction_configs,
            instruction_configs,
            program_ids,
        }
    }
}

// ==================== PARSER FUNCTIONS ====================

fn parse_marginfi_deposit_inner_instruction(
    data: &[u8],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let deposit_data = parse_marginfi_deposit_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse MarginFi deposit data".to_owned())
    })?;

    Ok(Box::new(MarginFiDepositEvent::new(
        EventParameters::new(
            metadata.id().to_owned(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        deposit_data,
    )) as Box<dyn Event>)
}

fn parse_marginfi_deposit_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let deposit_data =
        parse_marginfi_deposit_data_from_instruction(data, accounts).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Failed to parse MarginFi deposit instruction data".to_owned(),
            )
        })?;

    Ok(Box::new(MarginFiDepositEvent::new(
        EventParameters::new(
            metadata.id().to_owned(),
            metadata.signature.clone(),
            metadata.slot,
            0, // block_time - not available in SolanaEventMetadata
            0, // block_time_ms - not available in SolanaEventMetadata
            metadata.program_received_time_ms,
            metadata.index.clone(),
        ),
        deposit_data,
    )) as Box<dyn Event>)
}

fn parse_marginfi_withdraw_inner_instruction(
    data: &[u8],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let withdraw_data = parse_marginfi_withdraw_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse MarginFi withdraw data".to_owned())
    })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiWithdrawEvent::new(params, withdraw_data)) as Box<dyn Event>)
}

fn parse_marginfi_withdraw_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let withdraw_data =
        parse_marginfi_withdraw_data_from_instruction(data, accounts).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Failed to parse MarginFi withdraw instruction data".to_owned(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiWithdrawEvent::new(params, withdraw_data)) as Box<dyn Event>)
}

fn parse_marginfi_borrow_inner_instruction(
    data: &[u8],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let borrow_data = parse_marginfi_borrow_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse MarginFi borrow data".to_owned())
    })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiBorrowEvent::new(params, borrow_data)) as Box<dyn Event>)
}

fn parse_marginfi_borrow_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let borrow_data =
        parse_marginfi_borrow_data_from_instruction(data, accounts).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Failed to parse MarginFi borrow instruction data".to_owned(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiBorrowEvent::new(params, borrow_data)) as Box<dyn Event>)
}

fn parse_marginfi_repay_inner_instruction(
    data: &[u8],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let repay_data = parse_marginfi_repay_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse MarginFi repay data".to_owned())
    })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiRepayEvent::new(params, repay_data)) as Box<dyn Event>)
}

fn parse_marginfi_repay_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let repay_data =
        parse_marginfi_repay_data_from_instruction(data, accounts).ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Failed to parse MarginFi repay instruction data".to_owned(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiRepayEvent::new(params, repay_data)) as Box<dyn Event>)
}

fn parse_marginfi_liquidate_inner_instruction(
    data: &[u8],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidation_data = parse_marginfi_liquidate_data(data).ok_or_else(|| {
        ParseError::InvalidDataFormat("Failed to parse MarginFi liquidation data".to_owned())
    })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiLiquidationEvent::new(params, liquidation_data)) as Box<dyn Event>)
}

fn parse_marginfi_liquidate_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: &SolanaEventMetadata,
) -> ParseResult<Box<dyn Event>> {
    let liquidation_data = parse_marginfi_liquidate_data_from_instruction(data, accounts)
        .ok_or_else(|| {
            ParseError::InvalidDataFormat(
                "Failed to parse MarginFi liquidation instruction data".to_owned(),
            )
        })?;

    let params = EventParameters::new(
        metadata.id().to_owned(),
        metadata.signature.clone(),
        metadata.slot,
        0, // block_time - not available in SolanaEventMetadata
        0, // block_time_ms - not available in SolanaEventMetadata
        metadata.program_received_time_ms,
        metadata.index.clone(),
    );

    Ok(Box::new(MarginFiLiquidationEvent::new(params, liquidation_data)) as Box<dyn Event>)
}

// ==================== DATA PARSING HELPERS ====================
fn parse_marginfi_deposit_data(data: &[u8]) -> Option<MarginFiDepositData> {
    validate_data_length(data, 16, "MarginFi deposit data").ok()?;

    let offset: usize = 8; // Skip discriminator
    let amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;

    Some(MarginFiDepositData {
        marginfi_group: Pubkey::default(), // Would need to extract from accounts
        marginfi_account: Pubkey::default(), // Would need to extract from accounts
        signer: Pubkey::default(),         // Would need to extract from accounts
        bank: Pubkey::default(),           // Would need to extract from accounts
        token_account: Pubkey::default(),  // Would need to extract from accounts
        bank_liquidity_vault: Pubkey::default(), // Would need to extract from accounts
        token_program: Pubkey::default(),  // Would need to extract from accounts
        amount,
    })
}
fn parse_marginfi_deposit_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiDepositData> {
    let mut deposit_data = parse_marginfi_deposit_data(data)?;

    // Extract accounts (typical MarginFi deposit instruction layout)
    if validate_account_count(accounts, 8, "MarginFi deposit instruction").is_ok() {
        deposit_data.marginfi_group = safe_get_account(accounts, 1).unwrap_or_default();
        deposit_data.marginfi_account = safe_get_account(accounts, 2).unwrap_or_default();
        deposit_data.signer = safe_get_account(accounts, 0).unwrap_or_default();
        deposit_data.bank = safe_get_account(accounts, 3).unwrap_or_default();
        deposit_data.token_account = safe_get_account(accounts, 4).unwrap_or_default();
        deposit_data.bank_liquidity_vault = safe_get_account(accounts, 5).unwrap_or_default();
        deposit_data.token_program = safe_get_account(accounts, 6).unwrap_or_default();
    }

    Some(deposit_data)
}
fn parse_marginfi_withdraw_data(data: &[u8]) -> Option<MarginFiWithdrawData> {
    if data.len() < 17 {
        return None;
    }

    let mut offset: usize = 8; // Skip discriminator
    let amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;
    {
        offset = offset.checked_add(8)?;
    }
    let withdraw_all = data.get(offset)? != &0;

    Some(MarginFiWithdrawData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        bank_liquidity_vault_authority: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
        withdraw_all,
    })
}
fn parse_marginfi_withdraw_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiWithdrawData> {
    let mut withdraw_data = parse_marginfi_withdraw_data(data)?;

    if accounts.len() >= 9 {
        withdraw_data.marginfi_group = *accounts.get(1)?;
        withdraw_data.marginfi_account = *accounts.get(2)?;
        withdraw_data.signer = *accounts.first()?;
        withdraw_data.bank = *accounts.get(3)?;
        withdraw_data.token_account = *accounts.get(4)?;
        withdraw_data.bank_liquidity_vault = *accounts.get(5)?;
        withdraw_data.bank_liquidity_vault_authority = *accounts.get(6)?;
        withdraw_data.token_program = *accounts.get(7)?;
    }

    Some(withdraw_data)
}
fn parse_marginfi_borrow_data(data: &[u8]) -> Option<MarginFiBorrowData> {
    if data.len() < 16 {
        return None;
    }

    let offset: usize = 8; // Skip discriminator
    let amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;

    Some(MarginFiBorrowData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        bank_liquidity_vault_authority: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
    })
}
fn parse_marginfi_borrow_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiBorrowData> {
    let mut borrow_data = parse_marginfi_borrow_data(data)?;

    if accounts.len() >= 9 {
        borrow_data.marginfi_group = *accounts.get(1)?;
        borrow_data.marginfi_account = *accounts.get(2)?;
        borrow_data.signer = *accounts.first()?;
        borrow_data.bank = *accounts.get(3)?;
        borrow_data.token_account = *accounts.get(4)?;
        borrow_data.bank_liquidity_vault = *accounts.get(5)?;
        borrow_data.bank_liquidity_vault_authority = *accounts.get(6)?;
        borrow_data.token_program = *accounts.get(7)?;
    }

    Some(borrow_data)
}
fn parse_marginfi_repay_data(data: &[u8]) -> Option<MarginFiRepayData> {
    if data.len() < 17 {
        return None;
    }

    let mut offset: usize = 8; // Skip discriminator
    let amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;
    {
        offset = offset.checked_add(8)?;
    }
    let repay_all = data.get(offset)? != &0;

    Some(MarginFiRepayData {
        marginfi_group: Pubkey::default(),
        marginfi_account: Pubkey::default(),
        signer: Pubkey::default(),
        bank: Pubkey::default(),
        token_account: Pubkey::default(),
        bank_liquidity_vault: Pubkey::default(),
        token_program: Pubkey::default(),
        amount,
        repay_all,
    })
}
fn parse_marginfi_repay_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiRepayData> {
    let mut repay_data = parse_marginfi_repay_data(data)?;

    if accounts.len() >= 8 {
        repay_data.marginfi_group = *accounts.get(1)?;
        repay_data.marginfi_account = *accounts.get(2)?;
        repay_data.signer = *accounts.first()?;
        repay_data.bank = *accounts.get(3)?;
        repay_data.token_account = *accounts.get(4)?;
        repay_data.bank_liquidity_vault = *accounts.get(5)?;
        repay_data.token_program = *accounts.get(6)?;
    }

    Some(repay_data)
}
fn parse_marginfi_liquidate_data(data: &[u8]) -> Option<MarginFiLiquidationData> {
    if data.len() < 24 {
        return None;
    }

    let mut offset: usize = 8; // Skip discriminator
    let asset_amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;
    {
        offset = offset.checked_add(8)?;
    }
    let liab_amount = parse_u64_le(data.get(offset..offset.checked_add(8)?)?).ok()?;

    Some(MarginFiLiquidationData {
        marginfi_group: Pubkey::default(),
        asset_bank: Pubkey::default(),
        liab_bank: Pubkey::default(),
        liquidatee_marginfi_account: Pubkey::default(),
        liquidator_marginfi_account: Pubkey::default(),
        liquidator: Pubkey::default(),
        asset_bank_liquidity_vault: Pubkey::default(),
        liab_bank_liquidity_vault: Pubkey::default(),
        liquidator_token_account: Pubkey::default(),
        token_program: Pubkey::default(),
        asset_amount,
        liab_amount,
    })
}
fn parse_marginfi_liquidate_data_from_instruction(
    data: &[u8],
    accounts: &[Pubkey],
) -> Option<MarginFiLiquidationData> {
    let mut liquidation_data = parse_marginfi_liquidate_data(data)?;

    if accounts.len() >= 12 {
        liquidation_data.marginfi_group = *accounts.get(1)?;
        liquidation_data.asset_bank = *accounts.get(2)?;
        liquidation_data.liab_bank = *accounts.get(3)?;
        liquidation_data.liquidatee_marginfi_account = *accounts.get(4)?;
        liquidation_data.liquidator_marginfi_account = *accounts.get(5)?;
        liquidation_data.liquidator = *accounts.first()?;
        liquidation_data.asset_bank_liquidity_vault = *accounts.get(6)?;
        liquidation_data.liab_bank_liquidity_vault = *accounts.get(7)?;
        liquidation_data.liquidator_token_account = *accounts.get(8)?;
        liquidation_data.token_program = *accounts.get(9)?;
    }

    Some(liquidation_data)
}

// ==================== TESTS ====================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // Test module structure and re-exports
    #[test]
    fn module_reexports_all_public_types() {
        // Test that all major types are accessible through re-exports
        let _: EventParameters = EventParameters::default();
        let _: MarginFiDepositEvent = MarginFiDepositEvent::default();
        let _: MarginFiWithdrawEvent = MarginFiWithdrawEvent::default();
        let _: MarginFiBorrowEvent = MarginFiBorrowEvent::default();
        let _: MarginFiRepayEvent = MarginFiRepayEvent::default();
        let _: MarginFiLiquidationEvent = MarginFiLiquidationEvent::default();

        // Test data types
        let _: MarginFiDepositData = MarginFiDepositData::default();
        let _: MarginFiWithdrawData = MarginFiWithdrawData::default();
        let _: MarginFiBorrowData = MarginFiBorrowData::default();
        let _: MarginFiRepayData = MarginFiRepayData::default();
        let _: MarginFiLiquidationData = MarginFiLiquidationData::default();

        // Test parser
        let _: MarginFiEventParser = MarginFiEventParser::default();

        // Test utility functions
        let _: Pubkey = program_id();
        let _: Pubkey = bank_program_id();
    }

    #[test]
    fn event_parameters_creation_and_usage() {
        let params = EventParameters::new(
            "test_id".to_owned(),
            "test_signature".to_owned(),
            12345,
            1_640_995_200,
            1_640_995_200_000,
            1_640_995_200_500,
            "0".to_owned(),
        );

        // Test that parameters are stored correctly
        assert_eq!(params.id, "test_id");
        assert_eq!(params.signature, "test_signature");
        assert_eq!(params.slot, 12345);
        assert_eq!(params.block_time, 1_640_995_200);
        assert_eq!(params.block_time_ms, 1_640_995_200_000);
        assert_eq!(params.program_received_time_ms, 1_640_995_200_500);
        assert_eq!(params.index, "0");

        // Test event creation with parameters
        let deposit_data = MarginFiDepositData::default();
        let event = MarginFiDepositEvent::new(params, deposit_data);

        assert_eq!(event.id(), "test_id");
        assert_eq!(event.metadata.signature, "test_signature");
        assert_eq!(event.metadata.slot, 12345);
    }

    #[test]
    fn event_parameters_default() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    #[test]
    fn marginfi_deposit_event_creation() {
        let params = EventParameters::new(
            "deposit_test".to_owned(),
            "sig123".to_owned(),
            100,
            1_640_995_200,
            1_640_995_200_000,
            1_640_995_200_100,
            "1".to_owned(),
        );

        let deposit_data = MarginFiDepositData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 1_000_000,
        };

        let event = MarginFiDepositEvent::new(params, deposit_data);

        assert_eq!(event.id(), "deposit_test");
        assert_eq!(event.metadata.signature, "sig123");
        assert_eq!(event.metadata.slot, 100);
        assert_eq!(event.deposit_data.amount, 1_000_000);
        assert!(event.transfer_data.is_empty());
        assert_eq!(event.metadata.program_received_time_ms, 0);
    }

    #[test]
    fn marginfi_deposit_event_with_transfer_data() {
        let params = EventParameters::default();
        let deposit_data = MarginFiDepositData::default();

        let transfer_data = vec![
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 500_000,
                mint: Some(Pubkey::new_unique()),
            },
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 250_000,
                mint: Some(Pubkey::new_unique()),
            },
        ];

        let event =
            MarginFiDepositEvent::new(params, deposit_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        let second_transfer = event
            .transfer_data
            .get(1)
            .expect("Second transfer should exist");
        assert_eq!(first_transfer.amount, 500_000);
        assert_eq!(second_transfer.amount, 250_000);
    }

    #[test]
    fn marginfi_deposit_event_default() {
        let event = MarginFiDepositEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(
            {
                #[allow(clippy::absolute_paths)]
                metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0)
            },
            0
        );
        assert_eq!(
            {
                #[allow(clippy::absolute_paths)]
                metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0)
            } * 1000,
            0
        );
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.program_received_time_ms, 0);
        assert_eq!(event.metadata.index, "");
        assert_eq!(event.deposit_data.amount, 0);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn marginfi_withdraw_event_creation() {
        let params = EventParameters::new(
            "withdraw_test".to_owned(),
            "sig456".to_owned(),
            200,
            1_640_995_300,
            1_640_995_300_000,
            1_640_995_300_200,
            "2".to_owned(),
        );

        let withdraw_data = MarginFiWithdrawData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            bank_liquidity_vault_authority: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 2_000_000,
            withdraw_all: true,
        };

        let event = MarginFiWithdrawEvent::new(params, withdraw_data);

        assert_eq!(event.id(), "withdraw_test");
        assert_eq!(event.metadata.signature, "sig456");
        assert_eq!(event.metadata.slot, 200);
        assert_eq!(event.withdraw_data.amount, 2_000_000);
        assert!(event.withdraw_data.withdraw_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn marginfi_withdraw_event_with_transfer_data() {
        let params = EventParameters::default();
        let withdraw_data = MarginFiWithdrawData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 750_000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event =
            MarginFiWithdrawEvent::new(params, withdraw_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 1);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        assert_eq!(first_transfer.amount, 750_000);
    }

    #[test]
    fn marginfi_borrow_event_creation() {
        let params = EventParameters::new(
            "borrow_test".to_owned(),
            "sig789".to_owned(),
            300,
            1_640_995_400,
            1_640_995_400_000,
            1_640_995_400_300,
            "3".to_owned(),
        );

        let borrow_data = MarginFiBorrowData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            bank_liquidity_vault_authority: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 3_000_000,
        };

        let event = MarginFiBorrowEvent::new(params, borrow_data);

        assert_eq!(event.id(), "borrow_test");
        assert_eq!(event.metadata.signature, "sig789");
        assert_eq!(event.metadata.slot, 300);
        assert_eq!(event.borrow_data.amount, 3_000_000);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn marginfi_borrow_event_with_transfer_data() {
        let params = EventParameters::default();
        let borrow_data = MarginFiBorrowData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 1_500_000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event = MarginFiBorrowEvent::new(params, borrow_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 1);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        assert_eq!(first_transfer.amount, 1_500_000);
    }

    #[test]
    fn marginfi_repay_event_creation() {
        let params = EventParameters::new(
            "repay_test".to_owned(),
            "sig101112".to_owned(),
            400,
            1_640_995_500,
            1_640_995_500_000,
            1_640_995_500_400,
            "4".to_owned(),
        );

        let repay_data = MarginFiRepayData {
            marginfi_group: Pubkey::new_unique(),
            marginfi_account: Pubkey::new_unique(),
            signer: Pubkey::new_unique(),
            bank: Pubkey::new_unique(),
            token_account: Pubkey::new_unique(),
            bank_liquidity_vault: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            amount: 4_000_000,
            repay_all: false,
        };

        let event = MarginFiRepayEvent::new(params, repay_data);

        assert_eq!(event.id(), "repay_test");
        assert_eq!(event.metadata.signature, "sig101112");
        assert_eq!(event.metadata.slot, 400);
        assert_eq!(event.repay_data.amount, 4_000_000);
        assert!(!event.repay_data.repay_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn marginfi_repay_event_with_transfer_data() {
        let params = EventParameters::default();
        let repay_data = MarginFiRepayData::default();

        let transfer_data = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 2_250_000,
            mint: Some(Pubkey::new_unique()),
        }];

        let event = MarginFiRepayEvent::new(params, repay_data).with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 1);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        assert_eq!(first_transfer.amount, 2_250_000);
    }

    #[test]
    fn marginfi_liquidation_event_creation() {
        let params = EventParameters::new(
            "liquidation_test".to_owned(),
            "sig131415".to_owned(),
            500,
            1_640_995_600,
            1_640_995_600_000,
            1_640_995_600_500,
            "5".to_owned(),
        );

        let liquidation_data = MarginFiLiquidationData {
            marginfi_group: Pubkey::new_unique(),
            asset_bank: Pubkey::new_unique(),
            liab_bank: Pubkey::new_unique(),
            liquidatee_marginfi_account: Pubkey::new_unique(),
            liquidator_marginfi_account: Pubkey::new_unique(),
            liquidator: Pubkey::new_unique(),
            asset_bank_liquidity_vault: Pubkey::new_unique(),
            liab_bank_liquidity_vault: Pubkey::new_unique(),
            liquidator_token_account: Pubkey::new_unique(),
            token_program: Pubkey::new_unique(),
            asset_amount: 5_000_000,
            liab_amount: 4_500_000,
        };

        let event = MarginFiLiquidationEvent::new(params, liquidation_data);

        assert_eq!(event.id(), "liquidation_test");
        assert_eq!(event.metadata.signature, "sig131415");
        assert_eq!(event.metadata.slot, 500);
        assert_eq!(event.liquidation_data.asset_amount, 5_000_000);
        assert_eq!(event.liquidation_data.liab_amount, 4_500_000);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn marginfi_liquidation_event_with_transfer_data() {
        let params = EventParameters::default();
        let liquidation_data = MarginFiLiquidationData::default();

        let transfer_data = vec![
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 3_000_000,
                mint: Some(Pubkey::new_unique()),
            },
            TransferData {
                source: Pubkey::new_unique(),
                destination: Pubkey::new_unique(),
                amount: 2_700_000,
                mint: Some(Pubkey::new_unique()),
            },
        ];

        let event = MarginFiLiquidationEvent::new(params, liquidation_data)
            .with_transfer_data(transfer_data);

        assert_eq!(event.transfer_data.len(), 2);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        let second_transfer = event
            .transfer_data
            .get(1)
            .expect("Second transfer should exist");
        assert_eq!(first_transfer.amount, 3_000_000);
        assert_eq!(second_transfer.amount, 2_700_000);
    }

    #[test]
    fn marginfi_event_parser_default() {
        let parser = MarginFiEventParser::default();

        // Test that parser contains expected program IDs
        let program_ids = parser.supported_program_ids();
        assert_eq!(program_ids.len(), 2);
        assert!(program_ids.contains(&program_id()));
        assert!(program_ids.contains(&bank_program_id()));

        // Test should_handle method
        assert!(parser.should_handle(&program_id()));
        assert!(parser.should_handle(&bank_program_id()));
        assert!(!parser.should_handle(&Pubkey::new_unique()));

        // Test configs are populated
        let inner_configs = parser.inner_instruction_configs();
        let instruction_configs = parser.instruction_configs();

        assert!(!inner_configs.is_empty());
        assert!(!instruction_configs.is_empty());

        // Test specific discriminators are present
        assert!(inner_configs.contains_key("lendingAccountDeposit"));
        assert!(inner_configs.contains_key("lendingAccountWithdraw"));
        assert!(inner_configs.contains_key("lendingAccountBorrow"));
        assert!(inner_configs.contains_key("lendingAccountRepay"));
        assert!(inner_configs.contains_key("lendingAccountLiquidate"));

        assert!(instruction_configs.contains_key(MARGINFI_DEPOSIT_DISCRIMINATOR.as_slice()));
        assert!(instruction_configs.contains_key(MARGINFI_WITHDRAW_DISCRIMINATOR.as_slice()));
        assert!(instruction_configs.contains_key(MARGINFI_BORROW_DISCRIMINATOR.as_slice()));
        assert!(instruction_configs.contains_key(MARGINFI_REPAY_DISCRIMINATOR.as_slice()));
        assert!(instruction_configs.contains_key(MARGINFI_LIQUIDATE_DISCRIMINATOR.as_slice()));
    }

    #[test]
    fn program_id_functions() {
        let main_program_id = program_id();
        let bank_program_id = bank_program_id();

        // Test that program IDs are different
        assert_ne!(main_program_id, bank_program_id);

        // Test is_marginfi_program function
        assert!(is_marginfi_program(&main_program_id));
        assert!(is_marginfi_program(&bank_program_id));
        assert!(!is_marginfi_program(&Pubkey::new_unique()));
    }

    #[test]
    fn event_trait_implementations() {
        // Test deposit event
        let mut deposit_event = MarginFiDepositEvent::default();
        let test_id = "test_deposit_id";
        let test_kind = EventKind::Transfer;

        // Note: EventMetadata doesn't have set_id/set_kind methods
        // These would need to be set during metadata creation

        // Create new metadata with proper values
        deposit_event.metadata.core = riglr_events_core::EventMetadata::new(
            test_id.to_owned(),
            test_kind.clone(),
            "marginfi-test".to_owned(),
        );

        assert_eq!(deposit_event.id(), test_id);
        assert_eq!(deposit_event.kind(), &test_kind);
        assert!(!deposit_event.metadata().id.is_empty());

        // Test clone_boxed
        let boxed_event = deposit_event.clone_boxed();
        assert_eq!(boxed_event.id(), test_id);

        // Test as_any
        let any_event = deposit_event.as_any();
        assert!(any_event.downcast_ref::<MarginFiDepositEvent>().is_some());

        // Test withdraw event
        let mut withdraw_event = MarginFiWithdrawEvent::default();
        withdraw_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_withdraw_id".to_owned(),
            EventKind::Transfer,
            "marginfi-test".to_owned(),
        );
        assert_eq!(withdraw_event.id(), "test_withdraw_id");

        // Test borrow event
        let mut borrow_event = MarginFiBorrowEvent::default();
        borrow_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_borrow_id".to_owned(),
            EventKind::Transfer,
            "marginfi-test".to_owned(),
        );
        assert_eq!(borrow_event.id(), "test_borrow_id");

        // Test repay event
        let mut repay_event = MarginFiRepayEvent::default();
        repay_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_repay_id".to_owned(),
            EventKind::Transfer,
            "marginfi-test".to_owned(),
        );
        assert_eq!(repay_event.id(), "test_repay_id");

        // Test liquidation event
        let mut liquidation_event = MarginFiLiquidationEvent::default();
        liquidation_event.metadata.core = riglr_events_core::EventMetadata::new(
            "test_liquidation_id".to_owned(),
            EventKind::Transfer,
            "marginfi-test".to_owned(),
        );
        assert_eq!(liquidation_event.id(), "test_liquidation_id");
    }

    #[test]
    fn event_serialization() {
        let params = EventParameters::new(
            "serialize_test".to_owned(),
            "sig_serialize".to_owned(),
            999,
            1_640_995_999,
            1_640_995_999_000,
            1_640_995_999_999,
            "serialize".to_owned(),
        );

        let deposit_data = MarginFiDepositData {
            amount: 1_000_000,
            ..Default::default()
        };

        let event = MarginFiDepositEvent::new(params, deposit_data);

        // Test to_json method
        let json_result = event.to_json();
        assert!(json_result.is_ok());

        let json_value = json_result.unwrap_or(serde_json::Value::Null);
        assert!(json_value.is_object());

        // Verify some key fields are present in JSON
        let obj = json_value.as_object().expect("JSON should be an object");
        assert!(obj.contains_key("deposit_data"));

        // Test that metadata field is skipped (not serialized)
        assert!(!obj.contains_key("metadata"));
    }

    #[test]
    fn discriminators_consistency() {
        // Test that discriminator constants are correct length
        assert_eq!(MARGINFI_DEPOSIT_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_WITHDRAW_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_BORROW_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_REPAY_DISCRIMINATOR.len(), 8);
        assert_eq!(MARGINFI_LIQUIDATE_DISCRIMINATOR.len(), 8);

        // Test that discriminators are unique
        let discriminators = [
            MARGINFI_DEPOSIT_DISCRIMINATOR,
            MARGINFI_WITHDRAW_DISCRIMINATOR,
            MARGINFI_BORROW_DISCRIMINATOR,
            MARGINFI_REPAY_DISCRIMINATOR,
            MARGINFI_LIQUIDATE_DISCRIMINATOR,
        ];

        for (i, disc1) in discriminators.iter().enumerate() {
            for (j, disc2) in discriminators.iter().enumerate() {
                if i != j {
                    assert_ne!(disc1, disc2, "Discriminators should be unique");
                }
            }
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn utility_functions_with_edge_cases() {
        // Test health ratio calculations
        {
            assert!(calculate_health_ratio(0, 0, 5_000).is_infinite());
            assert!(calculate_health_ratio(1_000, 0, 5_000).is_infinite());
        };

        let ratio = calculate_health_ratio(2_000, 1_000, 5_000);
        assert!((ratio - 1.0).abs() < f64::EPSILON);

        // Test liquidation threshold
        assert_eq!(calculate_liquidation_threshold(0, 8_000), Some(0));
        assert_eq!(calculate_liquidation_threshold(10_000, 8_000), Some(8_000));

        // Test shares conversion
        assert_eq!(shares_to_amount(0, 1000), Some(0));
        assert_eq!(amount_to_shares(0, 1000), Some(0));
        assert_eq!(amount_to_shares(100, 0), Some(100));

        // Test interest rate calculation
        let rate = calculate_interest_rate(5_000, 200, 800, 2_000, 8_000).unwrap();
        assert!(rate >= 200); // Should be at least base rate

        let rate_high = calculate_interest_rate(9_000, 200, 800, 2_000, 8_000).unwrap();
        assert!(rate_high > rate); // Higher utilization should mean higher rate
    }

    #[test]
    fn marginfi_account_type_enum() {
        // Test enum variants
        assert_eq!(
            MarginFiAccountType::MarginfiGroup,
            MarginFiAccountType::MarginfiGroup
        );
        assert_ne!(
            MarginFiAccountType::MarginfiGroup,
            MarginFiAccountType::MarginfiAccount
        );
        assert_ne!(MarginFiAccountType::Bank, MarginFiAccountType::Unknown);

        // Test serialization/deserialization
        let account_type = MarginFiAccountType::Bank;
        let serialized = serde_json::to_string(&account_type)
            .unwrap_or_else(|_| "serialization failed".to_owned());
        let deserialized: MarginFiAccountType =
            serde_json::from_str(&serialized).unwrap_or(MarginFiAccountType::Unknown);
        assert_eq!(account_type, deserialized);
    }

    #[test]
    fn complex_data_structures() {
        // Test MarginFiBankConfig with all fields
        let bank_config = MarginFiBankConfig {
            bank: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            bank_authority: Pubkey::new_unique(),
            collected_insurance_fees_outstanding: 1000,
            fee_rate: 100,
            insurance_fee_rate: 50,
            insurance_vault: Pubkey::new_unique(),
            deposit_limit: 1_000_000_000,
            borrow_limit: 800_000_000,
            operational_state: 1,
            oracle_setup: 2,
            oracle_keys: [Pubkey::new_unique(); 5],
        };

        assert_eq!(bank_config.fee_rate, 100);
        assert_eq!(bank_config.oracle_keys.len(), 5);

        // Test MarginFiBalance
        let balance = MarginFiBalance {
            active: true,
            bank_pk: Pubkey::new_unique(),
            asset_shares: 1_000_000,
            liability_shares: 500_000,
            emissions_outstanding: 100,
            last_update: 1_640_995_200,
            padding: [0; 1],
        };

        assert!(balance.active);
        assert_eq!(balance.asset_shares, 1_000_000);
        assert_eq!(balance.liability_shares, 500_000);
    }

    #[test]
    fn empty_transfer_data_operations() {
        let params = EventParameters::default();
        let deposit_data = MarginFiDepositData::default();

        // Test with empty transfer data
        let event = MarginFiDepositEvent::new(params, deposit_data).with_transfer_data(vec![]);

        assert!(event.transfer_data.is_empty());

        // Test multiple with_transfer_data calls (should replace, not append)
        let transfer1 = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 100,
            mint: Some(Pubkey::new_unique()),
        }];

        let transfer2 = vec![TransferData {
            source: Pubkey::new_unique(),
            destination: Pubkey::new_unique(),
            amount: 200,
            mint: Some(Pubkey::new_unique()),
        }];

        let params2 = EventParameters::default();
        let deposit_data2 = MarginFiDepositData::default();

        let event = MarginFiDepositEvent::new(params2, deposit_data2)
            .with_transfer_data(transfer1)
            .with_transfer_data(transfer2);

        assert_eq!(event.transfer_data.len(), 1);
        let first_transfer = event
            .transfer_data
            .first()
            .expect("Transfer data should exist");
        assert_eq!(first_transfer.amount, 200);
    }
}
