use riglr_core::config::{EvmNetworkConfig, RpcConfig, SolanaNetworkConfig};
use std::collections::HashMap;
use std::env;

#[test]
fn default_networks_and_caip2() {
    let cfg = RpcConfig::default();
    // a couple of known networks exist
    assert!(cfg.evm_networks.contains_key("ethereum"));
    assert!(cfg.evm_networks.contains_key("base"));
    assert!(cfg.solana_networks.contains_key("mainnet"));

    // caip2 format
    let eth = cfg.evm_networks.get("ethereum").unwrap();
    assert_eq!(eth.caip2(), "eip155:1");
}

#[test]
fn evm_caip2_for_case_insensitive_and_missing() {
    let cfg = RpcConfig::default();
    assert_eq!(cfg.evm_caip2_for("EtHeReUm").as_deref(), Some("eip155:1"));
    assert!(cfg.evm_caip2_for("unknown").is_none());
}

#[test]
fn add_evm_network_overwrite_and_lowercase_key() {
    let mut cfg = RpcConfig::default();
    // Add a new custom network with mixed case key
    cfg.add_evm_network("MyChain", 4242, "http://rpc", Some("http://exp".into()));
    // Key should be lowercased
    assert!(cfg.evm_networks.contains_key("mychain"));
    let net = cfg.evm_networks.get("mychain").unwrap();
    assert_eq!(net.name, "mychain");
    assert_eq!(net.chain_id, 4242);

    // Overwrite existing by same key
    cfg.add_evm_network("MYCHAIN", 4343, "http://rpc2", None);
    let net2 = cfg.evm_networks.get("mychain").unwrap();
    assert_eq!(net2.chain_id, 4343);
    assert_eq!(net2.explorer_url, None);
}

#[test]
fn env_overrides_update_existing_and_add_new() {
    // Ensure clean env
    unsafe {
        env::remove_var("RPC_URL_8453");
    }
    unsafe {
        env::remove_var("RPC_URL_999999");
    }
    unsafe {
        env::remove_var("RPC_URL_NOTNUM");
    }

    // Override Base (8453) and add a new one 999999
    unsafe {
        env::set_var("RPC_URL_8453", "https://override.base");
    }
    unsafe {
        env::set_var("RPC_URL_999999", "https://new.chain");
    }
    // Should be ignored
    unsafe {
        env::set_var("RPC_URL_NOTNUM", "https://ignored");
    }

    let cfg = RpcConfig::default().with_env_overrides();

    // Existing (base) should be updated
    let base = cfg.evm_networks.get("base").unwrap();
    assert_eq!(base.rpc_url, "https://override.base");

    // New one should be added as chain_999999
    let added = cfg.evm_networks.get("chain_999999").unwrap();
    assert_eq!(added.chain_id, 999999);
    assert_eq!(added.rpc_url, "https://new.chain");

    // Cleanup
    unsafe {
        env::remove_var("RPC_URL_8453");
    }
    unsafe {
        env::remove_var("RPC_URL_999999");
    }
    unsafe {
        env::remove_var("RPC_URL_NOTNUM");
    }
}

#[test]
fn test_all_default_evm_networks() {
    let config = RpcConfig::default();

    // Verify all 16 EVM networks are initialized
    assert_eq!(config.evm_networks.len(), 16);

    // Test polygon configuration
    let polygon = config.evm_networks.get("polygon").unwrap();
    assert_eq!(polygon.name, "Polygon");
    assert_eq!(polygon.chain_id, 137);
    assert_eq!(polygon.rpc_url, "https://polygon.llamarpc.com");
    assert_eq!(
        polygon.explorer_url,
        Some("https://polygonscan.com".to_string())
    );

    // Test arbitrum configuration
    let arbitrum = config.evm_networks.get("arbitrum").unwrap();
    assert_eq!(arbitrum.name, "Arbitrum One");
    assert_eq!(arbitrum.chain_id, 42161);
    assert_eq!(arbitrum.rpc_url, "https://arbitrum.llamarpc.com");
    assert_eq!(
        arbitrum.explorer_url,
        Some("https://arbiscan.io".to_string())
    );

    // Test optimism configuration
    let optimism = config.evm_networks.get("optimism").unwrap();
    assert_eq!(optimism.name, "Optimism");
    assert_eq!(optimism.chain_id, 10);
    assert_eq!(optimism.rpc_url, "https://optimism.llamarpc.com");
    assert_eq!(
        optimism.explorer_url,
        Some("https://optimistic.etherscan.io".to_string())
    );

    // Test bsc configuration
    let bsc = config.evm_networks.get("bsc").unwrap();
    assert_eq!(bsc.name, "BNB Smart Chain");
    assert_eq!(bsc.chain_id, 56);
    assert_eq!(bsc.rpc_url, "https://bsc.llamarpc.com");
    assert_eq!(bsc.explorer_url, Some("https://bscscan.com".to_string()));

    // Test avalanche configuration
    let avalanche = config.evm_networks.get("avalanche").unwrap();
    assert_eq!(avalanche.name, "Avalanche C-Chain");
    assert_eq!(avalanche.chain_id, 43114);
    assert_eq!(avalanche.rpc_url, "https://avalanche.llamarpc.com");
    assert_eq!(
        avalanche.explorer_url,
        Some("https://snowtrace.io".to_string())
    );

    // Test gnosis configuration
    let gnosis = config.evm_networks.get("gnosis").unwrap();
    assert_eq!(gnosis.name, "Gnosis");
    assert_eq!(gnosis.chain_id, 100);
    assert_eq!(gnosis.rpc_url, "https://gnosis.llamarpc.com");
    assert_eq!(
        gnosis.explorer_url,
        Some("https://gnosisscan.io".to_string())
    );

    // Test fantom configuration
    let fantom = config.evm_networks.get("fantom").unwrap();
    assert_eq!(fantom.name, "Fantom");
    assert_eq!(fantom.chain_id, 250);
    assert_eq!(fantom.rpc_url, "https://fantom.llamarpc.com");
    assert_eq!(fantom.explorer_url, Some("https://ftmscan.com".to_string()));

    // Test linea configuration
    let linea = config.evm_networks.get("linea").unwrap();
    assert_eq!(linea.name, "Linea");
    assert_eq!(linea.chain_id, 59144);
    assert_eq!(linea.rpc_url, "https://linea.blockpi.network/v1/rpc/public");
    assert_eq!(
        linea.explorer_url,
        Some("https://lineascan.build".to_string())
    );

    // Test scroll configuration
    let scroll = config.evm_networks.get("scroll").unwrap();
    assert_eq!(scroll.name, "Scroll");
    assert_eq!(scroll.chain_id, 534352);
    assert_eq!(scroll.rpc_url, "https://rpc.scroll.io");
    assert_eq!(
        scroll.explorer_url,
        Some("https://scrollscan.com".to_string())
    );

    // Test blast configuration
    let blast = config.evm_networks.get("blast").unwrap();
    assert_eq!(blast.name, "Blast");
    assert_eq!(blast.chain_id, 81457);
    assert_eq!(blast.rpc_url, "https://blast.blockpi.network/v1/rpc/public");
    assert_eq!(blast.explorer_url, Some("https://blastscan.io".to_string()));

    // Test mode configuration
    let mode = config.evm_networks.get("mode").unwrap();
    assert_eq!(mode.name, "Mode");
    assert_eq!(mode.chain_id, 34443);
    assert_eq!(mode.rpc_url, "https://mainnet.mode.network");
    assert_eq!(mode.explorer_url, Some("https://modescan.io".to_string()));

    // Test mantle configuration
    let mantle = config.evm_networks.get("mantle").unwrap();
    assert_eq!(mantle.name, "Mantle");
    assert_eq!(mantle.chain_id, 5000);
    assert_eq!(mantle.rpc_url, "https://mantle.publicnode.com");
    assert_eq!(
        mantle.explorer_url,
        Some("https://explorer.mantle.xyz".to_string())
    );

    // Test celo configuration
    let celo = config.evm_networks.get("celo").unwrap();
    assert_eq!(celo.name, "Celo");
    assert_eq!(celo.chain_id, 42220);
    assert_eq!(celo.rpc_url, "https://forno.celo.org");
    assert_eq!(celo.explorer_url, Some("https://celoscan.io".to_string()));

    // Test cronos configuration
    let cronos = config.evm_networks.get("cronos").unwrap();
    assert_eq!(cronos.name, "Cronos");
    assert_eq!(cronos.chain_id, 25);
    assert_eq!(cronos.rpc_url, "https://node.cronos.org/rpc");
    assert_eq!(
        cronos.explorer_url,
        Some("https://cronoscan.com".to_string())
    );
}

#[test]
fn test_solana_networks() {
    let config = RpcConfig::default();

    assert_eq!(config.solana_networks.len(), 2);

    let mainnet = config.solana_networks.get("mainnet").unwrap();
    assert_eq!(mainnet.name, "Solana Mainnet");
    assert_eq!(mainnet.rpc_url, "https://api.mainnet-beta.solana.com");
    assert_eq!(
        mainnet.explorer_url,
        Some("https://explorer.solana.com".to_string())
    );

    let devnet = config.solana_networks.get("devnet").unwrap();
    assert_eq!(devnet.name, "Solana Devnet");
    assert_eq!(devnet.rpc_url, "https://api.devnet.solana.com");
    assert_eq!(
        devnet.explorer_url,
        Some("https://explorer.solana.com".to_string())
    );
}

#[test]
fn test_method_chaining() {
    let mut config = RpcConfig {
        evm_networks: HashMap::new(),
        solana_networks: HashMap::new(),
    };

    // Test method chaining returns &mut Self
    config
        .add_evm_network("chain1", 1001, "http://chain1.test", None)
        .add_evm_network(
            "chain2",
            1002,
            "http://chain2.test",
            Some("http://explorer2".to_string()),
        )
        .add_evm_network("chain3", 1003, "http://chain3.test", None);

    assert_eq!(config.evm_networks.len(), 3);
    assert!(config.evm_networks.contains_key("chain1"));
    assert!(config.evm_networks.contains_key("chain2"));
    assert!(config.evm_networks.contains_key("chain3"));
}

#[test]
fn test_serialization() {
    let config = RpcConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RpcConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.evm_networks.len(), deserialized.evm_networks.len());
    assert_eq!(
        config.solana_networks.len(),
        deserialized.solana_networks.len()
    );

    // Test a specific network after deserialization
    let eth = deserialized.evm_networks.get("ethereum").unwrap();
    assert_eq!(eth.name, "Ethereum Mainnet");
    assert_eq!(eth.chain_id, 1);
}

#[test]
fn test_debug_trait() {
    let config = RpcConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("RpcConfig"));
    assert!(debug_str.contains("evm_networks"));
    assert!(debug_str.contains("solana_networks"));

    let evm_config = EvmNetworkConfig {
        name: "Test".to_string(),
        chain_id: 1,
        rpc_url: "http://test".to_string(),
        explorer_url: None,
    };
    let evm_debug = format!("{:?}", evm_config);
    assert!(evm_debug.contains("EvmNetworkConfig"));

    let solana_config = SolanaNetworkConfig {
        name: "Test".to_string(),
        rpc_url: "http://test".to_string(),
        explorer_url: None,
    };
    let solana_debug = format!("{:?}", solana_config);
    assert!(solana_debug.contains("SolanaNetworkConfig"));
}

#[test]
fn test_clone_trait() {
    let config = RpcConfig::default();
    let cloned = config.clone();

    assert_eq!(config.evm_networks.len(), cloned.evm_networks.len());
    assert_eq!(config.solana_networks.len(), cloned.solana_networks.len());

    let evm_config = EvmNetworkConfig {
        name: "Test".to_string(),
        chain_id: 1,
        rpc_url: "http://test".to_string(),
        explorer_url: Some("http://explorer".to_string()),
    };
    let evm_cloned = evm_config.clone();
    assert_eq!(evm_config.name, evm_cloned.name);
    assert_eq!(evm_config.chain_id, evm_cloned.chain_id);

    let solana_config = SolanaNetworkConfig {
        name: "Test".to_string(),
        rpc_url: "http://test".to_string(),
        explorer_url: Some("http://explorer".to_string()),
    };
    let solana_cloned = solana_config.clone();
    assert_eq!(solana_config.name, solana_cloned.name);
}

#[test]
fn test_env_override_multiple_chains() {
    // Test overriding multiple existing chains and adding new ones
    unsafe {
        env::set_var("RPC_URL_1", "http://custom-eth");
    }
    unsafe {
        env::set_var("RPC_URL_137", "http://custom-polygon");
    }
    unsafe {
        env::set_var("RPC_URL_42161", "http://custom-arbitrum");
    }
    unsafe {
        env::set_var("RPC_URL_777", "http://new-chain-777");
    }

    let config = RpcConfig::default().with_env_overrides();

    // Check overrides
    assert_eq!(
        config.evm_networks.get("ethereum").unwrap().rpc_url,
        "http://custom-eth"
    );
    assert_eq!(
        config.evm_networks.get("polygon").unwrap().rpc_url,
        "http://custom-polygon"
    );
    assert_eq!(
        config.evm_networks.get("arbitrum").unwrap().rpc_url,
        "http://custom-arbitrum"
    );

    // Check new chain
    assert!(config.evm_networks.contains_key("chain_777"));
    assert_eq!(config.evm_networks.get("chain_777").unwrap().chain_id, 777);
    assert_eq!(
        config.evm_networks.get("chain_777").unwrap().rpc_url,
        "http://new-chain-777"
    );

    // Cleanup
    unsafe {
        env::remove_var("RPC_URL_1");
    }
    unsafe {
        env::remove_var("RPC_URL_137");
    }
    unsafe {
        env::remove_var("RPC_URL_42161");
    }
    unsafe {
        env::remove_var("RPC_URL_777");
    }
}
