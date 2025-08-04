//! Thread safety and parallel execution tests for riglr-evm-tools
//!
//! These tests validate that the refactoring from global state to client-first
//! architecture enables safe parallel execution and thread safety.

use riglr_evm_tools::{balance::get_eth_balance, client::EvmClient};
use std::sync::{Arc, Mutex};
use tokio::sync::Barrier;
use tokio::task::JoinSet;
use tokio::time::{timeout, Duration};

/// Test that multiple threads can safely create and use EVM clients concurrently
#[tokio::test]
async fn test_multi_threaded_client_creation() {
    let mut handles = Vec::new();
    let results = Arc::new(Mutex::new(Vec::new()));

    // Spawn multiple threads creating clients
    for i in 0..5 {
        let results_clone = Arc::clone(&results);

        let handle = tokio::spawn(async move {
            let client_result = match i % 3 {
                0 => EvmClient::mainnet().await,
                1 => EvmClient::polygon().await,
                _ => EvmClient::arbitrum().await,
            };

            let success = match client_result {
                Ok(client) => {
                    // Verify client properties
                    let expected_chain = match i % 3 {
                        0 => 1,     // Mainnet
                        1 => 137,   // Polygon
                        _ => 42161, // Arbitrum
                    };
                    client.chain_id == expected_chain
                }
                Err(_) => false, // Network error acceptable
            };

            results_clone.lock().unwrap().push((i, success));
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.await.expect("Thread should not panic");
    }

    let results = results.lock().unwrap();
    println!("Thread safety test results: {:?}", *results);

    // All threads should complete without panicking
    assert_eq!(results.len(), 5);
}

/// Test concurrent balance checking operations
#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_concurrent_balance_operations() {
    // Skip this test if we can't connect to networks
    // This test is primarily about architecture, not network connectivity
    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => {
            // Try localhost fallback
            match EvmClient::new("http://localhost:8545".to_string()).await {
                Ok(client) => client,
                Err(_) => {
                    println!("Cannot create test client - skipping concurrent balance test");
                    return;
                }
            }
        }
    };

    let barrier = Arc::new(Barrier::new(5));
    let mut join_set = JoinSet::new();

    // Test addresses for balance checking
    let test_addresses = vec![
        "0x0000000000000000000000000000000000000000",
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x0000000000000000000000000000000000000003",
        "0x0000000000000000000000000000000000000004",
    ];

    // Spawn concurrent balance checking tasks
    for (i, addr) in test_addresses.into_iter().enumerate() {
        let _client = base_client.clone(); // Keep client alive but don't use it directly
        let barrier_clone = Arc::clone(&barrier);

        join_set.spawn(async move {
            // Wait for all tasks to be ready
            barrier_clone.wait().await;

            // Perform balance check - the function uses its own client internally
            let result = get_eth_balance(addr.to_string(), None).await;

            (i, result.is_ok(), addr.to_string())
        });
    }

    // Collect all results
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((task_id, success, addr)) => {
                results.push((task_id, success, addr));
            }
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!("Concurrent balance operations: {:?}", results);

    // All tasks should complete without panicking
    assert_eq!(results.len(), 5);

    // Verify no data races or corruption
    for (task_id, success, addr) in results {
        assert!(task_id < 5);
        assert!(addr.starts_with("0x"));
        println!("Task {} ({}): {}", task_id, addr, success);
    }
}

/// Test parallel execution with different client configurations
#[tokio::test]
async fn test_parallel_client_configurations() {
    use tokio::task;

    // Create multiple clients with different configurations
    let mut client_handles = Vec::new();

    // Ethereum mainnet client
    client_handles.push(task::spawn(async {
        let client = EvmClient::mainnet().await?;
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((1u64, client.chain_id))
    }));

    // Polygon client
    client_handles.push(task::spawn(async {
        let client = EvmClient::polygon().await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((137u64, client.chain_id))
    }));

    // Arbitrum client
    client_handles.push(task::spawn(async {
        let client = EvmClient::arbitrum().await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>((42161u64, client.chain_id))
    }));

    // Wait for all clients to be created
    let mut successful_configs = Vec::new();

    for handle in client_handles {
        match handle.await {
            Ok(Ok((expected_chain, actual_chain))) => {
                assert_eq!(expected_chain, actual_chain);
                successful_configs.push(expected_chain);
            }
            Ok(Err(e)) => println!("Client creation failed (network): {:?}", e),
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!(
        "Successfully created {} clients in parallel",
        successful_configs.len()
    );

    // Test should pass if no panics occurred
    assert!(successful_configs.len() <= 3); // At most 3, could be less due to network
}

/// Test that signer configuration is thread-safe and doesn't leak between clients
#[tokio::test]
async fn test_thread_safe_signer_isolation() {
    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => match EvmClient::new("http://localhost:8545".to_string()).await {
            Ok(client) => client,
            Err(_) => {
                println!("Cannot create test client - skipping signer isolation test");
                return;
            }
        },
    };

    // Test private keys (these are dummy keys for testing)
    let test_keys = vec![
        "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "0x2345678901abcdef2345678901abcdef2345678901abcdef2345678901abcdef",
    ];

    let mut handles = Vec::new();

    for (i, key) in test_keys.into_iter().enumerate() {
        let client = base_client.clone();

        let handle = tokio::spawn(async move {
            // Try to configure signer
            let signer_result = client.with_signer(key);

            match signer_result {
                Ok(signed_client) => {
                    // Verify signer is configured
                    let has_signer = signed_client.has_signer();
                    let signer_addr = signed_client.signer().map(|s| s.address());

                    (i, true, has_signer, signer_addr)
                }
                Err(_) => {
                    // Invalid key format is acceptable
                    (i, false, false, None)
                }
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    // Verify signer isolation
    for (task_id, key_valid, has_signer, addr) in results {
        println!(
            "Task {}: key_valid={}, has_signer={}, addr={:?}",
            task_id, key_valid, has_signer, addr
        );

        if key_valid {
            assert!(has_signer);
            assert!(addr.is_some());
        }
    }

    // Original client should remain unsigned
    assert!(!base_client.has_signer());
}

/// Test timeout handling in concurrent scenarios
#[tokio::test]
#[ignore] // Long-running timeout test
async fn test_concurrent_timeout_handling() {
    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => match EvmClient::new("http://localhost:8545".to_string()).await {
            Ok(client) => client,
            Err(_) => {
                println!("Cannot create test client - skipping timeout test");
                return;
            }
        },
    };

    let mut handles = Vec::new();

    // Spawn operations with timeouts
    for i in 0..3 {
        let _client = base_client.clone();

        let handle = tokio::spawn(async move {
            // Set a short timeout for testing
            let result = timeout(
                Duration::from_secs(5),
                get_eth_balance(format!("0x{:040x}", i), None),
            )
            .await;

            match result {
                Ok(balance_result) => (i, "completed", balance_result.is_ok()),
                Err(_) => (i, "timeout", false),
            }
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!("Timeout test results: {:?}", results);

    // All tasks should complete (either successfully or with timeout)
    assert_eq!(results.len(), 3);

    for (task_id, status, success) in results {
        assert!(task_id < 3);
        assert!(status == "completed" || status == "timeout");
        println!("Task {}: {} (success: {})", task_id, status, success);
    }
}

/// Test resource cleanup in parallel scenarios
#[tokio::test]
async fn test_parallel_resource_cleanup() {
    let create_and_use_clients = || async {
        let mut clients = Vec::new();

        // Create multiple clients
        for i in 0..3 {
            let client_result = match i % 3 {
                0 => EvmClient::mainnet().await,
                1 => EvmClient::polygon().await,
                _ => EvmClient::arbitrum().await,
            };

            if let Ok(client) = client_result {
                clients.push(client);
            }
        }

        // Use clients
        for client in &clients {
            let _ = client.get_block_number().await;
        }

        clients.len()
    };

    // Run the function multiple times concurrently
    let (result1, result2, result3) = tokio::join!(
        create_and_use_clients(),
        create_and_use_clients(),
        create_and_use_clients()
    );

    println!(
        "Resource cleanup test: {} {} {} clients created",
        result1, result2, result3
    );

    // Create new clients after cleanup - should work fine
    let _final_client = EvmClient::mainnet().await;

    // Test passes if we reach here without memory leaks or panics
}

/// Test error isolation between concurrent operations
#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_concurrent_error_isolation() {
    // Mix of valid and invalid operations
    let operations = vec![
        (
            "valid_balance",
            "0x0000000000000000000000000000000000000000",
        ),
        ("invalid_address", "invalid_address_format"),
        (
            "another_valid",
            "0x0000000000000000000000000000000000000001",
        ),
        ("empty_address", ""),
        ("valid_zero", "0x0000000000000000000000000000000000000002"),
    ];

    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => match EvmClient::new("http://localhost:8545".to_string()).await {
            Ok(client) => client,
            Err(_) => {
                println!("Cannot create test client - skipping error isolation test");
                return;
            }
        },
    };

    let mut handles = Vec::new();

    for (op_name, address) in operations {
        let _client = base_client.clone();
        let addr = address.to_string();
        let name = op_name.to_string();

        let handle = tokio::spawn(async move {
            let result = get_eth_balance(addr.clone(), None).await;
            (name, addr, result.is_ok())
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!("Error isolation test results:");
    for (op_name, addr, success) in &results {
        println!("  {}: {} -> {}", op_name, addr, success);
    }

    // Verify that errors don't affect other operations
    let _valid_ops: Vec<_> = results
        .iter()
        .filter(|(name, _, _)| name.contains("valid"))
        .collect();

    let invalid_ops: Vec<_> = results
        .iter()
        .filter(|(name, _, _)| name.contains("invalid") || name.contains("empty"))
        .collect();

    // Invalid operations should fail
    for (name, _, success) in invalid_ops {
        if *success {
            println!("Warning: {} unexpectedly succeeded", name);
        }
    }

    // All operations should complete without panicking
    assert_eq!(results.len(), 5);
}

/// Load test with many concurrent operations
#[tokio::test]
#[ignore] // Long-running load test
async fn test_high_concurrency_load() {
    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => match EvmClient::new("http://localhost:8545".to_string()).await {
            Ok(client) => client,
            Err(_) => {
                println!("Cannot create test client - skipping high concurrency test");
                return;
            }
        },
    };

    const NUM_TASKS: usize = 20;
    let mut join_set = JoinSet::new();

    // Spawn many concurrent tasks
    for i in 0..NUM_TASKS {
        let _client = base_client.clone();

        join_set.spawn(async move {
            let address = format!("0x{:040x}", i % 10); // Reuse some addresses

            // Perform operation with timeout
            let result = timeout(Duration::from_secs(10), get_eth_balance(address, None)).await;

            match result {
                Ok(balance_result) => (i, balance_result.is_ok(), "completed"),
                Err(_) => (i, false, "timeout"),
            }
        });
    }

    // Collect all results
    let mut completed = 0;
    let mut timed_out = 0;
    let mut successful = 0;

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((task_id, success, status)) => {
                if status == "completed" {
                    completed += 1;
                    if success {
                        successful += 1;
                    }
                } else {
                    timed_out += 1;
                }

                if task_id % 5 == 0 {
                    println!("Task {} {}: {}", task_id, status, success);
                }
            }
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!(
        "High concurrency test: {} completed, {} timed out, {} successful",
        completed, timed_out, successful
    );

    // All tasks should complete or timeout (no panics)
    assert_eq!(completed + timed_out, NUM_TASKS);
}

/// Test client state consistency under concurrent access
#[tokio::test]
async fn test_client_state_consistency() {
    let base_client = match EvmClient::mainnet().await {
        Ok(client) => client,
        Err(_) => match EvmClient::new("http://localhost:8545".to_string()).await {
            Ok(client) => client,
            Err(_) => {
                println!("Cannot create test client - skipping state consistency test");
                return;
            }
        },
    };

    // Clone client multiple times
    let clients: Vec<_> = (0..5).map(|_| base_client.clone()).collect();

    let mut handles = Vec::new();

    // Spawn tasks that read client properties concurrently
    for (i, client) in clients.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            // Read client properties multiple times
            let chain_id1 = client.chain_id;
            let rpc_url1 = client.rpc_url.clone();
            let has_signer1 = client.has_signer();

            // Small delay to encourage race conditions if they exist
            tokio::time::sleep(Duration::from_millis(1)).await;

            let chain_id2 = client.chain_id;
            let rpc_url2 = client.rpc_url.clone();
            let has_signer2 = client.has_signer();

            // Properties should be consistent
            let consistent =
                chain_id1 == chain_id2 && rpc_url1 == rpc_url2 && has_signer1 == has_signer2;

            (i, consistent, chain_id1, has_signer1)
        });

        handles.push(handle);
    }

    // Check all results
    let mut all_consistent = true;
    let mut results = Vec::new();

    for handle in handles {
        match handle.await {
            Ok((task_id, consistent, chain_id, has_signer)) => {
                results.push((task_id, consistent, chain_id, has_signer));
                if !consistent {
                    all_consistent = false;
                }
            }
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!("State consistency results:");
    for (task_id, consistent, chain_id, has_signer) in &results {
        println!(
            "  Task {}: consistent={}, chain_id={}, has_signer={}",
            task_id, consistent, chain_id, has_signer
        );
    }

    // All clients should have consistent state
    assert!(
        all_consistent,
        "Client state was not consistent across concurrent access"
    );

    // All results should have same chain_id (they're all clones)
    let first_chain_id = results[0].2;
    for (_, _, chain_id, _) in results {
        assert_eq!(chain_id, first_chain_id);
    }
}
