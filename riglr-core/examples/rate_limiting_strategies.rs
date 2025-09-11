//! Example demonstrating different rate limiting strategies

use riglr_core::util::{RateLimitStrategyType, RateLimiter, TokenBucketStrategy};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Default Token Bucket Strategy
    println!("=== Token Bucket Strategy ===");
    let token_limiter = RateLimiter::new(5, Duration::from_secs(10));

    // Make some requests
    for i in 1..=7 {
        let result = token_limiter.check_rate_limit("user1");
        match result {
            Ok(()) => println!("Request {} allowed", i),
            Err(e) => println!("Request {} blocked: {}", i, e),
        }
    }

    println!("\n=== Fixed Window Strategy ===");
    // Example 2: Fixed Window Strategy via builder
    let fixed_limiter = RateLimiter::builder()
        .strategy(RateLimitStrategyType::FixedWindow)
        .max_requests(3)
        .time_window(Duration::from_secs(5))
        .build();

    for i in 1..=5 {
        let result = fixed_limiter.check_rate_limit("user2");
        match result {
            Ok(()) => println!("Request {} allowed", i),
            Err(e) => println!("Request {} blocked: {}", i, e),
        }
    }

    println!("\n=== Custom Token Bucket with Burst ===");
    // Example 3: Custom strategy directly
    let burst_strategy = TokenBucketStrategy::with_burst(
        10,                      // 10 requests per window
        Duration::from_secs(60), // 60 second window
        15,                      // Allow burst up to 15
    );
    let burst_limiter = RateLimiter::with_strategy(burst_strategy);

    // Burst requests
    for i in 1..=20 {
        let result = burst_limiter.check_rate_limit("user3");
        match result {
            Ok(()) => println!("Burst request {} allowed", i),
            Err(e) => println!("Burst request {} blocked: {}", i, e),
        }
    }

    // Show strategy names
    println!("\nStrategy names:");
    let token_strategy = token_limiter.strategy_name();
    let fixed_strategy = fixed_limiter.strategy_name();
    let burst_strategy = burst_limiter.strategy_name();
    println!("Token limiter: {}", token_strategy);
    println!("Fixed limiter: {}", fixed_strategy);
    println!("Burst limiter: {}", burst_strategy);

    Ok(())
}
