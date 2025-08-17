use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};

/// Direction of a trade operation in the BONK protocol
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum TradeDirection {
    /// Buy operation - purchasing tokens
    #[default]
    Buy,
    /// Sell operation - selling tokens
    Sell,
}

/// Current status of a liquidity pool in the BONK protocol
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub enum PoolStatus {
    /// Initial funding phase where liquidity is being raised
    #[default]
    Fund,
    /// Migration phase where pool is transitioning to DEX
    Migrate,
    /// Active trading phase where tokens can be traded
    Trade,
}

/// Parameters for minting new tokens in the BONK protocol
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct MintParams {
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Human-readable name of the token
    pub name: String,
    /// Trading symbol for the token
    pub symbol: String,
    /// URI pointing to token metadata
    pub uri: String,
}

/// Parameters for token vesting schedules
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct VestingParams {
    /// Total amount of tokens locked in vesting
    pub total_locked_amount: u64,
    /// Duration before any tokens can be unlocked
    pub cliff_period: u64,
    /// Period over which tokens are gradually unlocked
    pub unlock_period: u64,
}

/// Constant product bonding curve parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct ConstantCurve {
    /// Total token supply for the curve
    pub supply: u64,
    /// Total base tokens available for selling
    pub total_base_sell: u64,
    /// Total quote tokens raised during funding
    pub total_quote_fund_raising: u64,
    /// Type of migration when curve completes
    pub migrate_type: u8,
}

/// Fixed price bonding curve parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct FixedCurve {
    /// Total token supply for the curve
    pub supply: u64,
    /// Total quote tokens to be raised during funding
    pub total_quote_fund_raising: u64,
    /// Type of migration when curve completes
    pub migrate_type: u8,
}

/// Linear bonding curve parameters
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct LinearCurve {
    /// Total token supply for the curve
    pub supply: u64,
    /// Total quote tokens to be raised during funding
    pub total_quote_fund_raising: u64,
    /// Type of migration when curve completes
    pub migrate_type: u8,
}

/// Container for different bonding curve configurations
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct CurveParams {
    /// Constant product curve configuration, if used
    pub constant_curve: Option<ConstantCurve>,
    /// Fixed price curve configuration, if used
    pub fixed_curve: Option<FixedCurve>,
    /// Linear price curve configuration, if used
    pub linear_curve: Option<LinearCurve>,
}
