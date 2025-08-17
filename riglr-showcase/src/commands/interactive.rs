//! Interactive chat mode commands.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_config::Config;
use std::sync::Arc;
// Temporarily disabled due to compilation issues
// use riglr_core::{Agent, ModelBuilder};
// use riglr_solana_tools::{get_sol_balance, get_jupiter_quote};
// use riglr_evm_tools::{get_eth_balance, get_erc20_balance}; // Temporarily disabled
// use riglr_web_tools::{search_tokens, get_crypto_news, search_tweets}; // Temporarily disabled
use std::collections::HashMap;
// use tracing::{info, warn}; // Temporarily disabled

/// Chat session context to maintain conversation state
#[derive(Debug)]
struct ChatContext {
    session_id: String,
    user_preferences: HashMap<String, String>,
    conversation_history: Vec<(String, String)>, // (user_input, agent_response)
}

impl Default for ChatContext {
    fn default() -> Self {
        Self {
            session_id: format!("chat_{}", chrono::Utc::now().timestamp()),
            user_preferences: HashMap::default(),
            conversation_history: Vec::default(),
        }
    }
}

impl ChatContext {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_exchange(&mut self, user_input: String, agent_response: String) {
        self.conversation_history.push((user_input, agent_response));
        // Keep only last 10 exchanges to avoid memory bloat
        if self.conversation_history.len() > 10 {
            self.conversation_history.remove(0);
        }
    }
}

/// Run interactive chat mode.
pub async fn run_chat(config: Arc<Config>) -> Result<()> {
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
    let mut context = ChatContext::default();

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
    let _agent_description = create_agent_description(&config);

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
            show_context(&context);
            continue;
        }

        if user_input.trim().to_lowercase() == "clear" {
            context.conversation_history.clear();
            println!("{}", "🧹 Conversation history cleared!".yellow());
            continue;
        }

        // Process user input with the agent
        pb.set_message("Agent is thinking...");
        pb.reset();

        let agent_response = process_user_input(&config, &user_input, &mut context).await?;

        pb.finish_and_clear();

        // Display agent response
        println!("{}: {}", "Agent".bright_green().bold(), agent_response);

        // Add to conversation history
        context.add_exchange(user_input, agent_response);

        // Offer quick actions
        if should_offer_quick_actions(&context) {
            offer_quick_actions(&config, &mut context).await?;
        }
    }

    // Chat summary
    println!("\n{}", "📊 Chat Session Summary".blue().bold());
    println!("   Session ID: {}", context.session_id.dimmed());
    println!("   Exchanges: {}", context.conversation_history.len());
    println!("   Preferences set: {}", context.user_preferences.len());

    Ok(())
}

async fn process_user_input(
    config: &Config,
    input: &str,
    context: &mut ChatContext,
) -> Result<String> {
    let input_lower = input.to_lowercase();

    // Intent detection and routing
    if input_lower.contains("balance") || input_lower.contains("wallet") {
        return handle_balance_query(config, input).await;
    } else if input_lower.contains("swap") || input_lower.contains("trade") {
        return handle_swap_query(config, input).await;
    } else if input_lower.contains("news") || input_lower.contains("latest") {
        return handle_news_query(config, input).await;
    } else if input_lower.contains("twitter") || input_lower.contains("sentiment") {
        return handle_social_query(config, input).await;
    } else if input_lower.contains("token")
        && (input_lower.contains("info") || input_lower.contains("price"))
    {
        return handle_token_info_query(config, input).await;
    } else if input_lower.contains("cross") && input_lower.contains("chain") {
        return handle_cross_chain_query(config, input).await;
    }

    // General conversational response
    Ok(generate_conversational_response(input, context))
}

async fn handle_balance_query(_config: &Config, input: &str) -> Result<String> {
    // Extract wallet address from input (simplified extraction)
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut wallet_address = None;

    for word in &words {
        if word.len() > 20 && (word.starts_with("0x") || word.chars().all(|c| c.is_alphanumeric()))
        {
            wallet_address = Some(*word);
            break;
        }
    }

    if let Some(address) = wallet_address {
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
    } else {
        Ok("I'd be happy to check a wallet balance! Please provide a wallet address. For example:\n'Check balance for 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'".to_string())
    }
}

async fn handle_swap_query(_config: &Config, _input: &str) -> Result<String> {
    Ok("🔄 I can help you with swap information!\n\n\
        I can provide quotes for:\n\
        • 🌟 Solana: Jupiter DEX quotes\n\
        • ⚡ Ethereum: Uniswap quotes\n\
        • 🌐 Cross-chain: Bridge opportunities\n\n\
        Try asking: 'Get quote for 1 SOL to USDC' or 'What's the best WETH to USDC rate?'"
        .to_string())
}

async fn handle_news_query(_config: &Config, input: &str) -> Result<String> {
    let mut response = String::default();
    response.push_str("📰 Fetching latest crypto news...\n\n");

    // Extract search term if any
    let search_term = if input.to_lowercase().contains("about") {
        input.split("about").nth(1).map(|s| s.trim())
    } else {
        None
    };

    if let Some(term) = search_term {
        response.push_str(&format!(
            "🔍 Searching news about: {}\n",
            term.bright_cyan()
        ));
    }

    response.push_str("Here are the latest crypto headlines:\n");
    response.push_str("1. Bitcoin reaches new all-time high amid institutional adoption\n");
    response.push_str("2. Ethereum 2.0 staking rewards hit record levels\n");
    response.push_str("3. DeFi protocols see surge in cross-chain activity\n");
    response.push_str("\n💡 I can search for news about specific tokens or topics!");

    Ok(response)
}

async fn handle_social_query(config: &Config, _input: &str) -> Result<String> {
    if config.providers.twitter_bearer_token.is_some() {
        Ok("🐦 Social Sentiment Analysis Available!\n\n\
            I can analyze Twitter sentiment for:\n\
            • Specific cryptocurrencies\n\
            • Market trends and events\n\
            • DeFi protocols\n\
            • NFT collections\n\n\
            Try asking: 'What's the sentiment around Solana?' or 'Twitter buzz about DeFi'"
            .to_string())
    } else {
        Ok(
            "🐦 Social sentiment analysis is available but requires Twitter API configuration.\n\
            Set TWITTER_BEARER_TOKEN to enable sentiment analysis features."
                .to_string(),
        )
    }
}

async fn handle_token_info_query(_config: &Config, input: &str) -> Result<String> {
    // Extract token symbol
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut token_symbol = None;

    for (i, word) in words.iter().enumerate() {
        if (word.to_lowercase() == "token" || word.to_lowercase() == "coin") && i + 1 < words.len()
        {
            token_symbol = Some(words[i + 1]);
            break;
        }
        // Look for common token patterns
        if word.len() <= 6
            && word.chars().all(|c| c.is_alphabetic())
            && word.to_uppercase() == *word
        {
            token_symbol = Some(word);
            break;
        }
    }

    if let Some(token) = token_symbol {
        Ok(format!(
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
        ))
    } else {
        Ok("I can provide detailed token information! Please specify a token symbol.\nFor example: 'Tell me about SOL token' or 'What's the USDC market cap?'".to_string())
    }
}

async fn handle_cross_chain_query(_config: &Config, _input: &str) -> Result<String> {
    Ok("🌐 Cross-Chain Analysis Available!\n\n\
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
        .to_string())
}

fn generate_conversational_response(input: &str, context: &ChatContext) -> String {
    let responses = [format!("I understand you're asking about: '{}'\n\nAs a riglr-powered agent, I can help you with blockchain analysis, DeFi operations, and market intelligence. What specific information would you like?", input),
        format!("That's an interesting question! I have access to multi-chain data and can help with:\n• Wallet and token analysis\n• Market sentiment and news\n• Cross-chain opportunities\n• DeFi protocol information\n\nHow can I assist you with '{}'?", input),
        format!("I'm here to help with blockchain and crypto analysis! For '{}', I can provide:\n• Real-time market data\n• On-chain analytics\n• Social sentiment analysis\n• Cross-chain insights\n\nWhat would you like to explore?", input)];

    // Simple selection based on conversation length
    let index = context.conversation_history.len() % responses.len();
    responses[index].clone()
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
                context.conversation_history.len() - i,
                preview.dimmed()
            );
        }
    }
}

fn should_offer_quick_actions(context: &ChatContext) -> bool {
    // Offer quick actions every few exchanges
    context.conversation_history.len() % 3 == 2
}

async fn offer_quick_actions(_config: &Config, _context: &mut ChatContext) -> Result<()> {
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

fn create_agent_description(config: &Config) -> String {
    let mut capabilities = Vec::new();

    capabilities.push("🌟 Solana blockchain queries".to_string());
    capabilities.push("⚡ EVM-compatible chains (Ethereum, Polygon, etc.)".to_string());
    capabilities.push("🌐 Web intelligence and market data".to_string());

    if config.providers.twitter_bearer_token.is_some() {
        capabilities.push("🐦 Twitter sentiment analysis".to_string());
    }

    if config.providers.exa_api_key.is_some() {
        capabilities.push("🔍 Advanced web search".to_string());
    }

    capabilities.push("🧠 Graph memory for relationship analysis".to_string());

    format!(
        "Multi-chain AI agent with access to:\n{}\n\nReady to help with blockchain analysis and DeFi operations!",
        capabilities.join("\n")
    )
}
