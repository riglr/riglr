//! Comprehensive error handling example showing the new `ToolError` architecture
//!
//! This demonstrates:
//! - Structured error types with retry classification
//! - Error serialization for distributed processing
//! - Proper error context and source preservation
//! - Migration from Box<dyn Error> to structured errors

use core::{error::Error, time::Duration};
use riglr_core::{JobResult, ToolError};
use std::io;

#[expect(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    println!("=== riglr-core Error Handling Example ===\n");

    // Example 1: Create different types of errors with proper classification
    println!("📋 1. Error Classification Examples\n");

    // Retriable error (network issues, temporary failures)
    let network_error = io::Error::new(io::ErrorKind::TimedOut, "Connection timed out");
    let retriable_error = ToolError::retriable_with_source(
        network_error,
        "Failed to connect to RPC endpoint, should retry",
    );

    println!("🔄 Retriable Error:");
    println!("   Type: {retriable_error}");
    println!("   Is Retriable: {}", retriable_error.is_retriable());
    println!("   Is Rate Limited: {}", retriable_error.is_rate_limited());
    println!(
        "   Serialized: {}",
        serde_json::to_string(&retriable_error)?
    );

    // Permanent error (invalid input, auth failures)
    let auth_error = io::Error::new(io::ErrorKind::PermissionDenied, "Invalid API key");
    let permanent_error = ToolError::permanent_with_source(
        auth_error,
        "Authentication failed - check API key configuration",
    );

    println!("\n🚫 Permanent Error:");
    println!("   Type: {permanent_error}");
    println!("   Is Retriable: {}", permanent_error.is_retriable());
    println!("   Is Rate Limited: {}", permanent_error.is_rate_limited());
    println!(
        "   Serialized: {}",
        serde_json::to_string(&permanent_error)?
    );

    // Rate limited error (with retry delay)
    let rate_error = io::Error::other("Too many requests");
    let rate_limited_error = ToolError::rate_limited_with_source(
        rate_error,
        "API rate limit exceeded - backoff required",
        Some(Duration::from_secs(30)),
    );

    println!("\n⏱️  Rate Limited Error:");
    println!("   Type: {rate_limited_error}");
    println!("   Is Retriable: {}", rate_limited_error.is_retriable());
    println!(
        "   Is Rate Limited: {}",
        rate_limited_error.is_rate_limited()
    );
    println!("   Retry After: {:?}", rate_limited_error.retry_after());
    println!(
        "   Serialized: {}",
        serde_json::to_string(&rate_limited_error)?
    );

    // Invalid input error
    let validation_error = io::Error::new(io::ErrorKind::InvalidInput, "Amount must be positive");
    let input_error =
        ToolError::invalid_input_with_source(validation_error, "Parameter validation failed");

    println!("\n❌ Invalid Input Error:");
    println!("   Type: {input_error}");
    println!("   Is Retriable: {}", input_error.is_retriable());
    println!("   Is Rate Limited: {}", input_error.is_rate_limited());
    println!("   Serialized: {}", serde_json::to_string(&input_error)?);

    // Example 2: Create errors from string messages
    println!("\n📋 2. String-Based Error Creation\n");

    let simple_retriable = ToolError::retriable_string("Network timeout occurred");
    let simple_permanent = ToolError::permanent_string("Invalid wallet address format");
    let simple_rate_limited = ToolError::rate_limited_string("Rate limit exceeded");
    let simple_invalid_input = ToolError::invalid_input_string("Missing required field: amount");

    println!("🔄 String Retriable: {simple_retriable}");
    println!("🚫 String Permanent: {simple_permanent}");
    println!("⏱️  String Rate Limited: {simple_rate_limited}");
    println!("❌ String Invalid Input: {simple_invalid_input}");

    // Example 3: JobResult with structured error data
    println!("\n📋 3. JobResult with Structured Error Data\n");

    let _error_data = serde_json::json!({
        "endpoint": "https://api.example.com/v1/transfer",
        "retry_after_seconds": 60,
        "error_code": "RATE_LIMIT_EXCEEDED",
        "quota_reset_time": "2024-01-01T12:00:00Z"
    });

    let job_failure = JobResult::Failure {
        error: riglr_core::ToolError::rate_limited_string("API rate limit exceeded"),
    };

    println!("📊 JobResult with error data:");
    println!("   {}", serde_json::to_string_pretty(&job_failure)?);

    // Example 4: Error conversion patterns
    println!("\n📋 4. Error Conversion Examples\n");

    // From anyhow::Error (now requires explicit classification)
    let anyhow_error = anyhow::anyhow!("Something went wrong");
    let converted_anyhow =
        ToolError::permanent_string(format!("An unknown error occurred: {anyhow_error}"));
    println!(
        "🔀 From anyhow: {} (retriable: {})",
        converted_anyhow,
        converted_anyhow.is_retriable()
    );

    // From String (now requires explicit classification)
    let string_error = "Database connection failed".to_string();
    let converted_string = ToolError::retriable_string(string_error); // Assuming this is retriable
    println!(
        "🔀 From String: {} (retriable: {})",
        converted_string,
        converted_string.is_retriable()
    );

    // From &str (now requires explicit classification)
    let str_error = "Configuration file not found";
    let converted_str = ToolError::permanent_string(str_error); // This is a permanent config issue
    println!(
        "🔀 From &str: {} (retriable: {})",
        converted_str,
        converted_str.is_retriable()
    );

    // From Box<dyn Error> (now requires explicit classification)
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let converted_boxed =
        ToolError::permanent_with_source(io_error, "A required file was not found");
    println!(
        "🔀 From Box<Error>: {} (retriable: {})",
        converted_boxed,
        converted_boxed.is_retriable()
    );

    // Example 5: Best practices for error handling in tools
    println!("\n📋 5. Tool Error Handling Best Practices\n");

    let tool_result = simulate_tool_operation("valid_input");
    match tool_result {
        Ok(result) => println!("✅ Tool succeeded: {result:?}"),
        Err(e) => {
            println!("❌ Tool failed: {e}");
            if e.is_retriable() {
                if e.is_rate_limited() {
                    if let Some(delay) = e.retry_after() {
                        println!("   🔄 Should retry after: {delay:?}");
                    } else {
                        println!("   🔄 Should retry with exponential backoff");
                    }
                } else {
                    println!("   🔄 Should retry immediately with backoff");
                }
            } else {
                println!("   🚫 Should not retry - permanent failure");
            }
        }
    }

    let tool_result_bad = simulate_tool_operation("invalid_input");
    match tool_result_bad {
        Ok(result) => println!("✅ Tool succeeded: {result:?}"),
        Err(e) => {
            println!("❌ Tool failed: {e}");
            if e.is_retriable() {
                println!("   🔄 Should retry");
            } else {
                println!("   🚫 Should not retry - permanent failure");
            }
        }
    }

    println!("\n🎉 Error handling example completed!");
    println!("\n🔧 Key takeaways:");
    println!("   • Use appropriate error types based on failure category");
    println!("   • Preserve error sources with context for debugging");
    println!("   • Structured errors enable intelligent retry logic");
    println!("   • Error serialization supports distributed processing");
    println!("   • Rate limiting errors can include retry timing information");

    Ok(())
}

/// Simulates a tool operation that can fail in different ways
fn simulate_tool_operation(input: &str) -> Result<String, ToolError> {
    match input {
        "valid_input" => {
            // Simulate a successful operation
            Ok("Operation completed successfully".to_string())
        }
        "network_error" => {
            // Simulate a network timeout (retriable)
            let io_error = io::Error::new(io::ErrorKind::TimedOut, "Connection timed out");
            Err(ToolError::retriable_with_source(
                io_error,
                "Network connection failed",
            ))
        }
        "rate_limited" => {
            // Simulate rate limiting (retriable with delay)
            let rate_error = io::Error::other("Rate limit exceeded");
            Err(ToolError::rate_limited_with_source(
                rate_error,
                "API rate limit hit",
                Some(Duration::from_secs(60)),
            ))
        }
        "invalid_input" => {
            // Simulate invalid input (permanent error)
            Err(ToolError::invalid_input_string("Input validation failed"))
        }
        "auth_error" => {
            // Simulate authentication failure (permanent)
            let auth_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
            Err(ToolError::permanent_with_source(
                auth_error,
                "Authentication failed",
            ))
        }
        _ => {
            // Default case
            Ok(format!("Processed input: {input}"))
        }
    }
}
