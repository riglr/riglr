# riglr-agents

{{#include ../../../riglr-agents/README.md}}

## API Reference

## Key Components

> The most important types and functions in this crate.

### `Agent`

Core trait that all agents must implement.

[→ Full documentation](#traits)

### `AgentDispatcher`

Agent dispatcher for routing tasks to appropriate agents.

[→ Full documentation](#structs)

### `LocalAgentRegistry`

In-memory agent registry for single-node deployments.

[→ Full documentation](#structs)

### `RedisAgentRegistry`

Redis-backed distributed agent registry for multi-process coordination

[→ Full documentation](#structs)

### `ToolCallingAgent`

An agent that specializes in calling tools based on LLM decisions.

[→ Full documentation](#structs)

---

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `AgentId`

Unique identifier for an agent in the system.

---

#### `AgentMessage`

Message passed between agents.

---

#### `AgentStatus`

Agent status information.

---

#### `AgentSystem`

A complete agent system with local registry.

---

#### `AgentSystemBuilder`

Builder for creating and configuring agent systems.

The AgentSystemBuilder provides a fluent API for setting up complete
multi-agent systems with registries, dispatchers, and communication.

# Examples

```rust
use riglr_agents::{AgentSystemBuilder, RoutingStrategy};
use std::time::Duration;

# async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
let system = AgentSystemBuilder::default()
    .with_max_agents(50)
    .with_routing_strategy(RoutingStrategy::LeastLoaded)
    .with_task_timeout(Duration::from_secs(300))
    .build()
    .await?;
# Ok(())
# }
```

---

#### `Capability`

Agent capability definition.

---

#### `ChannelCommunication`

Channel-based communication system using tokio MPSC channels.

This implementation uses in-memory channels for message passing between
agents. It's suitable for single-node deployments and provides high
performance with low latency.

---

#### `CommunicationConfig`

Configuration for communication systems.

---

#### `CommunicationStats`

Statistics about the communication system.

---

#### `CustomAgentSystem`

A complete agent system with custom registry.

---

#### `DebuggableAgent`

A simple wrapper to make rig::Agent debuggable

---

#### `DebuggableCompletionModel`

A wrapper that adds Debug implementation to any CompletionModel

This wrapper allows completion models that don't implement Debug (like ResponsesCompletionModel)
to be used with ToolCallingAgent, which requires Debug for logging and debugging purposes.

## Rationale

The `ToolCallingAgent` and its underlying components require the `CompletionModel` to
implement the `Debug` trait for logging and debugging purposes. However, some models
provided by the `rig` framework (e.g., `ResponsesCompletionModel` used in tests) do not
implement `Debug`.

This wrapper provides a generic `Debug` implementation that allows these non-compliant
models to be used within the `riglr-agents` system without modifying the original `rig`
code. The wrapper displays a placeholder string `<completion_model>` when formatted for
debug output, avoiding the need to expose internal model details while still satisfying
the trait bounds.

## When to Use

Use this wrapper when:
- You encounter a compile error stating that a completion model doesn't implement `Debug`
- You need to use a `rig` completion model with `ToolCallingAgent`
- You're writing tests with mock models that don't implement `Debug`

# Example
```rust,no_run
use rig::providers::openai;
use rig::client::{ProviderClient, CompletionClient};
use riglr_agents::agents::tool_calling::DebuggableCompletionModel;

let openai_client = openai::Client::from_env();
let model = openai_client.completion_model("gpt-4o");
let debuggable_model = DebuggableCompletionModel::new(model);
```

---

#### `DispatchConfig`

Configuration for the agent dispatcher.

---

#### `DispatcherStats`

Statistics about dispatcher state.

---

#### `QueueWorker`

Worker that processes tasks from a queue for a local agent.

---

#### `RedisTaskQueue`

Redis-based implementation of distributed task queue.

---

#### `RegistryConfig`

Configuration for agent registry implementations.

---

#### `RegistryStats`

Statistics about the registry state.

---

#### `RigToolAdapter`

Bridge adapter between riglr_core::Tool and rig::tool::Tool traits.

The RigToolAdapter serves as a crucial bridge between riglr's internal tool system
and the rig framework's agent brain requirements. This adapter pattern allows
riglr tools to be seamlessly integrated into rig agents without modification.

# Architecture

The adapter wraps a `riglr_core::Tool` and implements `rig::tool::Tool`, handling:
- Parameter conversion from rig's format to riglr's format
- Error translation between the two systems
- Context propagation through the ApplicationContext
- SignerContext preservation through spawn_with_context

# Example

```rust,ignore
use riglr_agents::adapter::RigToolAdapter;
use riglr_agents::toolset::Toolset;
use riglr_core::Tool;
use std::sync::Arc;

// Given a riglr tool
let riglr_tool: Arc<dyn Tool> = get_balance_tool();
let context = ApplicationContext::from_env();

// Wrap it in the adapter
let rig_compatible_tool = RigToolAdapter::new(riglr_tool, context);

// Now it can be used in a Toolset for rig agents
let toolset = Toolset::builder()
    .tool(rig_compatible_tool)
    .build();

// The toolset can be given to a rig agent
let agent = rig::agent::Agent::builder()
    .tools(toolset)
    .build();
```

---

#### `Router`

Router for selecting agents based on routing strategies.

---

#### `SignerContextIntegration`

Integration utilities for SignerContext management in multi-agent systems.

---

#### `SystemHealth`

Health status of the agent system.

---

#### `SystemStats`

Combined statistics from all system components.

---

#### `Task`

A task to be executed by an agent.

---

#### `TaskMessage`

Message sent to a remote agent via queue.

---

#### `TaskResponse`

Response from a remote agent.

---

#### `ToolCallingAgentBuilder`

A fluent builder for creating a `ToolCallingAgent`.
This handles all the boilerplate of wiring up the components.

---

#### `ToolExecutor`

Wrapper for executing riglr-core tools within agent contexts.

---

#### `Toolset`

A collection of tools that can be used by an agent.
This is the single source of truth for an agent's capabilities.

---

### Enums

> Enumeration types for representing variants.

#### `AgentError`

Main error type for riglr-agents operations.

**Variants:**

- `AgentNotFound`
  - Agent not found in registry
- `NoSuitableAgent`
  - No suitable agent found for task
- `AgentUnavailable`
  - Agent is not available
- `TaskExecution`
  - Task execution failed
- `TaskTimeout`
  - Task timeout
- `TaskCancelled`
  - Task cancelled
- `InvalidRoutingRule`
  - Invalid routing rule
- `Communication`
  - Communication error
- `MessageDeliveryFailed`
  - Message delivery failed
- `Registry`
  - Registry operation failed
- `Dispatcher`
  - Dispatcher error
- `Configuration`
  - Configuration error
- `Serialization`
  - Serialization error
- `ToolExecutionFailed`
  - A tool executed by the agent failed. This wraps the original
- `Storage`
  - Storage error (e.g., Redis operations)
- `Capacity`
  - Capacity error (e.g., registry full)
- `Generic`
  - Generic error

---

#### `AgentProxy`

Represents either a local or remote agent for the dispatcher.

The dispatcher uses this enum to abstract over whether an agent
is running in the same process (Local) or in a different process
(Remote). This enables the dispatcher to route tasks appropriately.

**Variants:**

- `Local`
  - A local agent that can be called directly
- `Remote`
  - A remote agent represented by its status

---

#### `AgentState`

Agent state enumeration.

**Variants:**

- `Active`
  - Agent is active and ready to accept tasks
- `Busy`
  - Agent is busy but can accept more tasks
- `Full`
  - Agent is at capacity
- `Idle`
  - Agent is idle
- `Offline`
  - Agent is offline/unavailable
- `Maintenance`
  - Agent is in maintenance mode

---

#### `CapabilityType`

Agent capability types with strong typing.

**Variants:**

- `Trading`
  - Trading-related capabilities
- `Research`
  - Research and analysis capabilities
- `RiskAnalysis`
  - Risk assessment and management capabilities
- `Portfolio`
  - Portfolio management capabilities
- `Monitoring`
  - Market monitoring capabilities
- `ToolCalling`
  - Tool calling capabilities for interacting with external systems
- `Custom`
  - Custom capability type

---

#### `MessageFilter`

Message filter for selective message reception.

**Variants:**

- `All`
  - Accept all messages
- `MessageType`
  - Filter by message type
- `Sender`
  - Filter by sender
- `Priority`
  - Filter by priority
- `And`
  - Combine multiple filters with AND logic
- `Or`
  - Combine multiple filters with OR logic

---

#### `Priority`

Priority levels for task execution.

**Variants:**

- `Low`
  - Low priority tasks
- `Normal`
  - Normal priority tasks
- `High`
  - High priority tasks
- `Critical`
  - Critical priority tasks (emergency)

---

#### `RoutingRule`

Rules for routing tasks to agents.

**Variants:**

- `TaskType`
  - Route based on task type
- `Capability`
  - Route based on agent capability
- `Agent`
  - Route to specific agent
- `RoundRobin`
  - Round-robin routing among matching agents
- `LeastLoaded`
  - Route to least loaded agent
- `Priority`
  - Route based on priority
- `Custom`
  - Custom routing logic
- `All`
  - Combination of rules (ALL must match)
- `Any`
  - Alternative rules (ANY can match)

---

#### `RoutingStrategy`

Routing strategies for task dispatch.

**Variants:**

- `Capability`
  - Route based on agent capabilities
- `RoundRobin`
  - Round-robin among capable agents
- `LeastLoaded`
  - Route to least loaded agent
- `Random`
  - Route to random capable agent
- `Direct`
  - Route to specific agent (useful for directed tasks)

---

#### `TaskResult`

Result of task execution.

**Variants:**

- `Success`
  - Task completed successfully
- `Failure`
  - Task failed with error
- `Cancelled`
  - Task was cancelled
- `Timeout`
  - Task timed out

---

#### `TaskType`

Types of tasks that can be executed by agents.

**Variants:**

- `Trading`
  - Trading-related operations
- `Research`
  - Research and analysis tasks
- `RiskAnalysis`
  - Risk assessment and management
- `Portfolio`
  - Portfolio management
- `Monitoring`
  - Market monitoring
- `Custom`
  - Custom task type

---

### Traits

> Trait definitions for implementing common behaviors.

#### `AgentCommunication`

Trait for agent communication implementations.

The communication system enables agents to send messages to each other
for coordination, status updates, and data sharing. Implementations
can use various transport mechanisms (channels, queues, etc.).

**Methods:**

- `send_message()`
  - Send a message to a specific agent.
- `broadcast_message()`
  - Broadcast a message to all agents.
- `subscribe()`
  - Subscribe to messages for a specific agent.
- `unsubscribe()`
  - Unsubscribe from messages for a specific agent.
- `subscription_count()`
  - Get the number of active subscriptions.
- `health_check()`
  - Health check for the communication system.

---

#### `AgentRegistry`

Trait for agent registry implementations.

Registries manage the lifecycle and discovery of agents in the system.
They provide methods to register new agents, discover existing agents,
and query agent status and capabilities.

**Methods:**

- `register_agent()`
  - Register a new agent in the registry.
- `unregister_agent()`
  - Unregister an agent from the registry.
- `get_agent()`
  - Get an agent by its ID.
- `list_agents()`
  - List all registered agents.
- `find_agents_by_capability()`
  - Find agents that can handle a specific capability.
- `get_agent_status()`
  - Get the status of an agent.
- `update_agent_status()`
  - Update the status of an agent.
- `list_agent_statuses()`
  - Get all agent statuses.
- `find_agent_statuses_by_capability()`
  - Find agent statuses by capability (local and remote).
- `is_agent_registered()`
  - Check if an agent is registered.
- `agent_count()`
  - Get the number of registered agents.
- `health_check()`
  - Health check for the registry.

---

#### `DistributedAgentRegistry`

Trait for distributed agent registry implementations.

This trait extends the base `AgentRegistry` trait with additional methods
specific to distributed coordination across multiple processes or machines.

Distributed registries provide capabilities for:
- Cross-process agent discovery
- Remote agent status monitoring
- Distributed capability-based routing
- Load balancing across multiple nodes

# Implementation Note

Implementations of this trait should handle network partitions gracefully
and provide eventual consistency guarantees for distributed operations.

**Methods:**

- `list_agent_statuses_distributed()`
  - List all agent statuses from the distributed registry.
- `find_agent_statuses_by_capability_distributed()`
  - Find agent statuses by capability from the distributed registry.

---

#### `DistributedTaskQueue`

Trait for distributed task queue implementations.

This trait abstracts the underlying message queue mechanism,
allowing for different implementations (Redis, RabbitMQ, NATS, etc.)
while maintaining a consistent interface for the dispatcher.

**Methods:**

- `dispatch_remote_task()`
  - Dispatch a task to a remote agent and wait for the response.

---

#### `IntoAgent`

Trait for objects that can be converted into an Agent.

This allows for flexible agent creation and registration patterns.

**Methods:**

- `into_agent()`
  - Convert this object into an Agent.

---

#### `MessageReceiver`

Trait for receiving messages from the communication system.

**Methods:**

- `receive()`
  - Receive the next message.
- `try_receive()`
  - Try to receive a message without blocking.
- `close()`
  - Close the receiver.
- `is_closed()`
  - Check if the receiver is closed.

---

### Functions

> Standalone functions and utilities.

#### `task_type_to_capability`

Convert task type to capability type.

This function maps task types to their corresponding capability types.

---

### Type Aliases

#### `Result`

Result type alias for riglr-agents operations.

**Type:** `<T, >`

---

### Constants

#### `MONITORING`

Monitoring capability - for agents that can monitor systems and markets

**Type:** `&str`

---

#### `PORTFOLIO`

Portfolio management capability - for agents that can manage portfolios

**Type:** `&str`

---

#### `RESEARCH`

Research capability - for agents that can perform research and analysis

**Type:** `&str`

---

#### `RISK_ANALYSIS`

Risk analysis capability - for agents that can assess and analyze risk

**Type:** `&str`

---

#### `TOOL_CALLING`

Tool calling capability - for agents that can invoke tools and functions

**Type:** `&str`

---

#### `TRADING`

Trading capability - for agents that can execute trading operations

**Type:** `&str`

---
