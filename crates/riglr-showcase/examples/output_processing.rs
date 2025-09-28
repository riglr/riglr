//! Output processing example
//!
//! This example demonstrates how to use the output processing patterns
//! to create comprehensive processing pipelines for tool outputs.

use anyhow::Result;
use riglr_showcase::processors::{
    utils, ConsoleChannel, DiscordChannel, DistillationProcessor, Html, Json, Markdown,
    MultiFormatProcessor, NotificationRouter, OutputProcessor, ProcessorPipeline, RoutingCondition,
    RoutingRule, Smart, TelegramChannel,
};
use serde_json::json;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 riglr Output Processing Examples");
    println!("===================================\n");

    basic_formatting().await?;
    llm_distillation().await?;
    notification_routing().await?;
    complete_pipeline().await?;
    error_handling().await?;
    multi_format().await?;

    println!("✅ All examples completed successfully!");
    Ok(())
}

async fn basic_formatting() -> Result<()> {
    println!("📄 Example 1: Basic Formatting");
    println!("================================\n");
    // Create some sample tool outputs
    let balance_output = utils::success_output(
        "get_sol_balance",
        json!({
            "address": "11111111111111111111111111111112",
            "balance_lamports": 1_500_000_000,
            "balance_sol": 1.5
        }),
    );

    let swap_output = utils::success_output(
        "swap_tokens",
        json!({
            "from_token": "SOL",
            "to_token": "USDC",
            "amount_in": 1.0,
            "amount_out": 180.5,
            "transaction_signature": "5J7X8gD2F9K3L4M5N6P7Q8R9S0T1U2V3W4X5Y6Z7A8B9C0D1E2F3G4H5I6J7K8L9"
        }),
    );

    // Markdown formatting
    println!("  📝 Markdown Formatting:");
    let markdown_formatter = Markdown::new();
    let markdown_result = markdown_formatter.process(balance_output.clone()).await?;
    if let Some(markdown) = markdown_result.processed_result.get("markdown") {
        // Example code expects formatter to produce string values
        let Some(markdown_content) = markdown.as_str() else {
            eprintln!("Warning: markdown field should contain string value");
            return Ok(());
        };
        println!("{markdown_content}");
    }

    // HTML formatting
    println!("  🌐 HTML Formatting:");
    let html_formatter = Html::new().without_styles();
    let html_result = html_formatter.process(swap_output.clone()).await?;
    if let Some(html) = html_result.processed_result.get("html") {
        // Example code expects formatter to produce string values
        let Some(html_content) = html.as_str() else {
            eprintln!("Warning: html field should contain string value");
            return Ok(());
        };
        // Show just a snippet since HTML can be long
        let snippet = if html_content.len() > 200 {
            format!("{}...", &html_content[..200])
        } else {
            html_content.to_string()
        };
        println!("{snippet}");
    }

    // JSON formatting with field mapping
    println!("  📊 JSON Formatting (with field mapping):");
    let json_formatter = Json::new()
        .compact()
        .with_field_mapping("balance_sol", "solana_balance")
        .with_field_mapping("balance_lamports", "lamports_balance");

    let json_result = json_formatter.process(balance_output).await?;
    if let Some(structured) = json_result.processed_result.get("structured") {
        println!("{}", serde_json::to_string_pretty(structured)?);
    }

    Ok(())
}

async fn llm_distillation() -> Result<()> {
    println!("\n📈 Example 2: LLM Distillation");
    println!("================================");
    // Create a complex tool output that benefits from distillation
    let complex_output = utils::success_output(
        "analyze_defi_position",
        json!({
            "protocol": "Jupiter",
            "pool_address": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
            "total_value_locked": "1,234,567.89",
            "apy": 15.67,
            "impermanent_loss_risk": "Medium",
            "fees_24h": 45.23,
            "volume_24h": "987,654.32",
            "your_position": {
                "liquidity_provided": 1000.0,
                "current_value": 1089.45,
                "profit_loss": 89.45,
                "fees_earned": 23.12
            },
            "risk_factors": [
                "Smart contract risk",
                "Impermanent loss",
                "Liquidity risk"
            ]
        }),
    );

    // Use different distillation processors
    println!("  🤖 GPT-4 Distillation:");
    let gpt_distiller = DistillationProcessor::new("gpt-4o-mini");
    let gpt_result = gpt_distiller.process(complex_output.clone()).await?;
    if let Some(summary) = gpt_result.summary.as_ref() {
        println!("    Summary: {summary}");
    }

    println!("  🧠 Claude Distillation:");
    let claude_distiller = DistillationProcessor::new("claude-3-haiku");
    let claude_result = claude_distiller.process(complex_output.clone()).await?;
    if let Some(summary) = claude_result.summary.as_ref() {
        println!("    Summary: {summary}");
    }

    // Smart distiller that chooses the right model
    println!("  🎯 Smart Distillation (auto-selects model):");
    let smart_distiller = Smart::new();
    let smart_result = smart_distiller.process(complex_output).await?;
    if let Some(summary) = smart_result.summary.as_ref() {
        println!("    Summary: {summary}");
    }

    Ok(())
}

async fn notification_routing() -> Result<()> {
    println!("\n📢 Example 3: Notification Routing");
    println!("====================================");
    // Set up notification channels
    let discord_channel = DiscordChannel::new("https://discord.com/api/webhooks/dummy")
        .with_identity("RiglrBot", Some("https://example.com/avatar.png"));

    let telegram_channel = TelegramChannel::new("bot_token", "chat_id");
    let console_channel = ConsoleChannel::new();

    // Create notification router with routing rules
    let router = NotificationRouter::new()
        .add_channel("discord", discord_channel)
        .add_channel("telegram", telegram_channel)
        .add_channel("console", console_channel)
        .set_default_channel("console")
        .add_routing_rule(RoutingRule::new(
            "trading_alerts",
            RoutingCondition::ToolNameContains("swap".to_string()),
            vec!["discord".to_string(), "telegram".to_string()],
        ))
        .add_routing_rule(RoutingRule::new(
            "error_alerts",
            RoutingCondition::OnError,
            vec!["discord".to_string(), "console".to_string()],
        ))
        .add_routing_rule(RoutingRule::new(
            "high_value_alerts",
            RoutingCondition::And(vec![
                RoutingCondition::OnSuccess,
                RoutingCondition::ToolNameContains("trading".to_string()),
            ]),
            vec!["discord".to_string()],
        ));

    // Test different outputs with routing
    let trading_success = utils::success_output(
        "swap_tokens",
        json!({"from": "SOL", "to": "USDC", "amount": 100}),
    );

    let balance_check = utils::success_output("get_balance", json!({"balance": "5.5 SOL"}));

    let error_output = utils::error_output("trading_bot", "Insufficient balance for trade");

    println!("  📤 Processing trading success (should route to Discord + Telegram):");
    let result1 = router.process(trading_success).await?;
    println!(
        "    Routes used: {:?}",
        result1
            .processed_result
            .get("routes_used")
            .unwrap_or(&serde_json::Value::Null)
    );
    println!(
        "    Notifications sent: {}",
        result1
            .processed_result
            .get("total_notifications")
            .unwrap_or(&serde_json::Value::Null)
    );

    println!("  📤 Processing balance check (should route to Console only):");
    let result2 = router.process(balance_check).await?;
    println!(
        "    Routes used: {:?}",
        result2
            .processed_result
            .get("routes_used")
            .unwrap_or(&serde_json::Value::Null)
    );

    println!("  📤 Processing error (should route to Discord + Console):");
    let result3 = router.process(error_output).await?;
    println!(
        "    Routes used: {:?}",
        result3
            .processed_result
            .get("routes_used")
            .unwrap_or(&serde_json::Value::Null)
    );

    Ok(())
}

async fn complete_pipeline() -> Result<()> {
    println!("\n🔄 Example 4: Complete Processing Pipeline");
    println!("==========================================");
    // Create a comprehensive processing pipeline
    let pipeline = ProcessorPipeline::new()
        .add_processor(DistillationProcessor::new("gpt-4o-mini")) // First, distill the output
        .add_processor(Markdown::new()) // Then format as markdown
        .add_processor(
            NotificationRouter::new() // Finally, send notifications
                .add_channel("console", ConsoleChannel::new())
                .set_default_channel("console"),
        );

    // Process a complex DeFi operation
    let start_time = SystemTime::now();
    let defi_output = utils::with_timing(
        utils::with_metadata(
            utils::success_output(
                "provide_liquidity",
                json!({
                    "pool": "SOL/USDC",
                    "amount_sol": 10.0,
                    "amount_usdc": 1800.0,
                    "lp_tokens_received": 42.5,
                    "transaction_hash": "abc123def456",
                    "estimated_apy": 12.5,
                    "fees_tier": "0.3%"
                }),
            ),
            "user_id",
            "user_12345",
        ),
        start_time,
    );

    println!("  🔄 Processing through complete pipeline:");
    println!("    1. LLM Distillation → 2. Markdown Formatting → 3. Notification Routing");

    let final_result = pipeline.process(defi_output).await?;

    println!("    ✅ Pipeline completed!");
    println!("    📊 Pipeline info: {:?}", pipeline.info());
    if let Some(summary) = final_result.summary.as_ref() {
        println!("    📝 Final summary: {summary}");
    }

    Ok(())
}

async fn error_handling() -> Result<()> {
    println!("\n❌ Example 5: Error Handling");
    println!("===============================");
    // Test error handling with different error types
    let network_error =
        utils::error_output("get_price", "Connection timeout while fetching price data");
    let auth_error = utils::error_output("place_order", "Unauthorized: Invalid API key");
    let not_found_error =
        utils::error_output("get_transaction", "Transaction not found: invalid hash");

    println!("  🚨 Testing error message cleaning:");

    // Create an error-aware processor pipeline
    let error_pipeline = ProcessorPipeline::new()
        .add_processor(DistillationProcessor::new("gpt-4o-mini"))
        .add_processor(Markdown::new())
        .add_processor(
            NotificationRouter::new()
                .add_channel("console", ConsoleChannel::new())
                .set_default_channel("console")
                .add_routing_rule(RoutingRule::new(
                    "all_errors",
                    RoutingCondition::OnError,
                    vec!["console".to_string()],
                )),
        );

    let errors = vec![
        ("Network Error", network_error),
        ("Auth Error", auth_error),
        ("Not Found Error", not_found_error),
    ];

    for (error_type, error_output) in errors {
        // Example code expects error output to have error field
        let Some(error_msg) = error_output.error.as_ref() else {
            eprintln!("Warning: error output should have error field");
            continue;
        };
        println!("    {error_type} - Original: {error_msg}");
        let processed = error_pipeline.process(error_output.clone()).await?;
        if let Some(summary) = processed.summary.as_ref() {
            println!("    {error_type} - User-friendly: {summary}");
        }

        // Show user-friendly error message
        let friendly = utils::user_friendly_error(&error_output);
        println!("    {error_type} - Utility function: {friendly}");
        println!();
    }

    Ok(())
}

async fn multi_format() -> Result<()> {
    println!("\n📋 Example 6: Multi-format Output");
    println!("====================================");
    // Create a processor that outputs multiple formats simultaneously
    let multi_processor = MultiFormatProcessor::standard_formats();

    let trading_result = utils::with_timing(
        utils::success_output(
            "arbitrage_opportunity",
            json!({
                "path": ["SOL", "USDC", "USDT", "SOL"],
                "exchanges": ["Jupiter", "Raydium", "Orca"],
                "profit_potential": 0.15, // 0.15%
                "required_capital": 1000.0,
                "estimated_profit": 1.50,
                "execution_time_estimate": "15 seconds",
                "risk_level": "Low"
            }),
        ),
        SystemTime::now(),
    );

    println!("  📋 Generating multiple output formats:");
    let multi_result = multi_processor.process(trading_result).await?;

    println!("    Available formats in result:");
    if let serde_json::Value::Object(ref obj) = multi_result.processed_result {
        for key in obj.keys() {
            println!("      - {key}");
        }
    }

    println!(
        "    Summary: {}",
        multi_result.summary.as_deref().unwrap_or("No summary")
    );

    // Show a snippet of each format
    if let Some(markdown) = multi_result.processed_result.get("markdown") {
        // Example code expects formatter to produce string values
        let Some(content) = markdown.as_str() else {
            eprintln!("Warning: markdown field should contain string value");
            return Ok(());
        };
        let lines: Vec<&str> = content.lines().take(3).collect();
        println!("    Markdown preview: {} ...", lines.join(" "));
    }

    if let Some(html) = multi_result.processed_result.get("html") {
        // Example code expects formatter to produce string values
        let Some(content) = html.as_str() else {
            eprintln!("Warning: html field should contain string value");
            return Ok(());
        };
        let snippet = if content.len() > 100 {
            &content[..100]
        } else {
            content
        };
        println!("    HTML preview: {snippet}...");
    }

    if let Some(json_str) = multi_result.processed_result.get("json") {
        // Example code expects formatter to produce string values
        let Some(content) = json_str.as_str() else {
            eprintln!("Warning: json field should contain string value");
            return Ok(());
        };
        let lines: Vec<&str> = content.lines().take(5).collect();
        println!("    JSON preview:\n{}", lines.join("\n"));
    }

    Ok(())
}

// Helper function to create realistic timing

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_workflow() {
        // Test that all processors work together
        let pipeline = ProcessorPipeline::new()
            .add_processor(DistillationProcessor::new("gpt-4o-mini"))
            .add_processor(Markdown::new())
            .add_processor(
                NotificationRouter::new()
                    .add_channel("console", ConsoleChannel::new())
                    .set_default_channel("console"),
            );

        let test_output =
            utils::success_output("integration_test", json!({"test": true, "value": 42}));

        let result = pipeline.process(test_output).await.unwrap();

        // Verify all processors contributed
        assert!(result.summary.is_some()); // From distillation
        assert!(matches!(
            result.format,
            riglr_showcase::processors::OutputFormat::Json
        )); // From notification router
        assert!(result
            .processed_result
            .get("notification_results")
            .is_some()); // From router
    }

    #[tokio::test]
    async fn test_error_propagation() {
        // Test that errors are handled gracefully throughout the pipeline
        let pipeline = ProcessorPipeline::new()
            .add_processor(DistillationProcessor::new("gpt-4o-mini"))
            .add_processor(Markdown::new());

        let error_output = utils::error_output("test_error", "Simulated failure");
        let result = pipeline.process(error_output).await.unwrap();

        assert!(result.summary.is_some()); // Should have error summary
        assert!(!result.original.success); // Original error state preserved
    }
}
