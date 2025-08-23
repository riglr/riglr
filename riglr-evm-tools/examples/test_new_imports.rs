use riglr_evm_tools::{chain_id_to_name, chain_name_to_id, parse_evm_address, EvmConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test new import paths
    let address = "0x742d35Cc6634C0532925a3b8D94E07F644b71Dc2";
    let result = parse_evm_address(address)?;
    println!("Address parsing: {:?}", result);

    let chain_id = chain_name_to_id("ethereum")?;
    println!("Chain name to ID: {} -> {}", "ethereum", chain_id);

    let chain_name = chain_id_to_name(1)?;
    println!("Chain ID to name: {} -> {}", 1, chain_name);

    // Test config creation
    let config = EvmConfig::default();
    println!(
        "Default config: chain_id={}, timeout={}s",
        config.chain_id, config.timeout_seconds
    );

    println!("New import paths working correctly!");
    Ok(())
}
