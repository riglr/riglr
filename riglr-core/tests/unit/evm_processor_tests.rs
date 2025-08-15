use riglr_core::transactions::evm::GasConfig;
use alloy::primitives::U256;

#[test]
fn test_gas_config_default() {
    let config = GasConfig::default();
    assert!(config.use_eip1559);
    assert_eq!(config.gas_price_multiplier, 1.1);
    assert_eq!(config.max_gas_price, None);
    assert_eq!(config.max_priority_fee, Some(U256::from(2_000_000_000u64)));
}

#[test]
fn test_gas_config_custom() {
    let config = GasConfig {
        use_eip1559: false,
        gas_price_multiplier: 1.5,
        max_gas_price: Some(U256::from(100_000_000_000u64)),
        max_priority_fee: Some(U256::from(5_000_000_000u64)),
    };
    
    assert!(!config.use_eip1559);
    assert_eq!(config.gas_price_multiplier, 1.5);
    assert_eq!(config.max_gas_price, Some(U256::from(100_000_000_000u64)));
    assert_eq!(config.max_priority_fee, Some(U256::from(5_000_000_000u64)));
}

#[test]
fn test_gas_config_clone() {
    let config = GasConfig::default();
    let cloned = config.clone();
    
    assert_eq!(cloned.use_eip1559, config.use_eip1559);
    assert_eq!(cloned.gas_price_multiplier, config.gas_price_multiplier);
    assert_eq!(cloned.max_gas_price, config.max_gas_price);
    assert_eq!(cloned.max_priority_fee, config.max_priority_fee);
}

#[test]
fn test_gas_config_debug() {
    let config = GasConfig::default();
    let debug_str = format!("{:?}", config);
    
    assert!(debug_str.contains("GasConfig"));
    assert!(debug_str.contains("use_eip1559"));
    assert!(debug_str.contains("gas_price_multiplier"));
}

#[test]
fn test_gas_config_variations() {
    // Test with no priority fee
    let config1 = GasConfig {
        use_eip1559: true,
        gas_price_multiplier: 1.0,
        max_gas_price: None,
        max_priority_fee: None,
    };
    assert!(config1.max_priority_fee.is_none());
    
    // Test with max gas price cap
    let config2 = GasConfig {
        use_eip1559: false,
        gas_price_multiplier: 2.0,
        max_gas_price: Some(U256::from(50_000_000_000u64)),
        max_priority_fee: None,
    };
    assert_eq!(config2.max_gas_price, Some(U256::from(50_000_000_000u64)));
    
    // Test with all fields set
    let config3 = GasConfig {
        use_eip1559: true,
        gas_price_multiplier: 1.25,
        max_gas_price: Some(U256::from(200_000_000_000u64)),
        max_priority_fee: Some(U256::from(3_000_000_000u64)),
    };
    assert_eq!(config3.gas_price_multiplier, 1.25);
    assert_eq!(config3.max_priority_fee, Some(U256::from(3_000_000_000u64)));
}