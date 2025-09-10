# create-riglr-app

[![Crates.io](https://img.shields.io/crates/v/create-riglr-app.svg)](https://crates.io/crates/create-riglr-app)
[![Documentation](https://docs.rs/create-riglr-app/badge.svg)](https://docs.rs/create-riglr-app)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Official project generator for [Riglr](https://github.com/riglr/riglr) - the production-ready framework for building AI agents that interact with blockchains.

## Installation

```bash
cargo install create-riglr-app
```

## Usage

Create a new Riglr project:

```bash
create-riglr-app my-agent
```

This will:
1. Create a new directory with your project name
2. Generate a complete Riglr agent project structure
3. Set up all necessary configuration files
4. Include example implementations

### Interactive Mode

After creating your project:

```bash
cd my-agent
cp .env.example .env  # Configure your API keys and RPC URLs
cargo run -- --interactive
```

### Project Templates

The generator creates projects with:
- Pre-configured agent setup
- Multi-chain support (Solana, EVM)
- Tool implementations
- Configuration management
- Docker support
- Testing setup

## Features

- 🚀 **Quick Start**: Get a working agent in minutes
- 🔧 **Zero Boilerplate**: All the setup done for you
- 🌐 **Multi-Chain Ready**: Solana and EVM support out of the box
- 📦 **Production Ready**: Includes Docker, CI/CD, and deployment configs
- 🧪 **Testing Setup**: Unit and integration test templates included

## Generated Project Structure

```
my-agent/
├── Cargo.toml           # Project dependencies
├── .env.example         # Environment variables template
├── Dockerfile           # Container configuration
├── README.md           # Project documentation
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library code
│   ├── agents/         # Agent implementations
│   ├── tools/          # Custom tools
│   └── config/         # Configuration
└── tests/              # Test files
```

## Configuration

The generated `.env.example` includes all necessary configuration:

```env
# LLM Configuration
OPENAI_API_KEY=your_key_here

# Blockchain RPCs
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your_key

# Agent Configuration
AGENT_NAME=my-agent
LOG_LEVEL=info
```

## Requirements

- Rust 1.70 or higher
- Cargo

## Documentation

For detailed documentation on using Riglr agents, visit:
- [Riglr Documentation](https://riglr.com/docs)
- [API Reference](https://docs.rs/riglr-core)
- [GitHub Repository](https://github.com/riglr/riglr)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/riglr/riglr) for contribution guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details.