# Agent Coordination

The `riglr-agents` crate provides a powerful framework for building distributed networks of specialized AI agents that can coordinate, communicate, and collaborate on complex tasks.

## Core Concepts

### The Agent/Dispatcher/Registry Model

The agent coordination system is built on three fundamental components:

```rust
┌──────────────┐
│   Registry   │ ← Agent registration & discovery
└──────┬───────┘
       │
┌──────▼───────┐
│  Dispatcher  │ ← Task routing & orchestration
└──────┬───────┘
       │
┌──────▼───────┐ ┌──────────┐ ┌──────────┐
│   Agent A    │ │  Agent B │ │  Agent C │ ← Specialized agents
└──────────────┘ └──────────┘ └──────────┘
```

**Registry**: The central service discovery mechanism where agents register their capabilities and availability. It maintains a real-time view of all active agents in the system.

**Dispatcher**: The intelligent task router that receives requests and determines which agent(s) should handle them based on capabilities, load, and routing strategies.

**Agents**: Specialized workers that perform specific tasks. Each agent registers its capabilities with the Registry and receives tasks from the Dispatcher.

## Task Routing Strategies

The Dispatcher supports multiple routing strategies to optimize task distribution:

### Round-Robin
Distributes tasks evenly across all available agents, ensuring balanced load distribution.

```rust
let dispatcher = Dispatcher::builder()
    .routing_strategy(RoutingStrategy::RoundRobin)
    .build();
```

### Capability-Based
Routes tasks to agents based on their registered capabilities and specializations.

```rust
let dispatcher = Dispatcher::builder()
    .routing_strategy(RoutingStrategy::CapabilityBased)
    .build();
```

### Load-Balanced
Considers current agent workload and routes tasks to the least busy agents.

```rust
let dispatcher = Dispatcher::builder()
    .routing_strategy(RoutingStrategy::LoadBalanced)
    .build();
```

### Priority-Based
Routes high-priority tasks to specialized agents while distributing routine tasks across the pool.

```rust
let dispatcher = Dispatcher::builder()
    .routing_strategy(RoutingStrategy::Priority)
    .build();
```

## Inter-Agent Communication

Agents can communicate directly or through the message bus for complex coordination:

### Direct Messaging
Point-to-point communication between agents for tight coupling scenarios.

```rust
// Agent A sends a message to Agent B
agent_a.send_to("agent_b", Message {
    payload: json!({ "action": "prepare_data" }),
    correlation_id: uuid::Uuid::new_v4(),
}).await?;
```

### Broadcast Messages
One-to-many communication for notifications and events.

```rust
// Broadcast market event to all trading agents
dispatcher.broadcast(BroadcastMessage {
    topic: "market_event",
    payload: json!({ "symbol": "SOL", "price": 150.0 }),
}).await?;
```

### Request-Response Pattern
Synchronous communication with guaranteed responses.

```rust
// Request data from another agent
let response = agent_a.request_from("data_agent", Request {
    action: "fetch_prices",
    params: json!({ "symbols": ["SOL", "ETH"] }),
}).await?;
```

## Building a Multi-Agent System

Here's a complete example of setting up a multi-agent trading system:

```rust
use riglr_agents::{Agent, Dispatcher, Registry, RoutingStrategy};
use riglr_core::SignerContext;

// Create the registry
let registry = Registry::new();

// Create specialized agents
let market_analyzer = Agent::builder()
    .name("market_analyzer")
    .capabilities(vec!["analyze_market", "detect_trends"])
    .handler(|task| async move {
        // Market analysis logic
        analyze_market(task).await
    })
    .build();

let trade_executor = Agent::builder()
    .name("trade_executor")
    .capabilities(vec!["execute_trade", "manage_position"])
    .handler(|task| async move {
        // Trade execution logic
        execute_trade(task).await
    })
    .build();

let risk_manager = Agent::builder()
    .name("risk_manager")
    .capabilities(vec!["assess_risk", "set_limits"])
    .handler(|task| async move {
        // Risk management logic
        assess_risk(task).await
    })
    .build();

// Register agents
registry.register(market_analyzer).await?;
registry.register(trade_executor).await?;
registry.register(risk_manager).await?;

// Create dispatcher with capability-based routing
let dispatcher = Dispatcher::builder()
    .registry(registry)
    .routing_strategy(RoutingStrategy::CapabilityBased)
    .build();

// Submit tasks
dispatcher.submit(Task {
    capability_required: "analyze_market",
    payload: json!({ "symbol": "SOL" }),
}).await?;
```

## Agent Lifecycle Management

### Health Checks
Agents automatically report their health status to the Registry:

```rust
let agent = Agent::builder()
    .name("trading_agent")
    .health_check_interval(Duration::from_secs(30))
    .health_check(|| async {
        // Check RPC connection, memory usage, etc.
        HealthStatus::Healthy
    })
    .build();
```

### Graceful Shutdown
Agents can be gracefully shut down, completing current tasks before termination:

```rust
// Signal shutdown
agent.shutdown().await?;

// Wait for completion with timeout
agent.wait_for_completion(Duration::from_secs(60)).await?;
```

### Auto-Scaling
The system can automatically spawn new agents based on load:

```rust
let auto_scaler = AutoScaler::builder()
    .min_agents(2)
    .max_agents(10)
    .scale_up_threshold(0.8)  // 80% utilization
    .scale_down_threshold(0.2) // 20% utilization
    .build();

dispatcher.attach_auto_scaler(auto_scaler);
```

## Advanced Patterns

### Agent Pools
Group similar agents for efficient resource management:

```rust
let trading_pool = AgentPool::builder()
    .template(trading_agent_template)
    .size(5)
    .build();

registry.register_pool(trading_pool).await?;
```

### Workflow Orchestration
Chain multiple agents for complex workflows:

```rust
let workflow = Workflow::builder()
    .step("analyze", "market_analyzer")
    .step("assess_risk", "risk_manager")
    .step("execute", "trade_executor")
    .on_failure(FailureStrategy::Retry { max_attempts: 3 })
    .build();

dispatcher.execute_workflow(workflow, initial_data).await?;
```

### Event Sourcing
Track all agent actions for audit and replay:

```rust
let event_store = EventStore::postgres(database_url);

let dispatcher = Dispatcher::builder()
    .event_store(event_store)
    .build();

// Replay events for debugging
let events = dispatcher.replay_events(
    TimeRange::from(yesterday)..TimeRange::to(now)
).await?;
```

## Best Practices

1. **Single Responsibility**: Each agent should have a clear, focused purpose
2. **Idempotency**: Agent operations should be idempotent for retry safety
3. **Timeout Handling**: Always set appropriate timeouts for agent operations
4. **Error Propagation**: Use structured errors that can be handled by the Dispatcher
5. **Monitoring**: Instrument agents with metrics and logging for observability

## Integration with riglr Tools

Agents seamlessly integrate with other riglr components:

```rust
use riglr_solana_tools::pump;
use riglr_streams::StreamManager;

let trading_agent = Agent::builder()
    .name("pump_trader")
    .handler(|task| async move {
        // Access SignerContext
        let ctx = SignerContext::current();
        
        // Use riglr tools
        let balance = pump::get_token_balance(
            &task.token_address
        ).await?;
        
        // Subscribe to streams
        let stream = StreamManager::subscribe("pump_events").await?;
        
        // Execute trades
        if should_buy(balance, stream) {
            pump::buy_token(&task.token_address, amount).await?;
        }
        
        Ok(())
    })
    .build();
```

## Performance Considerations

- **Message Serialization**: Use efficient formats like MessagePack for inter-agent communication
- **Connection Pooling**: Reuse connections between frequently communicating agents
- **Batch Processing**: Group similar tasks for efficient processing
- **Caching**: Share cached data between agents via distributed cache

## Next Steps

- Learn about [Real-Time Streaming](streams.md) for event-driven agents
- Explore the [Indexer](indexer.md) for historical data access
- Understand [Configuration](configuration.md) for production deployments