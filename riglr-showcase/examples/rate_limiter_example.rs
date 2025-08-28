//! Example of using RateLimiter with ApplicationContext
//!
//! This example demonstrates how to configure and use the RateLimiter service
//! through dependency injection instead of global statics.

use anyhow::Result;
use riglr_config::Config;
use riglr_core::provider::ApplicationContext;
use riglr_core::util::RateLimiter;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env();

    // Create application context
    let context = ApplicationContext::from_config(&Arc::new(config));

    // Create and configure rate limiter
    // Option 1: Simple rate limiter with default settings
    let _simple_limiter = RateLimiter::new(10, Duration::from_secs(60));

    // Option 2: Advanced rate limiter with burst support
    let advanced_limiter = RateLimiter::builder()
        .max_requests(100)
        .time_window(Duration::from_secs(60))
        .burst_size(20) // Allow bursts of up to 20 additional requests
        .build();

    // Add rate limiter to application context
    context.set_extension(Arc::new(advanced_limiter));

    // Now tools can access the rate limiter from context
    simulate_tool_usage(&context).await?;

    Ok(())
}

/// Simulates a tool that uses rate limiting from ApplicationContext
async fn simulate_tool_usage(context: &ApplicationContext) -> Result<()> {
    // Get rate limiter from context
    let rate_limiter = context
        .get_extension::<Arc<RateLimiter>>()
        .ok_or_else(|| anyhow::anyhow!("RateLimiter not found in ApplicationContext"))?;

    // Simulate multiple requests from different users
    let users = vec!["user1", "user2", "user3"];

    for user in users {
        println!("\nSimulating requests from {}", user);

        // Make multiple requests
        for i in 1..=5 {
            match rate_limiter.check_rate_limit(user) {
                Ok(()) => {
                    println!("  ✅ Request {} allowed for {}", i, user);
                }
                Err(e) => {
                    println!("  ❌ Request {} blocked for {}: {}", i, user, e);

                    // Check if we have retry_after information
                    if let Some(retry_after) = e.retry_after() {
                        println!("     Retry after: {:?}", retry_after);
                    }
                    break;
                }
            }

            // Small delay between requests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // Demonstrate getting request count
    println!("\n📊 Current request counts:");
    for user in &["user1", "user2", "user3"] {
        let count = rate_limiter.get_request_count(user);
        println!("  {}: {} requests", user, count);
    }

    // Demonstrate resetting a user's rate limit
    println!("\n🔄 Resetting rate limit for user1...");
    rate_limiter.reset_client("user1");

    // Try again after reset
    match rate_limiter.check_rate_limit("user1") {
        Ok(()) => println!("  ✅ user1 can make requests again"),
        Err(e) => println!("  ❌ user1 still blocked: {}", e),
    }

    Ok(())
}
