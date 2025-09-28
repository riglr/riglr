# riglr-agents

[![Crates.io](https://img.shields.io/crates/v/riglr-agents.svg)](https://crates.io/crates/riglr-agents)
[![Documentation](https://docs.rs/riglr-agents/badge.svg)](https://docs.rs/riglr-agents)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Multi-agent coordination system for riglr blockchain automation with rig-core integration. Build sophisticated LLM-powered trading systems, risk management workflows, and multi-chain coordination using specialized agents that leverage rig-core for intelligent decision-making while working together seamlessly.

## Overview

riglr-agents provides a framework for creating and coordinating multiple specialized AI agents that can perform complex blockchain operations. Each agent uses rig-core for LLM-powered intelligence and can have specific capabilities (trading, research, risk analysis, etc.) while maintaining riglr's security guarantees through the SignerContext pattern.

## Key Features

- **Multi-Agent Coordination**: Orchestrate complex workflows across multiple specialized agents
- **LLM-Powered Intelligence**: Each agent uses rig-core for intelligent decision-making
- **Flexible Task Routing**: Route tasks based on capabilities, load, priority, and custom rules
- **Inter-Agent Communication**: Built-in message passing system for agent coordination
- **Scalable Architecture**: Support for both local and distributed agent registries
- **Security First**: Maintains riglr's SignerContext security model
- **Integration Ready**: Seamless integration with all riglr tools and rig-core patterns

## Quick Start

Add riglr-agents to your `Cargo.toml`:

```toml
[dependencies]
riglr-agents = "0.1.0"
riglr-core = "0.1.0"
rig-core = "0.19.0"  # For LLM integration
```

## Documentation

For comprehensive documentation, examples, and best practices, see: [docs/src/concepts/agents.md](../docs/src/concepts/agents.md)

The documentation covers:
- Core concepts (Registry, Dispatcher, Agents)
- Task routing strategies
- Inter-agent communication patterns
- Building multi-agent systems with rig-core integration
- Agent lifecycle management
- Advanced patterns (pools, workflows, event sourcing)
- Performance considerations and scalability
- Integration with riglr tools

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)