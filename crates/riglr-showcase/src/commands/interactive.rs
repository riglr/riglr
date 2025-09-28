//! Interactive chat mode commands.

use anyhow::Result;
use colored::Colorize;
use core::fmt::Write;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_core::provider::ApplicationContext;
use std::{collections::HashMap, sync::Arc};
// Temporarily disabled due to compilation issues
// use riglr_core::{Agent, ModelBuilder};
// use riglr_solana_tools::{get_sol_balance, get_jupiter_quote};
// use riglr_evm_tools::{get_eth_balance, get_erc20_balance}; // Temporarily disabled
// use riglr_web_tools::{search_tokens, get_crypto_news, search_tweets}; // Temporarily disabled
// use tracing::{info, warn}; // Temporarily disabled

/// Chat session context to maintain conversation state
#[derive(Debug)]
struct ChatContext {
    conversation_history: Vec<(String, String)>, // (user_input, agent_response)
    session_id: String,
    user_preferences: HashMap<String, String>,
}
impl Default for ChatContext {
    fn default() -> Self {
        Self {
            conversation_history: Vec::default(),
            session_id: format!("chat_{}", chrono::Utc::now().timestamp()),
            user_preferences: HashMap::default(),
        }
    }
}

impl ChatContext {
    fn add_exchange(&mut self, user_input: String, agent_response: String) {
        self.conversation_history.push((user_input, agent_response));
        // Keep only last 10 exchanges to avoid memory bloat
        if self.conversation_history.len() > 10 {
            self.conversation_history.remove(0);
        }
    }
}

/// Run interactive chat mode.
///
/// # Errors
///
/// This function will return an error if:
/// - Progress bar template formatting fails
/// - User input interaction fails
/// - Process user input fails during agent response generation
/// - Quick actions interaction fails
pub fn run_chat(context: &Arc<ApplicationContext>) -> Result<()> {
    println!("{}", "🤖 Interactive Riglr Agent Chat".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());

    println!(
        "\n{}",
        "Welcome to interactive chat with a riglr-powered AI agent!".cyan()
    );
    println!(
        "{}",
        "This agent has access to blockchain data across multiple chains,".dimmed()
    );
    println!(
        "{}",
        "market intelligence, social sentiment, and graph memory.".dimmed()
    );

    // Initialize chat context
    let mut chat_context = ChatContext::default();

    // Setup agent with tools
    println!(
        "\n{}",
        "🔧 Setting up AI agent with riglr tools...".yellow()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.set_message("Initializing agent...");

    // Create a mock agent for demonstration (in real implementation, this would use rig-core)
    let _agent_description = create_agent_description(context);

    pb.finish_and_clear();

    println!("{}", "✅ Agent ready! Available capabilities:".green());
    println!("   🌟 Solana blockchain queries (balances, swaps, token info)");
    println!("   ⚡ Ethereum & EVM chain analysis (balances, DeFi protocols)");
    println!("   🌐 Web intelligence (news, social sentiment, market data)");
    println!("   🧠 Graph memory (relationship analysis, pattern recognition)");

    // Interactive chat loop
    loop {
        println!("\n{}", "─".repeat(50).dimmed());

        // Get user input
        let user_input: String = Input::new()
            .with_prompt(format!("{}", "You".bright_cyan()))
            .interact_text()?;

        if user_input.trim().is_empty() {
            continue;
        }

        // Check for exit commands
        if user_input.trim().to_lowercase() == "exit" || user_input.trim().to_lowercase() == "quit"
        {
            println!(
                "\n{}",
                "Goodbye! Thanks for chatting with the riglr agent!".bright_green()
            );
            break;
        }

        // Check for help
        if user_input.trim().to_lowercase() == "help" {
            show_help();
            continue;
        }

        // Check for context commands
        if user_input.trim().to_lowercase() == "context" {
            show_context(&chat_context);
            continue;
        }

        if user_input.trim().to_lowercase() == "clear" {
            chat_context.conversation_history.clear();
            println!("{}", "🧹 Conversation history cleared!".yellow());
            continue;
        }

        // Process user input with the agent
        pb.set_message("Agent is thinking...");
        pb.reset();

        let agent_response = process_user_input(context, &user_input, &chat_context)?;

        pb.finish_and_clear();

        // Display agent response
        println!("{}: {}", "Agent".bright_green().bold(), agent_response);

        // Add to conversation history
        chat_context.add_exchange(user_input, agent_response);

        // Offer quick actions
        if should_offer_quick_actions(&chat_context) {
            offer_quick_actions(context, &mut chat_context)?;
        }
    }

    // Chat summary
    println!("\n{}", "📊 Chat Session Summary".blue().bold());
    println!("   Session ID: {}", chat_context.session_id.dimmed());
    println!("   Exchanges: {}", chat_context.conversation_history.len());
    println!(
        "   Preferences set: {}",
        chat_context.user_preferences.len()
    );

    Ok(())
}
fn process_user_input(
    context: &ApplicationContext,
    input: &str,
    chat_context: &ChatContext,
) -> Result<String> {
    let input_lower = input.to_lowercase();

    // Intent detection and routing
    if input_lower.contains("balance") || input_lower.contains("wallet") {
        return handle_balance_query(context, input);
    } else if input_lower.contains("swap") || input_lower.contains("trade") {
        return Ok(handle_swap_query(context, input));
    } else if input_lower.contains("news") || input_lower.contains("latest") {
        return Ok(handle_news_query(context, input));
    } else if input_lower.contains("twitter") || input_lower.contains("sentiment") {
        return Ok(handle_social_query(context, input));
    } else if input_lower.contains("token")
        && (input_lower.contains("info") || input_lower.contains("price"))
    {
        return handle_token_info_query(context, input);
    } else if input_lower.contains("cross") && input_lower.contains("chain") {
        return Ok(handle_cross_chain_query(context, input));
    }
    // General conversational response
    Ok(generate_conversational_response(input, chat_context))
}
fn handle_balance_query(_context: &ApplicationContext, input: &str) -> Result<String> {
    // Extract wallet address from input (simplified extraction)
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut wallet_address = None;

    for word in &words {
        if word.len() > 20 && (word.starts_with("0x") || word.chars().all(char::is_alphanumeric)) {
            wallet_address = Some(*word);
            break;
        }
    }

    wallet_address.map_or_else(|| Ok("I'd be happy to check a wallet balance! Please provide a wallet address. For example:\n'Check balance for 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'".to_string()), |address| {
        let mut response = format!(
            "🔍 Checking balance for wallet: {}\n",
            address.bright_cyan()
        );

        // Check Solana if it looks like a Solana address
        if !address.starts_with("0x") {
            // This would use actual Solana client
            response.push_str("🌟 Solana: Checking SOL balance...\n");
            response.push_str("   (Would show actual balance here)\n");
        }

        // Check Ethereum if it looks like an Ethereum address
        if address.starts_with("0x") {
            response.push_str("⚡ Ethereum: Checking ETH balance...\n");
            response.push_str("   (Would show actual balance here)\n");
        }

        response.push_str("\n💡 I can also check specific token balances! Just ask 'check USDC balance for [address]'");
        Ok(response)
    })
}
fn handle_swap_query(_context: &ApplicationContext, _input: &str) -> String {
    "🔄 I can help you with swap information!\n\n\
        I can provide quotes for:\n\
        • 🌟 Solana: Jupiter DEX quotes\n\
        • ⚡ Ethereum: Uniswap quotes\n\
        • 🌐 Cross-chain: Bridge opportunities\n\n\
        Try asking: 'Get quote for 1 SOL to USDC' or 'What's the best WETH to USDC rate?'"
        .to_string()
}
fn handle_news_query(_context: &ApplicationContext, input: &str) -> String {
    let mut response = String::default();
    response.push_str("📰 Fetching latest crypto news...\n\n");

    // Extract search term if any
    let search_term = if input.to_lowercase().contains("about") {
        input.split("about").nth(1).map(str::trim)
    } else {
        None
    };

    if let Some(term) = search_term {
        let _ = writeln!(response, "🔍 Searching news about: {}", term.bright_cyan());
    }

    response.push_str("Here are the latest crypto headlines:\n");
    response.push_str("1. Bitcoin reaches new all-time high amid institutional adoption\n");
    response.push_str("2. Ethereum 2.0 staking rewards hit record levels\n");
    response.push_str("3. DeFi protocols see surge in cross-chain activity\n");
    response.push_str("\n💡 I can search for news about specific tokens or topics!");

    response
}
fn handle_social_query(context: &ApplicationContext, _input: &str) -> String {
    if context.config.providers.twitter_bearer_token.is_some() {
        return "🐦 Social Sentiment Analysis Available!\n\n\
            I can analyze Twitter sentiment for:\n\
            • Specific cryptocurrencies\n\
            • Market trends and events\n\
            • DeFi protocols\n\
            • NFT collections\n\n\
            Try asking: 'What's the sentiment around Solana?' or 'Twitter buzz about DeFi'"
            .to_string();
    }
    "🐦 Social sentiment analysis is available but requires Twitter API configuration.\n\
        Set TWITTER_BEARER_TOKEN to enable sentiment analysis features."
        .to_string()
}
fn handle_token_info_query(_context: &ApplicationContext, input: &str) -> Result<String> {
    // Extract token symbol
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut token_symbol = None;

    for (i, word) in words.iter().enumerate() {
        // Look for patterns like "SOL token" or "about SOL" but not generic words after "token"
        if (word.to_lowercase() == "token" || word.to_lowercase() == "coin")
            && i.saturating_add(1) < words.len()
        {
            let Some(next_word) = words.get(i.saturating_add(1)) else {
                continue;
            };
            // Only consider it a token if it looks like a token symbol (short, alphanumeric, uppercase)
            if next_word.len() <= 6
                && next_word.chars().all(char::is_alphanumeric)
                && (next_word.to_uppercase() == *next_word
                    || next_word.chars().all(char::is_uppercase))
                && !next_word.eq_ignore_ascii_case("information")
                && !next_word.eq_ignore_ascii_case("details")
            {
                token_symbol = Some(next_word);
                break;
            }
        }
        // Look for common token patterns - all caps, short, alphabetic
        if word.len() <= 6
            && word.chars().all(char::is_alphabetic)
            && word.to_uppercase() == *word
            && word.len() >= 2
        // Minimum 2 characters for token symbols
        {
            token_symbol = Some(word);
            break;
        }
    }

    token_symbol.map_or_else(|| Ok("I can provide detailed token information! Please specify a token symbol.\nFor example: 'Tell me about SOL token' or 'What's the USDC market cap?'".to_string()), |token| Ok(format!(
            "🪙 Token Analysis for {}\n\n\
            📊 Market Data:\n\
            • Price: (Would fetch live price)\n\
            • Market Cap: (Would fetch market cap)\n\
            • 24h Volume: (Would fetch volume)\n\
            • Price Change: (Would fetch price change)\n\n\
            ⛓️ Cross-Chain Presence:\n\
            • Available on multiple DEXs\n\
            • Bridge liquidity analysis\n\
            • Yield farming opportunities\n\n\
            💡 Ask me about specific aspects like 'What's the SOL price trend?'",
            token.bright_cyan()
        )))
}
fn handle_cross_chain_query(_context: &ApplicationContext, _input: &str) -> String {
    "🌐 Cross-Chain Analysis Available!\n\n\
        I can analyze:\n\
        ⛓️ Multi-chain token presence\n\
        💱 Arbitrage opportunities\n\
        🌉 Bridge costs and routes\n\
        📊 Liquidity across chains\n\
        💰 Yield farming comparison\n\n\
        Supported chains:\n\
        • 🌟 Solana (Jupiter, Orca, Raydium)\n\
        • ⚡ Ethereum (Uniswap, Compound, Aave)\n\
        • 🔵 Polygon, Arbitrum, Base\n\n\
        Try: 'Compare USDC liquidity across chains' or 'Find arbitrage opportunities for WETH'"
        .to_string()
}
fn generate_conversational_response(input: &str, context: &ChatContext) -> String {
    let responses = [format!("I understand you're asking about: '{input}'\n\nAs a riglr-powered agent, I can help you with blockchain analysis, DeFi operations, and market intelligence. What specific information would you like?"),
        format!("That's an interesting question! I have access to multi-chain data and can help with:\n• Wallet and token analysis\n• Market sentiment and news\n• Cross-chain opportunities\n• DeFi protocol information\n\nHow can I assist you with '{input}'?"),
        format!("I'm here to help with blockchain and crypto analysis! For '{input}', I can provide:\n• Real-time market data\n• On-chain analytics\n• Social sentiment analysis\n• Cross-chain insights\n\nWhat would you like to explore?")];

    // Simple selection based on conversation length
    let index = if responses.is_empty() {
        0
    } else {
        let len = responses.len();
        context
            .conversation_history
            .len()
            .checked_rem(len)
            .unwrap_or(0)
    };
    responses.get(index).map_or_else(
        || String::from("I'm here to help with your blockchain needs!"),
        Clone::clone,
    )
}

fn show_help() {
    println!("\n{}", "🤖 Riglr Agent Help".bright_blue().bold());
    println!("{}", "=".repeat(30).blue());

    println!("\n{}", "Available Commands:".green().bold());
    println!("• {} - Show this help message", "help".bright_cyan());
    println!("• {} - Show conversation context", "context".bright_cyan());
    println!("• {} - Clear conversation history", "clear".bright_cyan());
    println!(
        "• {} or {} - Exit chat",
        "exit".bright_cyan(),
        "quit".bright_cyan()
    );

    println!("\n{}", "Example Queries:".green().bold());
    println!("💰 Balance: 'Check SOL balance for [address]'");
    println!("🔄 Swaps: 'Get quote for 1 ETH to USDC'");
    println!("📰 News: 'Latest news about Bitcoin'");
    println!("🐦 Social: 'What's the sentiment around DeFi?'");
    println!("🪙 Tokens: 'Tell me about USDC token'");
    println!("🌐 Cross-chain: 'Compare liquidity across chains'");

    println!(
        "\n{}",
        "I can understand natural language! Just ask me anything about crypto and blockchain."
            .dimmed()
    );
}

fn show_context(context: &ChatContext) {
    println!("\n{}", "💬 Chat Context".bright_blue().bold());
    println!("{}", "=".repeat(30).blue());

    println!("Session ID: {}", context.session_id.dimmed());
    println!(
        "Conversation History: {} exchanges",
        context.conversation_history.len()
    );
    println!("Preferences: {} settings", context.user_preferences.len());

    if !context.conversation_history.is_empty() {
        println!("\n{}", "Recent Exchanges:".green());
        #[expect(clippy::pattern_type_mismatch)]
        for (i, (user, _agent)) in context
            .conversation_history
            .iter()
            .rev()
            .take(3)
            .enumerate()
        {
            let preview = if user.len() > 50 {
                format!("{}...", &user[..50])
            } else {
                user.clone()
            };
            println!(
                "{}. You: {}",
                context.conversation_history.len().saturating_sub(i),
                preview.dimmed()
            );
        }
    }
}
const fn should_offer_quick_actions(context: &ChatContext) -> bool {
    // Offer quick actions every few exchanges
    context.conversation_history.len() % 3 == 2
}
fn offer_quick_actions(
    _context: &ApplicationContext,
    _chat_context: &mut ChatContext,
) -> Result<()> {
    println!("\n{}", "⚡ Quick Actions".yellow().bold());

    let options = vec![
        "Continue chatting",
        "Get market summary",
        "Check trending tokens",
        "Run full cross-chain analysis",
        "Switch to demo mode",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        1 => {
            println!("📊 Here's a quick market summary:");
            println!("• Bitcoin: $45,123 (+2.3%)");
            println!("• Ethereum: $2,891 (+1.8%)");
            println!("• Solana: $98.45 (+5.2%)");
            println!("• Overall market sentiment: Bullish");
        }
        2 => {
            println!("🚀 Trending tokens right now:");
            println!("1. $PEPE - Meme coin rally continues");
            println!("2. $ARB - Arbitrum ecosystem growth");
            println!("3. $MATIC - Polygon updates driving interest");
        }
        3 => {
            println!("🌐 This would launch a comprehensive cross-chain analysis...");
            println!("Analyzing liquidity, arbitrage opportunities, and yield farming across all supported chains.");
        }
        4 => {
            println!("🎮 Switching to demo mode would let you explore individual riglr tools.");
            println!("Use the main menu to access Solana, EVM, Web, or Graph demos.");
        }
        _ => {}
    }

    Ok(())
}
fn create_agent_description(context: &ApplicationContext) -> String {
    let mut capabilities = Vec::new();

    capabilities.push("🌟 Solana blockchain queries".to_string());
    capabilities.push("⚡ EVM-compatible chains (Ethereum, Polygon, etc.)".to_string());
    capabilities.push("🌐 Web intelligence and market data".to_string());

    if context.config.providers.twitter_bearer_token.is_some() {
        capabilities.push("🐦 Twitter sentiment analysis".to_string());
    }

    if context.config.providers.exa_api_key.is_some() {
        capabilities.push("🔍 Advanced web search".to_string());
    }

    capabilities.push("🧠 Graph memory for relationship analysis".to_string());

    format!(
        "Multi-chain AI agent with access to:\n{}\n\nReady to help with blockchain analysis and DeFi operations!",
        capabilities.join("\n")
    )
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;

    fn create_test_context() -> ApplicationContext {
        use riglr_config::*;

        let config = Config::builder()
            .app(AppConfig::default())
            .database(DatabaseConfig::default())
            .network(NetworkConfig::default())
            .providers(ProvidersConfig::default())
            .features(FeaturesConfig::default())
            .build()
            .expect("Test config should be valid");

        ApplicationContext::from_config(&config)
    }

    #[test]
    fn test_chat_context_default() {
        let context = ChatContext::default();
        assert!(!context.session_id.is_empty());
        assert!(context.session_id.starts_with("chat_"));
        assert!(context.user_preferences.is_empty());
        assert!(context.conversation_history.is_empty());
    }

    #[test]
    fn test_chat_context_add_exchange_single() {
        let mut context = ChatContext::default();
        context.add_exchange("test input".to_string(), "test response".to_string());

        assert_eq!(context.conversation_history.len(), 1);
        assert_eq!(
            context
                .conversation_history
                .first()
                .expect("Should have at least one entry")
                .0,
            "test input"
        );
        assert_eq!(
            context
                .conversation_history
                .first()
                .expect("Should have at least one entry")
                .1,
            "test response"
        );
    }

    #[test]
    fn test_chat_context_add_exchange_limit_history() {
        let mut context = ChatContext::default();

        // Add 12 exchanges (exceeds the 10 limit)
        for i in 0..12 {
            context.add_exchange(format!("input {i}"), format!("response {i}"));
        }

        // Should only keep the last 10
        assert_eq!(context.conversation_history.len(), 10);
        assert_eq!(
            context
                .conversation_history
                .first()
                .expect("Should have at least one entry")
                .0,
            "input 2"
        ); // First exchange should be from index 2
        assert_eq!(
            context
                .conversation_history
                .get(9)
                .expect("Should have entry at index 9")
                .0,
            "input 11"
        ); // Last exchange should be from index 11
    }

    #[test]
    fn test_handle_balance_query_with_ethereum_address() {
        let context = create_test_context();
        let input = "Check balance for 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

        let result = handle_balance_query(&context, input)
            .expect("Balance query should succeed with valid Ethereum address");

        assert!(result.contains("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
        assert!(result.contains("⚡ Ethereum"));
        assert!(!result.contains("🌟 Solana"));
    }

    #[test]
    fn test_handle_balance_query_with_solana_address() {
        let context = create_test_context();
        let input = "Check balance for 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";

        let result = handle_balance_query(&context, input)
            .expect("Balance query should succeed with valid Solana address");

        assert!(result.contains("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
        assert!(result.contains("🌟 Solana"));
        assert!(!result.contains("⚡ Ethereum"));
    }

    #[test]
    fn test_handle_balance_query_no_address() {
        let context = create_test_context();
        let input = "Check my balance";

        let result = handle_balance_query(&context, input)
            .expect("Balance query should succeed and request wallet address when none provided");

        assert!(result.contains("Please provide a wallet address"));
        assert!(result.contains("For example"));
    }

    #[test]
    fn test_handle_swap_query() {
        let context = create_test_context();
        let input = "Get quote for 1 SOL to USDC";

        let result = handle_swap_query(&context, input);

        assert!(result.contains("🔄 I can help you with swap information"));
        assert!(result.contains("Jupiter DEX"));
        assert!(result.contains("Uniswap"));
    }

    #[test]
    fn test_handle_news_query_general() {
        let context = create_test_context();
        let input = "Latest news";

        let result = handle_news_query(&context, input);

        assert!(result.contains("📰 Fetching latest crypto news"));
        assert!(result.contains("Bitcoin reaches new all-time high"));
        assert!(result.contains("Ethereum 2.0 staking"));
    }

    #[test]
    fn test_handle_news_query_with_search_term() {
        let context = create_test_context();
        let input = "Latest news about Bitcoin";

        let result = handle_news_query(&context, input);

        assert!(result.contains("🔍 Searching news about: Bitcoin"));
        assert!(result.contains("📰 Fetching latest crypto news"));
    }

    #[test]
    fn test_handle_social_query_with_twitter_token() {
        let mut context = create_test_context();
        context.config.providers.twitter_bearer_token = Some("test_token".to_string());
        let input = "What's the sentiment around Solana?";

        let result = handle_social_query(&context, input);

        assert!(result.contains("🐦 Social Sentiment Analysis Available"));
        assert!(result.contains("Twitter sentiment"));
    }

    #[test]
    fn test_handle_social_query_without_twitter_token() {
        let context = create_test_context();
        let input = "What's the sentiment around Solana?";

        let result = handle_social_query(&context, input);

        assert!(result.contains("requires Twitter API configuration"));
        assert!(result.contains("TWITTER_BEARER_TOKEN"));
    }

    #[test]
    fn test_handle_token_info_query_with_token() {
        let context = create_test_context();
        let input = "Tell me about SOL token";

        let result = handle_token_info_query(&context, input)
            .expect("Token info query should succeed with valid token symbol");

        assert!(result.contains("🪙 Token Analysis for SOL"));
        assert!(result.contains("📊 Market Data"));
        assert!(result.contains("⛓️ Cross-Chain Presence"));
    }

    #[test]
    fn test_handle_token_info_query_with_uppercase_token() {
        let context = create_test_context();
        let input = "What's the USDC price?";

        let result = handle_token_info_query(&context, input)
            .expect("Token info query should succeed with uppercase token symbol");

        assert!(result.contains("🪙 Token Analysis for USDC"));
    }

    #[test]
    fn test_handle_token_info_query_no_token() {
        let context = create_test_context();
        let input = "Tell me about token information";

        let result = handle_token_info_query(&context, input)
            .expect("Token info query should succeed and request token symbol when none provided");

        assert!(result.contains("Please specify a token symbol"));
        assert!(result.contains("For example"));
    }

    #[test]
    fn test_handle_cross_chain_query() {
        let context = create_test_context();
        let input = "Compare liquidity across chains";

        let result = handle_cross_chain_query(&context, input);

        assert!(result.contains("🌐 Cross-Chain Analysis Available"));
        assert!(result.contains("Multi-chain token presence"));
        assert!(result.contains("🌟 Solana"));
        assert!(result.contains("⚡ Ethereum"));
    }

    #[test]
    fn test_process_user_input_balance_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "Check my wallet balance";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for balance intent");

        assert!(result.contains("Please provide a wallet address"));
    }

    #[test]
    fn test_process_user_input_swap_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "I want to swap some tokens";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for swap intent");

        assert!(result.contains("🔄 I can help you with swap information"));
    }

    #[test]
    fn test_process_user_input_news_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "Show me the latest crypto news";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for news intent");

        assert!(result.contains("📰 Fetching latest crypto news"));
    }

    #[test]
    fn test_process_user_input_twitter_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "What's the Twitter sentiment about Bitcoin?";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for Twitter sentiment intent");

        assert!(result.contains("requires Twitter API configuration"));
    }

    #[test]
    fn test_process_user_input_token_info_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "Give me token info for SOL";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for token info intent");

        assert!(result.contains("🪙 Token Analysis for SOL"));
    }

    #[test]
    fn test_process_user_input_cross_chain_intent() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "Show me cross chain opportunities";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for cross-chain intent");

        assert!(result.contains("🌐 Cross-Chain Analysis Available"));
    }

    #[test]
    fn test_process_user_input_general_conversation() {
        let app_context = create_test_context();
        let chat_context = ChatContext::default();
        let input = "Hello, how are you?";

        let result = process_user_input(&app_context, input, &chat_context)
            .expect("Process user input should succeed for general conversation");

        assert!(result.contains("Hello, how are you?"));
        assert!(result.contains("riglr-powered agent"));
    }

    #[test]
    fn test_generate_conversational_response_cycles_responses() {
        let mut context = ChatContext::default();
        let input = "test input";

        let response1 = generate_conversational_response(input, &context);
        context.add_exchange("test".to_string(), "test".to_string());
        let response2 = generate_conversational_response(input, &context);
        context.add_exchange("test".to_string(), "test".to_string());
        let response3 = generate_conversational_response(input, &context);
        context.add_exchange("test".to_string(), "test".to_string());
        let response4 = generate_conversational_response(input, &context);

        // Should cycle through different responses
        assert_ne!(response1, response2);
        assert_ne!(response2, response3);
        assert_eq!(response1, response4); // Should cycle back to first
    }

    #[test]
    fn test_should_offer_quick_actions() {
        let mut context = ChatContext::default();

        // Should not offer actions initially (0 % 3 != 2)
        assert!(!should_offer_quick_actions(&context));

        // Add one exchange (1 % 3 != 2)
        context.add_exchange("test".to_string(), "test".to_string());
        assert!(!should_offer_quick_actions(&context));

        // Add second exchange (2 % 3 == 2)
        context.add_exchange("test".to_string(), "test".to_string());
        assert!(should_offer_quick_actions(&context));

        // Add third exchange (3 % 3 != 2)
        context.add_exchange("test".to_string(), "test".to_string());
        assert!(!should_offer_quick_actions(&context));
    }

    #[test]
    fn test_create_agent_description_basic() {
        let context = create_test_context();

        let description = create_agent_description(&context);

        assert!(description.contains("Multi-chain AI agent"));
        assert!(description.contains("🌟 Solana blockchain queries"));
        assert!(description.contains("⚡ EVM-compatible chains"));
        assert!(description.contains("🌐 Web intelligence"));
        assert!(description.contains("🧠 Graph memory"));
        assert!(!description.contains("🐦 Twitter sentiment"));
        assert!(!description.contains("🔍 Advanced web search"));
    }

    #[test]
    fn test_create_agent_description_with_twitter() {
        let mut context = create_test_context();
        context.config.providers.twitter_bearer_token = Some("test_token".to_string());

        let description = create_agent_description(&context);

        assert!(description.contains("🐦 Twitter sentiment analysis"));
    }

    #[test]
    fn test_create_agent_description_with_exa() {
        let mut context = create_test_context();
        context.config.providers.exa_api_key = Some("test_key".to_string());

        let description = create_agent_description(&context);

        assert!(description.contains("🔍 Advanced web search"));
    }

    #[test]
    fn test_create_agent_description_with_all_providers() {
        let mut context = create_test_context();
        context.config.providers.twitter_bearer_token = Some("test_token".to_string());
        context.config.providers.exa_api_key = Some("test_key".to_string());

        let description = create_agent_description(&context);

        assert!(description.contains("🐦 Twitter sentiment analysis"));
        assert!(description.contains("🔍 Advanced web search"));
    }

    // Note: The show_help, show_context, and offer_quick_actions functions primarily print to stdout
    // and are difficult to test without mocking the print macros. They contain mostly display logic
    // with minimal business logic that would benefit from unit testing.
    // The run_chat function is also an interactive loop that would require complex mocking
    // of user input to test effectively and is better suited for integration testing.
}
