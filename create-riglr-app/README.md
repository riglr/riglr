# {{project-name}}

{{description}}

This project was generated using [create-riglr-app](https://github.com/riglr-project/create-riglr-app), a template for building sophisticated AI agents that interact with blockchain networks and web data sources.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
{% if include-graph-memory -%}
- Neo4j database (for knowledge storage)
{% endif %}
- Redis (for job queue and caching)
- API keys for your chosen services (see Configuration section)

### Installation

1. **Clone and setup your project:**
   ```bash
   # If using cargo-generate (recommended):
   cargo generate riglr-project/create-riglr-app --name {{project-name}}
   cd {{project-name}}
   
   # Or if you have this project already:
   # (repository setup instructions would go here)
   ```

2. **Configure environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your actual API keys and configuration
   ```

3. **Start required services:**

   {% if include-graph-memory -%}
   **Neo4j (for knowledge storage):**
   ```bash
   # Using Docker (recommended):
   docker run -d \
     --name neo4j \
     -p 7474:7474 -p 7687:7687 \
     -e NEO4J_AUTH=neo4j/password \
     neo4j:latest
   
   # Or install locally: https://neo4j.com/download/
   ```
   {% endif %}

   **Redis (for job queue):**
   ```bash
   # Using Docker:
   docker run -d --name redis -p 6379:6379 redis:alpine
   
   # Or install locally: https://redis.io/download
   ```

4. **Build and run:**
   ```bash
   cargo build --release
   cargo run -- --interactive
   ```

## ⚙️ Configuration

### Required API Keys

{% if include-web-tools -%}
#### Social Media & Web Data
- **Twitter/X API**: Get from [Twitter Developer Portal](https://developer.twitter.com/)
- **Exa Search API**: Get from [Exa.ai](https://exa.ai/)
- **NewsAPI**: Get from [NewsAPI.org](https://newsapi.org/)
- **CryptoPanic**: Get from [CryptoPanic Developers](https://cryptopanic.com/developers/api/)

{% endif %}
#### AI Provider
Choose one of the supported LLM providers:
- **OpenAI**: Get from [OpenAI Platform](https://platform.openai.com/)
- **Anthropic**: Get from [Anthropic Console](https://console.anthropic.com/)
- **Other providers**: See [rig-core documentation](https://docs.rig.rs/)

#### Blockchain RPC
{% if primary-chain == "solana" or primary-chain == "both" -%}
- **Solana**: Use public endpoint or get from [Alchemy](https://www.alchemy.com/), [Helius](https://helius.xyz/), or [QuickNode](https://www.quicknode.com/)
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
- **Ethereum**: Use public endpoint or get from [Alchemy](https://www.alchemy.com/), [Infura](https://infura.io/), or [QuickNode](https://www.quicknode.com/)
{% endif %}

### Configuration Files

All configuration is done through environment variables in `.env`. Key settings include:

- `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`: Your AI provider API key
{% if primary-chain == "solana" or primary-chain == "both" -%}
- `SOLANA_RPC_URL`: Solana network endpoint
{% endif %}
{% if primary-chain == "ethereum" or primary-chain == "both" -%}
- `ETHEREUM_RPC_URL`: Ethereum network endpoint  
{% endif %}
{% if include-web-tools -%}
- `TWITTER_BEARER_TOKEN`: Twitter API access
- `EXA_API_KEY`: Web search capabilities
- `NEWSAPI_KEY`: News aggregation
{% endif %}
- `REDIS_URL`: Redis connection for job queue
{% if include-graph-memory -%}
- `NEO4J_URL`: Neo4j connection for knowledge storage
{% endif %}

## 🎯 Usage

### Interactive Mode

Run the agent in interactive mode for real-time conversation:

```bash
cargo run -- --interactive
```

Available commands:
- `help` - Show available commands
- `status` - Display agent status and configuration
{% if agent-type == "trading-bot" -%}
- `portfolio` - Check current portfolio balances
{% endif %}
- `quit` / `exit` - Shutdown the agent

### Task Execution

Execute a specific task:

```bash
cargo run -- --task "Check the latest news about Solana and summarize the key points"
```

### Example Interactions

{% if agent-type == "trading-bot" -%}
**Trading Bot Examples:**
```
> What's my current SOL balance?
> Analyze the market sentiment for ETH before I make a trade
> Check DexScreener for the best USDC/SOL pools
> What's the latest news that might affect Bitcoin price?
```

{% elif agent-type == "market-analyst" -%}
**Market Analyst Examples:**
```
> Analyze the current sentiment around DeFi tokens
> What are the trending topics in crypto Twitter today?
> Find recent news about regulatory changes affecting crypto
> Compare the market data for top 10 cryptocurrencies
```

{% elif agent-type == "news-monitor" -%}
**News Monitor Examples:**
```
> Monitor breaking news about Ethereum upgrades
> What's the sentiment around the latest Bitcoin ETF news?
> Find trending discussions about DeFi on social media
> Alert me about any major regulatory announcements
```

{% else -%}
**General Examples:**
```
> What's the current price and sentiment for [TOKEN]?
> Find recent news about [TOPIC]
> Check social media discussion about [PROJECT]
> Analyze on-chain activity for [ADDRESS]
```
{% endif %}

## 🛠️ Development

### Project Structure

```
{{project-name}}/
├── src/
│   ├── main.rs              # Main application entry point
│   {% if agent-type == "trading-bot" -%}
│   ├── bin/
│   │   └── trading_bot.rs   # Specialized trading bot binary
│   {% endif %}
│   └── lib.rs               # Library code (if needed)
├── Cargo.toml               # Rust dependencies
├── .env.example             # Environment configuration template
└── README.md                # This file
```

### Adding Custom Tools

To add your own tools to the agent:

1. **Create a new tool function:**
   ```rust
   use riglr_macros::tool;
   
   #[tool]
   /// Custom tool description for the AI
   pub async fn my_custom_tool(param: String) -> Result<String> {
       // Your tool implementation
       Ok(format!("Processed: {}", param))
   }
   ```

2. **Add it to your agent:**
   ```rust
   let agent = Agent::builder(&provider)
       .tool(my_custom_tool)  // Add your tool here
       .build();
   ```

### Testing

Run the test suite:

```bash
cargo test
```

For integration tests with live APIs, set `TEST_MODE=false` in your `.env`:

```bash
TEST_MODE=false cargo test -- --ignored
```

### Logging

Adjust logging levels in `.env`:

```env
RUST_LOG=debug  # trace, debug, info, warn, error
```

## 📊 Monitoring

{% if agent-type == "trading-bot" -%}
### Portfolio Tracking

The trading bot includes built-in portfolio tracking:

- Real-time balance monitoring
- Transaction history
- P&L calculations
- Risk metrics

Access via the `portfolio` command in interactive mode.

### Risk Management

Configured via environment variables:

- `MAX_TRADE_SIZE_USD`: Maximum trade size
- `STOP_LOSS_PERCENTAGE`: Automatic stop-loss threshold  
- `TAKE_PROFIT_PERCENTAGE`: Automatic take-profit threshold

{% endif %}

### Health Checks

The agent provides status information:

```bash
cargo run -- --task "Show system status"
```

This displays:
- Service connectivity status
- API rate limit usage
{% if include-graph-memory -%}
- Knowledge base statistics
{% endif %}
- Recent activity summary

## 🔧 Troubleshooting

### Common Issues

**API Key Errors:**
- Verify all required API keys are set in `.env`
- Check API key permissions and usage limits
- Test connectivity to each service

**Database Connection Issues:**
{% if include-graph-memory -%}
- Ensure Neo4j is running and accessible
{% endif %}
- Verify Redis is running on the correct port
- Check firewall and network settings

**Build Errors:**
- Update Rust to latest stable version
- Clear cargo cache: `cargo clean`
- Check for conflicting dependency versions

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run -- --interactive
```

### Getting Help

- Check the [riglr documentation](https://docs.riglr.dev)
- Visit the [riglr GitHub repository](https://github.com/riglr-project/riglr)
- Join the [Discord community](https://discord.gg/riglr)

## 🚀 Next Steps

Now that you have {{agent-name}} running, consider:

1. **Customize the system prompt** in `main.rs` to match your specific needs
2. **Add domain-specific tools** for your use case
3. **Implement custom data sources** beyond the included web tools
4. **Set up monitoring and alerting** for production usage
5. **Explore advanced riglr features** like:
   - Graph memory for persistent learning
   - Multi-agent coordination
   - Custom execution strategies

## 📚 Learn More

- [riglr Documentation](https://docs.riglr.dev) - Comprehensive guides and API reference
- [rig-core Documentation](https://docs.rig.rs/) - Underlying AI agent framework
- [Examples Repository](https://github.com/riglr-project/examples) - More example agents
- [Best Practices Guide](https://docs.riglr.dev/best-practices) - Production deployment tips

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

*Built with [riglr](https://riglr.dev) - The framework for building intelligent blockchain agents.*