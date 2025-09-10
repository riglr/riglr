# Quick Start

Get your first riglr agent running in minutes. This guide assumes you have Rust and Docker installed.

### 1. Install `create-riglr-app`

The easiest way to start is with our official project generator.

```bash
cargo install create-riglr-app
```

### 2. Generate a New Project

Create a new trading bot project. The CLI will guide you through the setup.

```bash
create-riglr-app my-trading-bot
```

### 3. Configure Your Environment

Navigate into your new project directory and set up your environment variables.

```bash
cd my-trading-bot
cp .env.example .env
```

Now, open the `.env` file and add your API keys and blockchain RPC endpoints. At a minimum, you'll need:

* `REDIS_URL` - Required for the job queue and caching (e.g., `redis://localhost:6379`)
* `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` - For AI model access
* `SOLANA_RPC_URL` - For Solana blockchain access
* `RPC_URL_1` - For Ethereum mainnet (or other `RPC_URL_{CHAIN_ID}` for EVM chains)

### 4. Start Required Services

riglr uses Redis for its job queue. You can start it easily with Docker.

```bash
docker run -d --name redis -p 6379:6379 redis:alpine
```

### 5. Run Your Agent

Build and run your agent in interactive mode.

```bash
cargo build
cargo run -- --interactive
```

You can now chat with your agent! Try asking it a question like: "What is the SOL balance of `So11111111111111111111111111111111111111112`?"

### Next Steps

* Explore how to [Create Your First Agent](create-riglr-app.md) in more detail.
* Dive into our [Tutorials](../tutorials/index.md) to build more complex agents.