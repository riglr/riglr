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

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Research      │    │   Risk Mgmt     │    │   Execution     │
│     Agent       │    │     Agent       │    │     Agent       │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────▼────────────┐
                    │     Agent Registry       │
                    │   & Task Dispatcher      │
                    └─────────────┬────────────┘
                                  │
                         ┌────────▼────────┐
                         │  Communication  │
                         │     System      │
                         └─────────────────┘
```

## Quick Start

Add riglr-agents to your `Cargo.toml`:

```toml
[dependencies]
riglr-agents = "0.1.0"
riglr-core = "0.1.0"
rig-core = "0.17.1"  # For LLM integration
```

### Basic Example

```rust
use riglr_agents::{
    Agent, AgentDispatcher, LocalAgentRegistry, 
    Task, TaskResult, TaskType, Priority, AgentId
};
use rig_core::{completion::Prompt, providers};
use riglr_core::SignerContext;
use async_trait::async_trait;
use std::sync::Arc;

// Define a simple trading agent with rig-core integration
struct TradingAgent {
    id: AgentId,
    rig_agent: rig_core::Agent<providers::openai::completion::CompletionModel>,
}

impl TradingAgent {
    fn new(id: &str, client: &providers::openai::Client) -> Self {
        let rig_agent = client
            .agent("gpt-4")
            .preamble("You are a trading agent. Execute trades based on market conditions.")
            .build();
        
        Self {
            id: AgentId::new(id),
            rig_agent,
        }
    }
}

#[async_trait]
impl Agent for TradingAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        // Use rig-core for intelligent processing
        let prompt = format!("Analyze and execute trade: {}", task.parameters);
        let llm_analysis = self.rig_agent.prompt(&prompt).await.ok();
        
        // Access signer context for blockchain operations
        let signer = SignerContext::current().await?;
        
        // Perform trading logic using riglr tools
        // ... trading implementation
        
        Ok(TaskResult::success(
            serde_json::json!({
                "status": "trade_executed",
                "llm_analysis": llm_analysis
            }),
            None,
            std::time::Duration::from_millis(100)
        ))
    }

    fn id(&self) -> &AgentId { &self.id }
    
    fn capabilities(&self) -> Vec<String> {
        vec!["trading".to_string(), "execution".to_string()]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create OpenAI client for rig-core
    let client = providers::openai::Client::new(&std::env::var("OPENAI_API_KEY")?);
    
    // Create and register agents with rig-core integration
    let agent = Arc::new(TradingAgent::new("trader-1", &client));
    let registry = Arc::new(LocalAgentRegistry::new());
    registry.register_agent(agent).await?;
    
    // Create dispatcher
    let dispatcher = AgentDispatcher::new(registry, Default::default());
    
    // Dispatch a trading task
    let task = Task::new(
        TaskType::Trading,
        serde_json::json!({"symbol": "BTC/USD", "action": "buy", "amount": 1.0})
    );
    
    let result = dispatcher.dispatch_task(task).await?;
    println!("Task result: {}", result.output);
    
    Ok(())
}
```

## Core Concepts

### Agents

Agents are autonomous units that implement the `Agent` trait and leverage rig-core for LLM operations. They can:
- Execute tasks within their capabilities using LLM-powered intelligence
- Communicate with other agents through the messaging system
- Access blockchain signers through SignerContext
- Maintain their own state and status
- Use rig-core for intelligent decision-making and analysis

### Task Types

Built-in task types include:
- `Trading`: Execute buy/sell operations
- `Research`: Market analysis and intelligence
- `RiskAnalysis`: Risk assessment and validation
- `Portfolio`: Portfolio management operations  
- `Monitoring`: System and market monitoring
- `Custom(String)`: User-defined task types

### Agent Registry

The registry manages agent discovery and lifecycle:
- Register/unregister agents
- Query agents by capabilities
- Monitor agent health and status
- Support for both local and distributed registries

### Task Dispatcher  

Routes tasks to appropriate agents using configurable strategies:
- **Capability-based**: Route to agents with matching capabilities
- **Load-balanced**: Distribute tasks based on agent load
- **Priority-based**: Higher priority tasks get precedence
- **Custom**: Implement your own routing logic

### Communication System

Agents communicate through structured messages:
- Broadcast messages to all agents
- Send targeted messages to specific agents
- Subscribe to message types
- Async message handling

## Examples

The repository includes comprehensive examples:

### Basic Dispatcher
Simple multi-agent system with task routing:
```bash
cargo run --example basic_dispatcher
```

### Trading Swarm
Sophisticated trading system with research, risk management, and execution agents:
```bash
cargo run --example trading_swarm
```

### Risk Management System
Coordinated risk assessment across multiple agents:
```bash
cargo run --example risk_management_system
```

## Integration with riglr Tools

riglr-agents seamlessly integrates with all riglr tools:

```rust
use riglr_agents::Agent;
use riglr_solana_tools::pump::buy_pump_token;
use riglr_evm_tools::swap::swap_exact_eth_for_tokens;
use riglr_web_tools::dexscreener::get_token_data;

#[derive(Clone)]
struct MultiChainAgent {
    // ... agent fields
}

#[async_trait]
impl Agent for MultiChainAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        match task.task_type {
            TaskType::Trading => {
                let network = task.input.get("network").and_then(|n| n.as_str()).unwrap_or("solana");
                
                match network {
                    "solana" => {
                        // Use riglr-solana-tools
                        let result = buy_pump_token("token_addr", 1.0, Some(0.05), &rpc_url).await?;
                        // ... handle result
                    }
                    "ethereum" => {
                        // Use riglr-evm-tools  
                        let result = swap_exact_eth_for_tokens("token_addr", 1.0, 0.0, 1, None).await?;
                        // ... handle result
                    }
                    _ => return Err(riglr_agents::AgentError::InvalidTask("Unsupported network".to_string())),
                }
            }
            TaskType::Research => {
                // Use riglr-web-tools for research
                let data = get_token_data("token_addr", "solana").await?;
                // ... analyze data
            }
            _ => {
                // Handle other task types
            }
        }
        
        Ok(TaskResult::success(serde_json::json!({}), None, Duration::from_millis(100)))
    }
    
    // ... other trait methods
}
```

## SignerContext in Multi-Agent Systems

riglr-agents maintains riglr's security model. Each agent operation runs within a SignerContext:

```rust
use riglr_core::{SignerContext, signer::MemorySignerFactory};

async fn run_agents() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let signer_factory = MemorySignerFactory::new();
    
    // All agent operations run within this context
    SignerContext::new(&signer_factory).execute(async {
        let dispatcher = create_dispatcher().await;
        
        // Tasks executed by agents will have access to signers
        let result = dispatcher.dispatch_task(trading_task).await?;
        
        Ok(())
    }).await?;
    
    Ok(())
}
```

Agents access the current signer context:

```rust
#[async_trait]
impl Agent for MyAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        // Get the current signer - this is secure and isolated
        let signer = SignerContext::current().await?;
        
        // Use the signer for blockchain operations
        let balance = get_sol_balance(&signer.solana_address().to_string(), &rpc_url).await?;
        
        // ... rest of implementation
    }
}
```

## Error Handling

riglr-agents follows riglr's error classification system:

```rust
use riglr_agents::{AgentError, TaskResult};

impl Agent for MyAgent {
    async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
        match risky_operation().await {
            Ok(result) => Ok(TaskResult::success(result, None, duration)),
            Err(e) if e.is_network_error() => {
                // Retriable errors can be retried by the dispatcher
                Ok(TaskResult::failure(
                    format!("Network error: {}", e),
                    true,  // retriable
                    duration
                ))
            }
            Err(e) => {
                // Permanent errors should not be retried
                Err(AgentError::ExecutionFailed(format!("Permanent failure: {}", e)))
            }
        }
    }
}
```

## Configuration

Configure agent behavior through `DispatchConfig`:

```rust
use riglr_agents::{DispatchConfig, RoutingStrategy};
use std::time::Duration;

let config = DispatchConfig {
    routing_strategy: RoutingStrategy::CapabilityBased,
    max_retries: 3,
    timeout: Duration::from_secs(30),
    enable_load_balancing: true,
};

let dispatcher = AgentDispatcher::new(registry, config);
```

## Testing

riglr-agents provides testing utilities:

```rust
use riglr_agents::test_utils::MockAgent;

#[tokio::test]
async fn test_agent_coordination() {
    let mock_agent = MockAgent::new("test-agent", vec!["trading".to_string()]);
    let registry = LocalAgentRegistry::new();
    registry.register_agent(Arc::new(mock_agent)).await.unwrap();
    
    let dispatcher = AgentDispatcher::new(registry, Default::default());
    
    let task = Task::new(TaskType::Trading, serde_json::json!({}));
    let result = dispatcher.dispatch_task(task).await.unwrap();
    
    assert!(result.is_success());
}
```

## Performance Considerations

- **Agent Load Balancing**: Use `enable_load_balancing` for high-throughput scenarios
- **Connection Pooling**: Agents should reuse HTTP/RPC connections
- **Task Batching**: Group related tasks when possible
- **Timeout Configuration**: Set appropriate timeouts for blockchain operations
- **Retry Strategy**: Configure retries based on your risk tolerance

## Security Best Practices

1. **Signer Isolation**: Never pass signers directly between agents
2. **Input Validation**: Validate all task inputs before execution
3. **Rate Limiting**: Implement rate limits for external API calls
4. **Error Handling**: Don't leak sensitive information in error messages
5. **Audit Logging**: Log all significant agent actions
6. **Capability Principle**: Agents should only have minimum required capabilities

## Roadmap

- [ ] Distributed agent registry with Redis/database backing
- [ ] Advanced routing strategies (ML-based, geographic)
- [ ] Built-in metrics and monitoring
- [ ] Agent marketplace and plugin system
- [ ] GraphQL API for agent management
- [ ] Integration with container orchestration (Kubernetes)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- [Documentation](https://docs.rs/riglr-agents)
- [Repository](https://github.com/riglr/riglr)
- [Examples](https://github.com/riglr/riglr/tree/main/riglr-agents/examples)
- [riglr Main Documentation](https://docs.rs/riglr-core)