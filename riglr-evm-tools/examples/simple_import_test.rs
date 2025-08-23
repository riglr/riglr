//! Simple import test for riglr-evm-tools refactoring
//! Tests that the most important functions are accessible

use riglr_evm_tools::{chain_name_to_id, parse_evm_address};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing basic imports from riglr-evm-tools...");

    // Test address parsing
    let addr = parse_evm_address("0x742d35Cc6634C0532925a3b8D94E07F644b71Dc2")?;
    println!("✅ Address parsing works: {:?}", addr);

    // Test chain mapping
    let chain_id = chain_name_to_id("ethereum")?;
    println!("✅ Chain mapping works: ethereum -> {}", chain_id);

    println!("🎉 Basic imports test passed!");
    Ok(())
}
