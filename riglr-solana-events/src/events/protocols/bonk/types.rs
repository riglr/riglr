use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Direction of a trade operation in the BONK protocol
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
)]
pub enum TradeDirection {
    /// Buy operation - purchasing tokens
    #[default]
    Buy,
    /// Sell operation - selling tokens
    Sell,
}

/// Current status of a liquidity pool in the BONK protocol
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshDeserialize,
    BorshSerialize,
)]
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
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
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
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct VestingParams {
    /// Total amount of tokens locked in vesting
    pub total_locked_amount: u64,
    /// Duration before any tokens can be unlocked
    pub cliff_period: u64,
    /// Period over which tokens are gradually unlocked
    pub unlock_period: u64,
}

/// Constant product bonding curve parameters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
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
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct FixedCurve {
    /// Total token supply for the curve
    pub supply: u64,
    /// Total quote tokens to be raised during funding
    pub total_quote_fund_raising: u64,
    /// Type of migration when curve completes
    pub migrate_type: u8,
}

/// Linear bonding curve parameters
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct LinearCurve {
    /// Total token supply for the curve
    pub supply: u64,
    /// Total quote tokens to be raised during funding
    pub total_quote_fund_raising: u64,
    /// Type of migration when curve completes
    pub migrate_type: u8,
}

/// Container for different bonding curve configurations
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize,
)]
pub struct CurveParams {
    /// Constant product curve configuration, if used
    pub constant_curve: Option<ConstantCurve>,
    /// Fixed price curve configuration, if used
    pub fixed_curve: Option<FixedCurve>,
    /// Linear price curve configuration, if used
    pub linear_curve: Option<LinearCurve>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // TradeDirection Tests
    #[test]
    fn test_trade_direction_default_should_be_buy() {
        assert_eq!(TradeDirection::default(), TradeDirection::Buy);
    }

    #[test]
    fn test_trade_direction_clone_should_work() {
        let direction = TradeDirection::Sell;
        let cloned = direction.clone();
        assert_eq!(direction, cloned);
    }

    #[test]
    fn test_trade_direction_debug_should_work() {
        let buy_debug = format!("{:?}", TradeDirection::Buy);
        let sell_debug = format!("{:?}", TradeDirection::Sell);
        assert_eq!(buy_debug, "Buy");
        assert_eq!(sell_debug, "Sell");
    }

    #[test]
    fn test_trade_direction_partial_eq_should_work() {
        assert_eq!(TradeDirection::Buy, TradeDirection::Buy);
        assert_eq!(TradeDirection::Sell, TradeDirection::Sell);
        assert_ne!(TradeDirection::Buy, TradeDirection::Sell);
    }

    #[test]
    fn test_trade_direction_copy_should_work() {
        let direction = TradeDirection::Buy;
        let copied = direction;
        assert_eq!(direction, copied);
    }

    #[test]
    fn test_trade_direction_serialize_deserialize_should_work() {
        let buy = TradeDirection::Buy;
        let sell = TradeDirection::Sell;

        let buy_json = serde_json::to_string(&buy).unwrap();
        let sell_json = serde_json::to_string(&sell).unwrap();

        let buy_deserialized: TradeDirection = serde_json::from_str(&buy_json).unwrap();
        let sell_deserialized: TradeDirection = serde_json::from_str(&sell_json).unwrap();

        assert_eq!(buy, buy_deserialized);
        assert_eq!(sell, sell_deserialized);
    }

    // PoolStatus Tests
    #[test]
    fn test_pool_status_default_should_be_fund() {
        assert_eq!(PoolStatus::default(), PoolStatus::Fund);
    }

    #[test]
    fn test_pool_status_clone_should_work() {
        let status = PoolStatus::Trade;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_pool_status_debug_should_work() {
        let fund_debug = format!("{:?}", PoolStatus::Fund);
        let migrate_debug = format!("{:?}", PoolStatus::Migrate);
        let trade_debug = format!("{:?}", PoolStatus::Trade);
        assert_eq!(fund_debug, "Fund");
        assert_eq!(migrate_debug, "Migrate");
        assert_eq!(trade_debug, "Trade");
    }

    #[test]
    fn test_pool_status_partial_eq_should_work() {
        assert_eq!(PoolStatus::Fund, PoolStatus::Fund);
        assert_eq!(PoolStatus::Migrate, PoolStatus::Migrate);
        assert_eq!(PoolStatus::Trade, PoolStatus::Trade);
        assert_ne!(PoolStatus::Fund, PoolStatus::Migrate);
        assert_ne!(PoolStatus::Fund, PoolStatus::Trade);
        assert_ne!(PoolStatus::Migrate, PoolStatus::Trade);
    }

    #[test]
    fn test_pool_status_copy_should_work() {
        let status = PoolStatus::Migrate;
        let copied = status;
        assert_eq!(status, copied);
    }

    #[test]
    fn test_pool_status_serialize_deserialize_should_work() {
        let fund = PoolStatus::Fund;
        let migrate = PoolStatus::Migrate;
        let trade = PoolStatus::Trade;

        let fund_json = serde_json::to_string(&fund).unwrap();
        let migrate_json = serde_json::to_string(&migrate).unwrap();
        let trade_json = serde_json::to_string(&trade).unwrap();

        let fund_deserialized: PoolStatus = serde_json::from_str(&fund_json).unwrap();
        let migrate_deserialized: PoolStatus = serde_json::from_str(&migrate_json).unwrap();
        let trade_deserialized: PoolStatus = serde_json::from_str(&trade_json).unwrap();

        assert_eq!(fund, fund_deserialized);
        assert_eq!(migrate, migrate_deserialized);
        assert_eq!(trade, trade_deserialized);
    }

    // MintParams Tests
    #[test]
    fn test_mint_params_default_should_have_empty_fields() {
        let params = MintParams::default();
        assert_eq!(params.decimals, 0);
        assert_eq!(params.name, "");
        assert_eq!(params.symbol, "");
        assert_eq!(params.uri, "");
    }

    #[test]
    fn test_mint_params_clone_should_work() {
        let params = MintParams {
            decimals: 9,
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            uri: "https://example.com/metadata".to_string(),
        };
        let cloned = params.clone();
        assert_eq!(params, cloned);
    }

    #[test]
    fn test_mint_params_debug_should_work() {
        let params = MintParams {
            decimals: 6,
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            uri: "https://usdc.com/metadata".to_string(),
        };
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("decimals: 6"));
        assert!(debug_str.contains("USDC"));
    }

    #[test]
    fn test_mint_params_partial_eq_should_work() {
        let params1 = MintParams {
            decimals: 9,
            name: "Solana".to_string(),
            symbol: "SOL".to_string(),
            uri: "https://solana.com".to_string(),
        };
        let params2 = MintParams {
            decimals: 9,
            name: "Solana".to_string(),
            symbol: "SOL".to_string(),
            uri: "https://solana.com".to_string(),
        };
        let params3 = MintParams {
            decimals: 6,
            name: "Solana".to_string(),
            symbol: "SOL".to_string(),
            uri: "https://solana.com".to_string(),
        };

        assert_eq!(params1, params2);
        assert_ne!(params1, params3);
    }

    #[test]
    fn test_mint_params_serialize_deserialize_should_work() {
        let params = MintParams {
            decimals: 18,
            name: "Ethereum".to_string(),
            symbol: "ETH".to_string(),
            uri: "https://ethereum.org/metadata".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: MintParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params, deserialized);
    }

    #[test]
    fn test_mint_params_with_edge_cases_should_work() {
        let params = MintParams {
            decimals: u8::MAX,
            name: "".to_string(),
            symbol: "🚀".to_string(),
            uri: "data:text/plain;base64,SGVsbG8gV29ybGQ=".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: MintParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params, deserialized);
        assert_eq!(params.decimals, 255);
        assert_eq!(params.name, "");
        assert_eq!(params.symbol, "🚀");
    }

    // VestingParams Tests
    #[test]
    fn test_vesting_params_default_should_have_zero_values() {
        let params = VestingParams::default();
        assert_eq!(params.total_locked_amount, 0);
        assert_eq!(params.cliff_period, 0);
        assert_eq!(params.unlock_period, 0);
    }

    #[test]
    fn test_vesting_params_clone_should_work() {
        let params = VestingParams {
            total_locked_amount: 1000000,
            cliff_period: 86400,
            unlock_period: 2592000,
        };
        let cloned = params.clone();
        assert_eq!(params, cloned);
    }

    #[test]
    fn test_vesting_params_debug_should_work() {
        let params = VestingParams {
            total_locked_amount: 500000,
            cliff_period: 43200,
            unlock_period: 1296000,
        };
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("total_locked_amount: 500000"));
        assert!(debug_str.contains("cliff_period: 43200"));
        assert!(debug_str.contains("unlock_period: 1296000"));
    }

    #[test]
    fn test_vesting_params_partial_eq_should_work() {
        let params1 = VestingParams {
            total_locked_amount: 1000,
            cliff_period: 100,
            unlock_period: 200,
        };
        let params2 = VestingParams {
            total_locked_amount: 1000,
            cliff_period: 100,
            unlock_period: 200,
        };
        let params3 = VestingParams {
            total_locked_amount: 2000,
            cliff_period: 100,
            unlock_period: 200,
        };

        assert_eq!(params1, params2);
        assert_ne!(params1, params3);
    }

    #[test]
    fn test_vesting_params_serialize_deserialize_should_work() {
        let params = VestingParams {
            total_locked_amount: u64::MAX,
            cliff_period: u64::MAX / 2,
            unlock_period: 1,
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: VestingParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params, deserialized);
    }

    // ConstantCurve Tests
    #[test]
    fn test_constant_curve_default_should_have_zero_values() {
        let curve = ConstantCurve::default();
        assert_eq!(curve.supply, 0);
        assert_eq!(curve.total_base_sell, 0);
        assert_eq!(curve.total_quote_fund_raising, 0);
        assert_eq!(curve.migrate_type, 0);
    }

    #[test]
    fn test_constant_curve_clone_should_work() {
        let curve = ConstantCurve {
            supply: 1000000000,
            total_base_sell: 500000000,
            total_quote_fund_raising: 250000,
            migrate_type: 1,
        };
        let cloned = curve.clone();
        assert_eq!(curve, cloned);
    }

    #[test]
    fn test_constant_curve_debug_should_work() {
        let curve = ConstantCurve {
            supply: 1000,
            total_base_sell: 500,
            total_quote_fund_raising: 250,
            migrate_type: 2,
        };
        let debug_str = format!("{:?}", curve);
        assert!(debug_str.contains("supply: 1000"));
        assert!(debug_str.contains("total_base_sell: 500"));
        assert!(debug_str.contains("total_quote_fund_raising: 250"));
        assert!(debug_str.contains("migrate_type: 2"));
    }

    #[test]
    fn test_constant_curve_partial_eq_should_work() {
        let curve1 = ConstantCurve {
            supply: 100,
            total_base_sell: 50,
            total_quote_fund_raising: 25,
            migrate_type: 1,
        };
        let curve2 = ConstantCurve {
            supply: 100,
            total_base_sell: 50,
            total_quote_fund_raising: 25,
            migrate_type: 1,
        };
        let curve3 = ConstantCurve {
            supply: 200,
            total_base_sell: 50,
            total_quote_fund_raising: 25,
            migrate_type: 1,
        };

        assert_eq!(curve1, curve2);
        assert_ne!(curve1, curve3);
    }

    #[test]
    fn test_constant_curve_serialize_deserialize_should_work() {
        let curve = ConstantCurve {
            supply: u64::MAX,
            total_base_sell: u64::MAX / 2,
            total_quote_fund_raising: 1,
            migrate_type: u8::MAX,
        };

        let json = serde_json::to_string(&curve).unwrap();
        let deserialized: ConstantCurve = serde_json::from_str(&json).unwrap();

        assert_eq!(curve, deserialized);
    }

    // FixedCurve Tests
    #[test]
    fn test_fixed_curve_default_should_have_zero_values() {
        let curve = FixedCurve::default();
        assert_eq!(curve.supply, 0);
        assert_eq!(curve.total_quote_fund_raising, 0);
        assert_eq!(curve.migrate_type, 0);
    }

    #[test]
    fn test_fixed_curve_clone_should_work() {
        let curve = FixedCurve {
            supply: 2000000000,
            total_quote_fund_raising: 500000,
            migrate_type: 3,
        };
        let cloned = curve.clone();
        assert_eq!(curve, cloned);
    }

    #[test]
    fn test_fixed_curve_debug_should_work() {
        let curve = FixedCurve {
            supply: 5000,
            total_quote_fund_raising: 1000,
            migrate_type: 4,
        };
        let debug_str = format!("{:?}", curve);
        assert!(debug_str.contains("supply: 5000"));
        assert!(debug_str.contains("total_quote_fund_raising: 1000"));
        assert!(debug_str.contains("migrate_type: 4"));
    }

    #[test]
    fn test_fixed_curve_partial_eq_should_work() {
        let curve1 = FixedCurve {
            supply: 300,
            total_quote_fund_raising: 150,
            migrate_type: 2,
        };
        let curve2 = FixedCurve {
            supply: 300,
            total_quote_fund_raising: 150,
            migrate_type: 2,
        };
        let curve3 = FixedCurve {
            supply: 300,
            total_quote_fund_raising: 200,
            migrate_type: 2,
        };

        assert_eq!(curve1, curve2);
        assert_ne!(curve1, curve3);
    }

    #[test]
    fn test_fixed_curve_serialize_deserialize_should_work() {
        let curve = FixedCurve {
            supply: 0,
            total_quote_fund_raising: u64::MAX,
            migrate_type: 127,
        };

        let json = serde_json::to_string(&curve).unwrap();
        let deserialized: FixedCurve = serde_json::from_str(&json).unwrap();

        assert_eq!(curve, deserialized);
    }

    // LinearCurve Tests
    #[test]
    fn test_linear_curve_default_should_have_zero_values() {
        let curve = LinearCurve::default();
        assert_eq!(curve.supply, 0);
        assert_eq!(curve.total_quote_fund_raising, 0);
        assert_eq!(curve.migrate_type, 0);
    }

    #[test]
    fn test_linear_curve_clone_should_work() {
        let curve = LinearCurve {
            supply: 3000000000,
            total_quote_fund_raising: 750000,
            migrate_type: 5,
        };
        let cloned = curve.clone();
        assert_eq!(curve, cloned);
    }

    #[test]
    fn test_linear_curve_debug_should_work() {
        let curve = LinearCurve {
            supply: 7500,
            total_quote_fund_raising: 3750,
            migrate_type: 6,
        };
        let debug_str = format!("{:?}", curve);
        assert!(debug_str.contains("supply: 7500"));
        assert!(debug_str.contains("total_quote_fund_raising: 3750"));
        assert!(debug_str.contains("migrate_type: 6"));
    }

    #[test]
    fn test_linear_curve_partial_eq_should_work() {
        let curve1 = LinearCurve {
            supply: 400,
            total_quote_fund_raising: 200,
            migrate_type: 3,
        };
        let curve2 = LinearCurve {
            supply: 400,
            total_quote_fund_raising: 200,
            migrate_type: 3,
        };
        let curve3 = LinearCurve {
            supply: 400,
            total_quote_fund_raising: 200,
            migrate_type: 4,
        };

        assert_eq!(curve1, curve2);
        assert_ne!(curve1, curve3);
    }

    #[test]
    fn test_linear_curve_serialize_deserialize_should_work() {
        let curve = LinearCurve {
            supply: 1,
            total_quote_fund_raising: 0,
            migrate_type: u8::MIN,
        };

        let json = serde_json::to_string(&curve).unwrap();
        let deserialized: LinearCurve = serde_json::from_str(&json).unwrap();

        assert_eq!(curve, deserialized);
    }

    // CurveParams Tests
    #[test]
    fn test_curve_params_default_should_have_none_values() {
        let params = CurveParams::default();
        assert!(params.constant_curve.is_none());
        assert!(params.fixed_curve.is_none());
        assert!(params.linear_curve.is_none());
    }

    #[test]
    fn test_curve_params_clone_should_work() {
        let params = CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 1000,
                total_base_sell: 500,
                total_quote_fund_raising: 250,
                migrate_type: 1,
            }),
            fixed_curve: None,
            linear_curve: Some(LinearCurve {
                supply: 2000,
                total_quote_fund_raising: 1000,
                migrate_type: 2,
            }),
        };
        let cloned = params.clone();
        assert_eq!(params, cloned);
    }

    #[test]
    fn test_curve_params_debug_should_work() {
        let params = CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 100,
                total_base_sell: 50,
                total_quote_fund_raising: 25,
                migrate_type: 1,
            }),
            fixed_curve: None,
            linear_curve: None,
        };
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("constant_curve: Some"));
        assert!(debug_str.contains("fixed_curve: None"));
        assert!(debug_str.contains("linear_curve: None"));
    }

    #[test]
    fn test_curve_params_partial_eq_should_work() {
        let params1 = CurveParams {
            constant_curve: Some(ConstantCurve::default()),
            fixed_curve: None,
            linear_curve: None,
        };
        let params2 = CurveParams {
            constant_curve: Some(ConstantCurve::default()),
            fixed_curve: None,
            linear_curve: None,
        };
        let params3 = CurveParams {
            constant_curve: None,
            fixed_curve: Some(FixedCurve::default()),
            linear_curve: None,
        };

        assert_eq!(params1, params2);
        assert_ne!(params1, params3);
    }

    #[test]
    fn test_curve_params_serialize_deserialize_should_work() {
        let params = CurveParams {
            constant_curve: None,
            fixed_curve: Some(FixedCurve {
                supply: u64::MAX,
                total_quote_fund_raising: 12345,
                migrate_type: 99,
            }),
            linear_curve: Some(LinearCurve {
                supply: 0,
                total_quote_fund_raising: u64::MAX,
                migrate_type: 0,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: CurveParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params, deserialized);
    }

    #[test]
    fn test_curve_params_with_all_curves_should_work() {
        let params = CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 1000,
                total_base_sell: 800,
                total_quote_fund_raising: 200,
                migrate_type: 1,
            }),
            fixed_curve: Some(FixedCurve {
                supply: 2000,
                total_quote_fund_raising: 400,
                migrate_type: 2,
            }),
            linear_curve: Some(LinearCurve {
                supply: 3000,
                total_quote_fund_raising: 600,
                migrate_type: 3,
            }),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: CurveParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params, deserialized);
        assert!(params.constant_curve.is_some());
        assert!(params.fixed_curve.is_some());
        assert!(params.linear_curve.is_some());
    }

    #[test]
    fn test_curve_params_with_only_constant_curve_should_work() {
        let params = CurveParams {
            constant_curve: Some(ConstantCurve {
                supply: 5000,
                total_base_sell: 2500,
                total_quote_fund_raising: 1250,
                migrate_type: 7,
            }),
            fixed_curve: None,
            linear_curve: None,
        };

        assert!(params.constant_curve.is_some());
        assert!(params.fixed_curve.is_none());
        assert!(params.linear_curve.is_none());

        let constant = params.constant_curve.unwrap();
        assert_eq!(constant.supply, 5000);
        assert_eq!(constant.total_base_sell, 2500);
        assert_eq!(constant.total_quote_fund_raising, 1250);
        assert_eq!(constant.migrate_type, 7);
    }
}
