use crate::solana_metadata::SolanaEventMetadata;
use riglr_events_core::EventMetadata as CoreEventMetadata;
use riglr_events_core::{Event, EventKind};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::any::Any;

/// Raydium AMM V4 swap event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4SwapEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Amount of tokens going into the swap
    pub amount_in: u64,
    /// Amount of tokens coming out of the swap
    pub amount_out: u64,
    /// Direction of the swap (BaseIn or BaseOut)
    pub direction: SwapDirection,

    // Account keys
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC (price currency) token account
    pub pool_pc_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

/// Swap event data for Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumAmmV4SwapData {
    /// Amount of tokens going into the swap
    pub amount_in: u64,
    /// Amount of tokens coming out of the swap
    pub amount_out: u64,
    /// Direction of the swap (BaseIn or BaseOut)
    pub direction: SwapDirection,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC (price currency) token account
    pub pool_pc_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

impl RaydiumAmmV4SwapEvent {
    /// Creates a new RaydiumAmmV4SwapEvent with the provided metadata and swap data
    pub fn new(metadata: SolanaEventMetadata, swap_data: RaydiumAmmV4SwapData) -> Self {
        Self {
            metadata,
            amount_in: swap_data.amount_in,
            amount_out: swap_data.amount_out,
            direction: swap_data.direction,
            amm: swap_data.amm,
            amm_authority: swap_data.amm_authority,
            amm_open_orders: swap_data.amm_open_orders,
            pool_coin_token_account: swap_data.pool_coin_token_account,
            pool_pc_token_account: swap_data.pool_pc_token_account,
            serum_program: swap_data.serum_program,
            serum_market: swap_data.serum_market,
            user_coin_token_account: swap_data.user_coin_token_account,
            user_pc_token_account: swap_data.user_pc_token_account,
            user_owner: swap_data.user_owner,
        }
    }
}

/// Direction of the swap operation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum SwapDirection {
    /// Swapping base token in for quote token out
    #[default]
    BaseIn,
    /// Swapping quote token in for base token out
    BaseOut,
}

// EventParameters is now imported from crate::events::core

/// Raydium AMM V4 deposit event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4DepositEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Maximum amount of coin tokens to deposit
    pub max_coin_amount: u64,
    /// Maximum amount of PC tokens to deposit
    pub max_pc_amount: u64,
    /// Base side identifier
    pub base_side: u64,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

/// Deposit event data for Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumAmmV4DepositData {
    /// Maximum amount of coin tokens to deposit
    pub max_coin_amount: u64,
    /// Maximum amount of PC tokens to deposit
    pub max_pc_amount: u64,
    /// Base side identifier
    pub base_side: u64,
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
}

impl RaydiumAmmV4DepositEvent {
    /// Creates a new RaydiumAmmV4DepositEvent with the provided metadata and deposit data
    pub fn new(metadata: SolanaEventMetadata, deposit_data: RaydiumAmmV4DepositData) -> Self {
        Self {
            metadata,
            max_coin_amount: deposit_data.max_coin_amount,
            max_pc_amount: deposit_data.max_pc_amount,
            base_side: deposit_data.base_side,
            token_program: deposit_data.token_program,
            amm: deposit_data.amm,
            amm_authority: deposit_data.amm_authority,
            amm_open_orders: deposit_data.amm_open_orders,
            amm_target_orders: deposit_data.amm_target_orders,
            lp_mint_address: deposit_data.lp_mint_address,
            pool_coin_token_account: deposit_data.pool_coin_token_account,
            pool_pc_token_account: deposit_data.pool_pc_token_account,
            serum_market: deposit_data.serum_market,
            user_coin_token_account: deposit_data.user_coin_token_account,
            user_pc_token_account: deposit_data.user_pc_token_account,
            user_lp_token_account: deposit_data.user_lp_token_account,
            user_owner: deposit_data.user_owner,
        }
    }
}

/// Raydium AMM V4 initialize2 event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4Initialize2Event {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Nonce value for initialization
    pub nonce: u8,
    /// Time when the pool opens
    pub open_time: u64,
    /// Initial PC token amount
    pub init_pc_amount: u64,
    /// Initial coin token amount
    pub init_coin_amount: u64,

    // Account keys
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Coin mint address
    pub coin_mint_address: Pubkey,
    /// PC mint address
    pub pc_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Pool liquidity provider token account
    pub pool_lp_token_account: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User wallet account
    pub user_wallet: Pubkey,
}

/// Initialize2 event data for Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumAmmV4Initialize2Data {
    /// Nonce value for initialization
    pub nonce: u8,
    /// Time when the pool opens
    pub open_time: u64,
    /// Initial PC token amount
    pub init_pc_amount: u64,
    /// Initial coin token amount
    pub init_coin_amount: u64,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Coin mint address
    pub coin_mint_address: Pubkey,
    /// PC mint address
    pub pc_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Pool liquidity provider token account
    pub pool_lp_token_account: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// User wallet account
    pub user_wallet: Pubkey,
}

impl RaydiumAmmV4Initialize2Event {
    /// Creates a new RaydiumAmmV4Initialize2Event with the provided metadata and initialization data
    pub fn new(metadata: SolanaEventMetadata, init_data: RaydiumAmmV4Initialize2Data) -> Self {
        Self {
            metadata,
            nonce: init_data.nonce,
            open_time: init_data.open_time,
            init_pc_amount: init_data.init_pc_amount,
            init_coin_amount: init_data.init_coin_amount,
            amm: init_data.amm,
            amm_authority: init_data.amm_authority,
            amm_open_orders: init_data.amm_open_orders,
            lp_mint_address: init_data.lp_mint_address,
            coin_mint_address: init_data.coin_mint_address,
            pc_mint_address: init_data.pc_mint_address,
            pool_coin_token_account: init_data.pool_coin_token_account,
            pool_pc_token_account: init_data.pool_pc_token_account,
            pool_withdraw_queue: init_data.pool_withdraw_queue,
            amm_target_orders: init_data.amm_target_orders,
            pool_lp_token_account: init_data.pool_lp_token_account,
            pool_temp_lp_token_account: init_data.pool_temp_lp_token_account,
            serum_program: init_data.serum_program,
            serum_market: init_data.serum_market,
            user_wallet: init_data.user_wallet,
        }
    }
}

/// Raydium AMM V4 withdraw event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// Amount of tokens being withdrawn
    pub amount: u64,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum bids account
    pub serum_bids: Pubkey,
    /// Serum asks account
    pub serum_asks: Pubkey,
}

/// Withdraw event data for Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumAmmV4WithdrawData {
    /// Amount of tokens being withdrawn
    pub amount: u64,
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Liquidity provider mint address
    pub lp_mint_address: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Pool withdrawal queue account
    pub pool_withdraw_queue: Pubkey,
    /// Pool temporary LP token account
    pub pool_temp_lp_token_account: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
    /// User liquidity provider token account
    pub user_lp_token_account: Pubkey,
    /// User coin token account
    pub user_coin_token_account: Pubkey,
    /// User PC token account
    pub user_pc_token_account: Pubkey,
    /// User owner account
    pub user_owner: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum bids account
    pub serum_bids: Pubkey,
    /// Serum asks account
    pub serum_asks: Pubkey,
}

impl RaydiumAmmV4WithdrawEvent {
    /// Creates a new RaydiumAmmV4WithdrawEvent with the provided metadata and withdrawal data
    pub fn new(metadata: SolanaEventMetadata, withdraw_data: RaydiumAmmV4WithdrawData) -> Self {
        Self {
            metadata,
            amount: withdraw_data.amount,
            token_program: withdraw_data.token_program,
            amm: withdraw_data.amm,
            amm_authority: withdraw_data.amm_authority,
            amm_open_orders: withdraw_data.amm_open_orders,
            amm_target_orders: withdraw_data.amm_target_orders,
            lp_mint_address: withdraw_data.lp_mint_address,
            pool_coin_token_account: withdraw_data.pool_coin_token_account,
            pool_pc_token_account: withdraw_data.pool_pc_token_account,
            pool_withdraw_queue: withdraw_data.pool_withdraw_queue,
            pool_temp_lp_token_account: withdraw_data.pool_temp_lp_token_account,
            serum_program: withdraw_data.serum_program,
            serum_market: withdraw_data.serum_market,
            serum_coin_vault_account: withdraw_data.serum_coin_vault_account,
            serum_pc_vault_account: withdraw_data.serum_pc_vault_account,
            serum_vault_signer: withdraw_data.serum_vault_signer,
            user_lp_token_account: withdraw_data.user_lp_token_account,
            user_coin_token_account: withdraw_data.user_coin_token_account,
            user_pc_token_account: withdraw_data.user_pc_token_account,
            user_owner: withdraw_data.user_owner,
            serum_event_queue: withdraw_data.serum_event_queue,
            serum_bids: withdraw_data.serum_bids,
            serum_asks: withdraw_data.serum_asks,
        }
    }
}

/// Raydium AMM V4 withdraw PNL event
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RaydiumAmmV4WithdrawPnlEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,

    // Account keys
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM configuration account
    pub amm_config: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Coin PNL (Profit and Loss) token account
    pub coin_pnl_token_account: Pubkey,
    /// PC PNL token account
    pub pc_pnl_token_account: Pubkey,
    /// PNL owner account
    pub pnl_owner_account: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
}

/// WithdrawPnl event data for Raydium AMM V4
#[derive(Debug, Clone)]
pub struct RaydiumAmmV4WithdrawPnlData {
    /// Token program account
    pub token_program: Pubkey,
    /// Automated Market Maker account
    pub amm: Pubkey,
    /// AMM configuration account
    pub amm_config: Pubkey,
    /// AMM authority account
    pub amm_authority: Pubkey,
    /// AMM open orders account
    pub amm_open_orders: Pubkey,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool PC token account
    pub pool_pc_token_account: Pubkey,
    /// Coin PNL (Profit and Loss) token account
    pub coin_pnl_token_account: Pubkey,
    /// PC PNL token account
    pub pc_pnl_token_account: Pubkey,
    /// PNL owner account
    pub pnl_owner_account: Pubkey,
    /// AMM target orders account
    pub amm_target_orders: Pubkey,
    /// Serum program account
    pub serum_program: Pubkey,
    /// Serum market account
    pub serum_market: Pubkey,
    /// Serum event queue account
    pub serum_event_queue: Pubkey,
    /// Serum coin vault account
    pub serum_coin_vault_account: Pubkey,
    /// Serum PC vault account
    pub serum_pc_vault_account: Pubkey,
    /// Serum vault signer account
    pub serum_vault_signer: Pubkey,
}

impl RaydiumAmmV4WithdrawPnlEvent {
    /// Creates a new RaydiumAmmV4WithdrawPnlEvent with the provided metadata and PNL data
    pub fn new(metadata: SolanaEventMetadata, pnl_data: RaydiumAmmV4WithdrawPnlData) -> Self {
        Self {
            metadata,
            token_program: pnl_data.token_program,
            amm: pnl_data.amm,
            amm_config: pnl_data.amm_config,
            amm_authority: pnl_data.amm_authority,
            amm_open_orders: pnl_data.amm_open_orders,
            pool_coin_token_account: pnl_data.pool_coin_token_account,
            pool_pc_token_account: pnl_data.pool_pc_token_account,
            coin_pnl_token_account: pnl_data.coin_pnl_token_account,
            pc_pnl_token_account: pnl_data.pc_pnl_token_account,
            pnl_owner_account: pnl_data.pnl_owner_account,
            amm_target_orders: pnl_data.amm_target_orders,
            serum_program: pnl_data.serum_program,
            serum_market: pnl_data.serum_market,
            serum_event_queue: pnl_data.serum_event_queue,
            serum_coin_vault_account: pnl_data.serum_coin_vault_account,
            serum_pc_vault_account: pnl_data.serum_pc_vault_account,
            serum_vault_signer: pnl_data.serum_vault_signer,
        }
    }
}

// Event trait implementations
// Event trait implementations

impl Event for RaydiumAmmV4SwapEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static SWAP_KIND: EventKind = EventKind::Swap;
        &SWAP_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4DepositEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4Initialize2Event {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static CONTRACT_KIND: EventKind = EventKind::Contract;
        &CONTRACT_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4WithdrawEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl Event for RaydiumAmmV4WithdrawPnlEvent {
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut CoreEventMetadata> {
        Ok(&mut self.metadata.core)
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;
    use riglr_events_core::{Event, EventKind};
    use solana_sdk::pubkey::Pubkey;

    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    fn create_test_metadata() -> SolanaEventMetadata {
        let core = riglr_events_core::EventMetadata::new(
            "test_id".to_string(),
            EventKind::Swap,
            "test_protocol".to_string(),
        );

        SolanaEventMetadata::new(
            "test-signature".to_string(),
            12345,
            crate::types::EventType::Swap,
            crate::types::ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            1640995200000,
            core,
        )
    }

    // SwapDirection tests
    #[test]
    fn test_swap_direction_default_should_be_base_in() {
        let direction = SwapDirection::default();
        matches!(direction, SwapDirection::BaseIn);
    }

    #[test]
    fn test_swap_direction_clone_should_work() {
        let direction = SwapDirection::BaseOut;
        let cloned = direction.clone();
        matches!(cloned, SwapDirection::BaseOut);
    }

    #[test]
    fn test_swap_direction_debug_should_work() {
        let direction = SwapDirection::BaseIn;
        let debug_str = format!("{:?}", direction);
        assert!(debug_str.contains("BaseIn"));
    }

    #[test]
    fn test_swap_direction_serialize_deserialize_should_work() {
        let direction = SwapDirection::BaseOut;
        let serialized = serde_json::to_string(&direction).unwrap();
        let deserialized: SwapDirection = serde_json::from_str(&serialized).unwrap();
        matches!(deserialized, SwapDirection::BaseOut);
    }

    // RaydiumAmmV4SwapEvent tests
    #[test]
    fn test_raydium_amm_v4_swap_event_default_should_work() {
        let event = RaydiumAmmV4SwapEvent::default();
        assert_eq!(event.amount_in, 0);
        assert_eq!(event.amount_out, 0);
        matches!(event.direction, SwapDirection::BaseIn);
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_clone_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.amount_in = 100;
        event.amount_out = 200;
        let cloned = event.clone();
        assert_eq!(cloned.amount_in, 100);
        assert_eq!(cloned.amount_out, 200);
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_debug_should_work() {
        let event = RaydiumAmmV4SwapEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumAmmV4SwapEvent"));
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_serialize_deserialize_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.amount_in = 100;
        event.amount_out = 200;
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumAmmV4SwapEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.amount_in, 100);
        assert_eq!(deserialized.amount_out, 200);
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_id_should_return_metadata_id() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.metadata = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_kind_should_return_swap() {
        let event = RaydiumAmmV4SwapEvent::default();
        assert!(matches!(event.kind(), &EventKind::Swap));
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_metadata_should_return_cached_metadata() {
        let event = RaydiumAmmV4SwapEvent::default();
        let metadata = event.metadata();
        assert_eq!(metadata.source, "raydium-amm-v4");
        assert!(matches!(&metadata.kind, &EventKind::Swap));
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_metadata_mut_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-swap-id".to_string();
        assert_eq!(event.metadata().id, "test-swap-id");
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_as_any_should_work() {
        let event = RaydiumAmmV4SwapEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<RaydiumAmmV4SwapEvent>().is_some());
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_as_any_mut_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<RaydiumAmmV4SwapEvent>().is_some());
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_clone_boxed_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.amount_in = 100;
        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<RaydiumAmmV4SwapEvent>()
            .unwrap();
        assert_eq!(downcast.amount_in, 100);
    }

    #[test]
    fn test_raydium_amm_v4_swap_event_to_json_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.amount_in = 100;
        let json = event.to_json().unwrap();
        assert!(json.get("amount_in").is_some());
    }

    // RaydiumAmmV4DepositEvent tests
    #[test]
    fn test_raydium_amm_v4_deposit_event_default_should_work() {
        let event = RaydiumAmmV4DepositEvent::default();
        assert_eq!(event.max_coin_amount, 0);
        assert_eq!(event.max_pc_amount, 0);
        assert_eq!(event.base_side, 0);
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_clone_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.max_coin_amount = 100;
        event.max_pc_amount = 200;
        let cloned = event.clone();
        assert_eq!(cloned.max_coin_amount, 100);
        assert_eq!(cloned.max_pc_amount, 200);
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_debug_should_work() {
        let event = RaydiumAmmV4DepositEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumAmmV4DepositEvent"));
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_serialize_deserialize_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.max_coin_amount = 100;
        event.max_pc_amount = 200;
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumAmmV4DepositEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.max_coin_amount, 100);
        assert_eq!(deserialized.max_pc_amount, 200);
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_id_should_return_metadata_id() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.metadata = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_kind_should_return_liquidity() {
        let event = RaydiumAmmV4DepositEvent::default();
        assert!(matches!(event.kind(), &EventKind::Liquidity));
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_metadata_should_return_cached_metadata() {
        let event = RaydiumAmmV4DepositEvent::default();
        let metadata = event.metadata();
        assert_eq!(metadata.source, "raydium-amm-v4");
        assert!(matches!(&metadata.kind, &EventKind::Liquidity));
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_metadata_mut_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-deposit-id".to_string();
        assert_eq!(event.metadata().id, "test-deposit-id");
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_as_any_should_work() {
        let event = RaydiumAmmV4DepositEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<RaydiumAmmV4DepositEvent>().is_some());
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_as_any_mut_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<RaydiumAmmV4DepositEvent>().is_some());
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_clone_boxed_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.max_coin_amount = 100;
        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<RaydiumAmmV4DepositEvent>()
            .unwrap();
        assert_eq!(downcast.max_coin_amount, 100);
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_to_json_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.max_coin_amount = 100;
        let json = event.to_json().unwrap();
        assert!(json.get("max_coin_amount").is_some());
    }

    // RaydiumAmmV4Initialize2Event tests
    #[test]
    fn test_raydium_amm_v4_initialize2_event_default_should_work() {
        let event = RaydiumAmmV4Initialize2Event::default();
        assert_eq!(event.nonce, 0);
        assert_eq!(event.open_time, 0);
        assert_eq!(event.init_pc_amount, 0);
        assert_eq!(event.init_coin_amount, 0);
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_clone_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.nonce = 5;
        event.open_time = 1000;
        let cloned = event.clone();
        assert_eq!(cloned.nonce, 5);
        assert_eq!(cloned.open_time, 1000);
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_debug_should_work() {
        let event = RaydiumAmmV4Initialize2Event::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumAmmV4Initialize2Event"));
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_serialize_deserialize_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.nonce = 5;
        event.open_time = 1000;
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumAmmV4Initialize2Event = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.nonce, 5);
        assert_eq!(deserialized.open_time, 1000);
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_id_should_return_metadata_id() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.metadata = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_kind_should_return_contract() {
        let event = RaydiumAmmV4Initialize2Event::default();
        assert!(matches!(event.kind(), &EventKind::Contract));
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_metadata_should_return_cached_metadata() {
        let event = RaydiumAmmV4Initialize2Event::default();
        let metadata = event.metadata();
        assert_eq!(metadata.source, "raydium-amm-v4");
        assert!(matches!(&metadata.kind, &EventKind::Contract));
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_metadata_mut_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-initialize2-id".to_string();
        assert_eq!(event.metadata().id, "test-initialize2-id");
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_as_any_should_work() {
        let event = RaydiumAmmV4Initialize2Event::default();
        let any_ref = event.as_any();
        assert!(any_ref
            .downcast_ref::<RaydiumAmmV4Initialize2Event>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_as_any_mut_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut
            .downcast_mut::<RaydiumAmmV4Initialize2Event>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_clone_boxed_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.nonce = 5;
        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<RaydiumAmmV4Initialize2Event>()
            .unwrap();
        assert_eq!(downcast.nonce, 5);
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_to_json_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.nonce = 5;
        let json = event.to_json().unwrap();
        assert!(json.get("nonce").is_some());
    }

    // RaydiumAmmV4WithdrawEvent tests
    #[test]
    fn test_raydium_amm_v4_withdraw_event_default_should_work() {
        let event = RaydiumAmmV4WithdrawEvent::default();
        assert_eq!(event.amount, 0);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_clone_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.amount = 500;
        let cloned = event.clone();
        assert_eq!(cloned.amount, 500);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_debug_should_work() {
        let event = RaydiumAmmV4WithdrawEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumAmmV4WithdrawEvent"));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_serialize_deserialize_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.amount = 500;
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumAmmV4WithdrawEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.amount, 500);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_id_should_return_metadata_id() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.metadata = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_kind_should_return_liquidity() {
        let event = RaydiumAmmV4WithdrawEvent::default();
        assert!(matches!(event.kind(), &EventKind::Liquidity));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_metadata_should_return_cached_metadata() {
        let event = RaydiumAmmV4WithdrawEvent::default();
        let metadata = event.metadata();
        assert_eq!(metadata.source, "raydium-amm-v4");
        assert!(matches!(&metadata.kind, &EventKind::Liquidity));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_metadata_mut_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-withdraw-id".to_string();
        assert_eq!(event.metadata().id, "test-withdraw-id");
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_as_any_should_work() {
        let event = RaydiumAmmV4WithdrawEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref
            .downcast_ref::<RaydiumAmmV4WithdrawEvent>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_as_any_mut_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut
            .downcast_mut::<RaydiumAmmV4WithdrawEvent>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_clone_boxed_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.amount = 500;
        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawEvent>()
            .unwrap();
        assert_eq!(downcast.amount, 500);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_to_json_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.amount = 500;
        let json = event.to_json().unwrap();
        assert!(json.get("amount").is_some());
    }

    // RaydiumAmmV4WithdrawPnlEvent tests
    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_default_should_work() {
        let event = RaydiumAmmV4WithdrawPnlEvent::default();
        // All fields are Pubkey or EventMetadata, so just verify structure
        assert_eq!(
            format!("{:?}", event.metadata),
            format!("{:?}", SolanaEventMetadata::default())
        );
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_clone_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        event.amm = create_test_pubkey();
        let cloned = event.clone();
        assert_eq!(cloned.amm, event.amm);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_debug_should_work() {
        let event = RaydiumAmmV4WithdrawPnlEvent::default();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("RaydiumAmmV4WithdrawPnlEvent"));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_serialize_deserialize_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        event.amm = create_test_pubkey();
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: RaydiumAmmV4WithdrawPnlEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.amm, event.amm);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_id_should_return_metadata_id() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        event.metadata = create_test_metadata();
        assert_eq!(event.id(), "test_id");
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_kind_should_return_transfer() {
        let event = RaydiumAmmV4WithdrawPnlEvent::default();
        assert!(matches!(event.kind(), &EventKind::Transfer));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_metadata_should_return_cached_metadata() {
        let event = RaydiumAmmV4WithdrawPnlEvent::default();
        let metadata = event.metadata();
        assert_eq!(metadata.source, "raydium-amm-v4");
        assert!(matches!(&metadata.kind, &EventKind::Transfer));
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_metadata_mut_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        let metadata = event.metadata_mut().unwrap();
        metadata.id = "test-withdraw-pnl-id".to_string();
        assert_eq!(event.metadata().id, "test-withdraw-pnl-id");
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_as_any_should_work() {
        let event = RaydiumAmmV4WithdrawPnlEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref
            .downcast_ref::<RaydiumAmmV4WithdrawPnlEvent>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_as_any_mut_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut
            .downcast_mut::<RaydiumAmmV4WithdrawPnlEvent>()
            .is_some());
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_clone_boxed_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        event.amm = create_test_pubkey();
        let boxed = event.clone_boxed();
        let downcast = boxed
            .as_any()
            .downcast_ref::<RaydiumAmmV4WithdrawPnlEvent>()
            .unwrap();
        assert_eq!(downcast.amm, event.amm);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_pnl_event_to_json_should_work() {
        let mut event = RaydiumAmmV4WithdrawPnlEvent::default();
        event.amm = create_test_pubkey();
        let json = event.to_json().unwrap();
        assert!(json.get("amm").is_some());
    }

    // Edge case tests for maximum values
    #[test]
    fn test_raydium_amm_v4_swap_event_with_max_values_should_work() {
        let mut event = RaydiumAmmV4SwapEvent::default();
        event.amount_in = u64::MAX;
        event.amount_out = u64::MAX;
        assert_eq!(event.amount_in, u64::MAX);
        assert_eq!(event.amount_out, u64::MAX);
    }

    #[test]
    fn test_raydium_amm_v4_deposit_event_with_max_values_should_work() {
        let mut event = RaydiumAmmV4DepositEvent::default();
        event.max_coin_amount = u64::MAX;
        event.max_pc_amount = u64::MAX;
        event.base_side = u64::MAX;
        assert_eq!(event.max_coin_amount, u64::MAX);
        assert_eq!(event.max_pc_amount, u64::MAX);
        assert_eq!(event.base_side, u64::MAX);
    }

    #[test]
    fn test_raydium_amm_v4_initialize2_event_with_max_values_should_work() {
        let mut event = RaydiumAmmV4Initialize2Event::default();
        event.nonce = u8::MAX;
        event.open_time = u64::MAX;
        event.init_pc_amount = u64::MAX;
        event.init_coin_amount = u64::MAX;
        assert_eq!(event.nonce, u8::MAX);
        assert_eq!(event.open_time, u64::MAX);
        assert_eq!(event.init_pc_amount, u64::MAX);
        assert_eq!(event.init_coin_amount, u64::MAX);
    }

    #[test]
    fn test_raydium_amm_v4_withdraw_event_with_max_values_should_work() {
        let mut event = RaydiumAmmV4WithdrawEvent::default();
        event.amount = u64::MAX;
        assert_eq!(event.amount, u64::MAX);
    }

    // Test all Pubkey fields are properly initialized
    #[test]
    fn test_all_pubkey_fields_are_default_initialized() {
        let swap_event = RaydiumAmmV4SwapEvent::default();
        assert_eq!(swap_event.amm, Pubkey::default());
        assert_eq!(swap_event.amm_authority, Pubkey::default());
        assert_eq!(swap_event.user_owner, Pubkey::default());

        let deposit_event = RaydiumAmmV4DepositEvent::default();
        assert_eq!(deposit_event.token_program, Pubkey::default());
        assert_eq!(deposit_event.lp_mint_address, Pubkey::default());

        let init_event = RaydiumAmmV4Initialize2Event::default();
        assert_eq!(init_event.coin_mint_address, Pubkey::default());
        assert_eq!(init_event.pc_mint_address, Pubkey::default());

        let withdraw_event = RaydiumAmmV4WithdrawEvent::default();
        assert_eq!(withdraw_event.serum_bids, Pubkey::default());
        assert_eq!(withdraw_event.serum_asks, Pubkey::default());

        let pnl_event = RaydiumAmmV4WithdrawPnlEvent::default();
        assert_eq!(pnl_event.coin_pnl_token_account, Pubkey::default());
        assert_eq!(pnl_event.pc_pnl_token_account, Pubkey::default());
    }
}
#[cfg(test)]
mod metadata_fix_verification {
    use super::*;
    use riglr_events_core::{Event, EventKind};

    #[test]
    fn test_metadata_isolation_between_events() {
        // Create two events with different metadata
        let mut event1 = RaydiumAmmV4SwapEvent::default();
        let mut event2 = RaydiumAmmV4SwapEvent::default();

        // Set different IDs
        event1.metadata.id = "event1".to_string();
        event2.metadata.id = "event2".to_string();

        // Verify they have different IDs (this would fail with static shared metadata)
        assert_eq!(event1.id(), "event1");
        assert_eq!(event2.id(), "event2");

        // Modify one event's metadata
        event1.metadata.id = "modified".to_string();

        // Verify the other event is unchanged (this would fail with static shared metadata)
        assert_eq!(event1.id(), "modified");
        assert_eq!(event2.id(), "event2"); // This was failing before our fix!
    }

    #[test]
    fn test_metadata_mut_works() {
        let mut event = RaydiumAmmV4SwapEvent::default();

        // This should no longer panic
        let metadata_mut = event.metadata_mut().unwrap();
        metadata_mut.id = "test".to_string();

        assert_eq!(event.id(), "test");
    }

    #[test]
    fn test_kind_uses_instance_metadata() {
        let mut event = RaydiumAmmV4SwapEvent::default();

        // Set specific kind in instance metadata
        event.metadata.kind = EventKind::Transfer;

        // Should return the instance metadata kind, not a static one
        assert_eq!(event.kind(), &EventKind::Transfer);
    }
}
