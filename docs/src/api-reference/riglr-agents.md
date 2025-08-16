# riglr-agents API Reference

Comprehensive API documentation for the `riglr-agents` crate.

## Table of Contents

### Structs

- [`AgentBuilder`](#agentbuilder)
- [`AgentDispatcher`](#agentdispatcher)
- [`AgentId`](#agentid)
- [`AgentMessage`](#agentmessage)
- [`AgentStatus`](#agentstatus)
- [`AgentSystem`](#agentsystem)
- [`Capability`](#capability)
- [`ChannelCommunication`](#channelcommunication)
- [`CommunicationConfig`](#communicationconfig)
- [`CommunicationStats`](#communicationstats)
- [`CustomAgentSystem`](#customagentsystem)
- [`DispatchConfig`](#dispatchconfig)
- [`DispatcherStats`](#dispatcherstats)
- [`DistributedAgentRegistry`](#distributedagentregistry)
- [`LocalAgentRegistry`](#localagentregistry)
- [`RegistryConfig`](#registryconfig)
- [`RegistryStats`](#registrystats)
- [`Router`](#router)
- [`SignerContextIntegration`](#signercontextintegration)
- [`SingleAgentBuilder`](#singleagentbuilder)
- [`SystemHealth`](#systemhealth)
- [`SystemStats`](#systemstats)
- [`Task`](#task)
- [`ToolExecutor`](#toolexecutor)

### Functions

- [`agent`](#agent)
- [`agent_count`](#agent_count)
- [`agent_count`](#agent_count)
- [`agent_error_to_tool_error`](#agent_error_to_tool_error)
- [`agent_id`](#agent_id)
- [`agent_not_found`](#agent_not_found)
- [`agent_unavailable`](#agent_unavailable)
- [`as_str`](#as_str)
- [`broadcast`](#broadcast)
- [`build`](#build)
- [`build_with_registry`](#build_with_registry)
- [`can_execute_tools`](#can_execute_tools)
- [`can_retry`](#can_retry)
- [`cancelled`](#cancelled)
- [`capabilities`](#capabilities)
- [`cleanup`](#cleanup)
- [`communication`](#communication)
- [`communication_with_source`](#communication_with_source)
- [`config`](#config)
- [`config`](#config)
- [`configuration`](#configuration)
- [`data`](#data)
- [`dispatch_task`](#dispatch_task)
- [`dispatch_tasks`](#dispatch_tasks)
- [`dispatcher`](#dispatcher)
- [`dispatcher_with_source`](#dispatcher_with_source)
- [`error`](#error)
- [`execute_job`](#execute_job)
- [`execute_with_signer`](#execute_with_signer)
- [`failure`](#failure)
- [`generate`](#generate)
- [`generic`](#generic)
- [`generic_with_source`](#generic_with_source)
- [`health_check`](#health_check)
- [`health_check`](#health_check)
- [`health_check`](#health_check)
- [`increment_retry`](#increment_retry)
- [`invalid_routing_rule`](#invalid_routing_rule)
- [`is_expired`](#is_expired)
- [`is_past_deadline`](#is_past_deadline)
- [`is_retriable`](#is_retriable)
- [`is_retriable`](#is_retriable)
- [`is_success`](#is_success)
- [`job_result_to_task_result`](#job_result_to_task_result)
- [`job_to_task`](#job_to_task)
- [`matches`](#matches)
- [`matches`](#matches)
- [`matches`](#matches)
- [`message_delivery_failed`](#message_delivery_failed)
- [`metadata`](#metadata)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`no_suitable_agent`](#no_suitable_agent)
- [`register_agent`](#register_agent)
- [`register_agent`](#register_agent)
- [`register_agents`](#register_agents)
- [`register_agents`](#register_agents)
- [`registry`](#registry)
- [`registry_with_source`](#registry_with_source)
- [`retry_delay`](#retry_delay)
- [`select_agent`](#select_agent)
- [`set_strategy`](#set_strategy)
- [`stats`](#stats)
- [`stats`](#stats)
- [`stats`](#stats)
- [`stats`](#stats)
- [`strategy`](#strategy)
- [`success`](#success)
- [`task_cancelled`](#task_cancelled)
- [`task_execution`](#task_execution)
- [`task_execution_with_source`](#task_execution_with_source)
- [`task_result_to_job_result`](#task_result_to_job_result)
- [`task_timeout`](#task_timeout)
- [`task_to_job`](#task_to_job)
- [`timeout`](#timeout)
- [`tool_error_to_agent_error`](#tool_error_to_agent_error)
- [`validate`](#validate)
- [`validate_agent_compatibility`](#validate_agent_compatibility)
- [`with_capabilities`](#with_capabilities)
- [`with_capability`](#with_capability)
- [`with_channel_buffer_size`](#with_channel_buffer_size)
- [`with_config`](#with_config)
- [`with_config`](#with_config)
- [`with_config`](#with_config)
- [`with_config`](#with_config)
- [`with_deadline`](#with_deadline)
- [`with_expiration`](#with_expiration)
- [`with_health_checks`](#with_health_checks)
- [`with_id`](#with_id)
- [`with_load_balancing`](#with_load_balancing)
- [`with_maintenance_interval`](#with_maintenance_interval)
- [`with_max_agents`](#with_max_agents)
- [`with_max_concurrent_tasks`](#with_max_concurrent_tasks)
- [`with_max_pending_messages`](#with_max_pending_messages)
- [`with_max_retries`](#with_max_retries)
- [`with_max_retries`](#with_max_retries)
- [`with_max_subscriptions`](#with_max_subscriptions)
- [`with_message_persistence`](#with_message_persistence)
- [`with_message_ttl`](#with_message_ttl)
- [`with_metadata`](#with_metadata)
- [`with_metadata`](#with_metadata)
- [`with_parameter`](#with_parameter)
- [`with_priority`](#with_priority)
- [`with_priority`](#with_priority)
- [`with_registry_timeout`](#with_registry_timeout)
- [`with_retry_delay`](#with_retry_delay)
- [`with_routing_strategy`](#with_routing_strategy)
- [`with_task_timeout`](#with_task_timeout)
- [`with_timeout`](#with_timeout)

### Enums

- [`AgentError`](#agenterror)
- [`AgentState`](#agentstate)
- [`MessageFilter`](#messagefilter)
- [`Priority`](#priority)
- [`RoutingRule`](#routingrule)
- [`RoutingStrategy`](#routingstrategy)
- [`TaskResult`](#taskresult)
- [`TaskType`](#tasktype)

### Traits

- [`Agent`](#agent)
- [`AgentCommunication`](#agentcommunication)
- [`AgentRegistry`](#agentregistry)
- [`IntoAgent`](#intoagent)
- [`MessageReceiver`](#messagereceiver)

## Structs

### AgentBuilder

**Source**: `src/builder.rs`

```rust
pub struct AgentBuilder { registry_config: RegistryConfig, dispatch_config: DispatchConfig, communication_config: CommunicationConfig, }
```

Builder for creating and configuring agent systems.

The AgentBuilder provides a fluent API for setting up complete
multi-agent systems with registries, dispatchers, and communication.

# Examples

```rust
use riglr_agents::{AgentBuilder, RoutingStrategy};
use std::time::Duration;

# async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
let system = AgentBuilder::new()
.with_max_agents(50)
.with_routing_strategy(RoutingStrategy::LeastLoaded)
.with_task_timeout(Duration::from_secs(300))
.build()
.await?;
# Ok(())
# }
```

---

### AgentDispatcher

**Source**: `dispatcher/mod.rs`

```rust
pub struct AgentDispatcher<R: AgentRegistry> { /// Agent registry registry: Arc<R>, /// Routing engine router: Router, /// Dispatcher configuration config: DispatchConfig, }
```

Agent dispatcher for routing tasks to appropriate agents.

The dispatcher maintains a registry of available agents and routes
incoming tasks based on agent capabilities, load, and routing rules.
It preserves the SignerContext security model by ensuring each task
execution maintains its own signer context.

---

### AgentId

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

```rust
pub struct AgentId(pub String);
```

Unique identifier for an agent in the system.

---

### AgentMessage

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AgentMessage { /// Message ID pub id: String, /// Source agent ID pub from: AgentId, /// Target agent ID (None for broadcast)
```

Message passed between agents.

---

### AgentStatus

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct AgentStatus { /// Agent ID pub agent_id: AgentId, /// Current status pub status: AgentState, /// Number of active tasks pub active_tasks: u32, /// Agent load (0.0 to 1.0)
```

Agent status information.

---

### AgentSystem

**Source**: `src/builder.rs`

```rust
pub struct AgentSystem { /// Agent registry pub registry: Arc<LocalAgentRegistry>, /// Task dispatcher pub dispatcher: AgentDispatcher<LocalAgentRegistry>, /// Communication system pub communication: Arc<ChannelCommunication>, }
```

A complete agent system with local registry.

---

### Capability

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
```

```rust
pub struct Capability { /// Capability name pub name: String, /// Capability version pub version: String, /// Optional capability parameters pub parameters: HashMap<String, serde_json::Value>, }
```

Agent capability definition.

---

### ChannelCommunication

**Source**: `communication/channels.rs`

```rust
pub struct ChannelCommunication { /// Message channels per agent channels: RwLock<HashMap<AgentId, mpsc::UnboundedSender<AgentMessage>>>, /// Configuration config: CommunicationConfig, /// Statistics #[allow(dead_code)]
```

Channel-based communication system using tokio MPSC channels.

This implementation uses in-memory channels for message passing between
agents. It's suitable for single-node deployments and provides high
performance with low latency.

---

### CommunicationConfig

**Source**: `communication/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct CommunicationConfig { /// Maximum number of pending messages per agent pub max_pending_messages: usize, /// Message time-to-live pub message_ttl: std::time::Duration, /// Enable message persistence pub enable_persistence: bool, /// Buffer size for channels pub channel_buffer_size: usize, /// Maximum number of concurrent subscriptions pub max_subscriptions: Option<usize>, }
```

Configuration for communication systems.

---

### CommunicationStats

**Source**: `communication/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct CommunicationStats { /// Number of active subscriptions pub active_subscriptions: usize, /// Total messages sent pub messages_sent: u64, /// Total messages received pub messages_received: u64, /// Number of failed message deliveries pub failed_deliveries: u64, /// Number of expired messages pub expired_messages: u64, }
```

Statistics about the communication system.

---

### CustomAgentSystem

**Source**: `src/builder.rs`

```rust
pub struct CustomAgentSystem<R: AgentRegistry> { /// Agent registry pub registry: Arc<R>, /// Task dispatcher pub dispatcher: AgentDispatcher<R>, /// Communication system pub communication: Arc<ChannelCommunication>, }
```

A complete agent system with custom registry.

---

### DispatchConfig

**Source**: `dispatcher/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct DispatchConfig { /// Default timeout for task execution pub default_task_timeout: Duration, /// Maximum number of retry attempts for failed tasks pub max_retries: u32, /// Delay between retry attempts pub retry_delay: Duration, /// Maximum number of concurrent tasks per agent pub max_concurrent_tasks_per_agent: u32, /// Enable load balancing pub enable_load_balancing: bool, /// Routing strategy to use pub routing_strategy: RoutingStrategy, }
```

Configuration for the agent dispatcher.

---

### DispatcherStats

**Source**: `dispatcher/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct DispatcherStats { /// Number of registered agents pub registered_agents: usize, /// Total active tasks across all agents pub total_active_tasks: u32, /// Average load across all agents pub average_agent_load: f64, /// Current routing strategy pub routing_strategy: RoutingStrategy, }
```

Statistics about dispatcher state.

---

### DistributedAgentRegistry

**Source**: `registry/distributed.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct DistributedAgentRegistry { _config: RegistryConfig, }
```

Distributed agent registry using Redis backend.

This registry enables multi-node deployments with shared agent state
across multiple riglr-agents instances. Agents can be registered on
one node and discovered by other nodes in the cluster.

**Note**: This is currently a stub implementation. Full Redis-based
distributed registry will be implemented in a future release.

---

### LocalAgentRegistry

**Source**: `registry/local.rs`

```rust
pub struct LocalAgentRegistry { /// Registered agents agents: RwLock<HashMap<AgentId, Arc<dyn Agent>>>, /// Agent status information statuses: RwLock<HashMap<AgentId, AgentStatus>>, /// Registry configuration config: RegistryConfig, }
```

In-memory agent registry for single-node deployments.

This registry stores all agent information in memory and provides
fast access to agent data. It's suitable for development, testing,
and single-node production deployments.

---

### RegistryConfig

**Source**: `registry/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RegistryConfig { /// Maximum number of agents that can be registered pub max_agents: Option<usize>, /// Timeout for registry operations pub operation_timeout: std::time::Duration, /// Whether to enable health checks pub enable_health_checks: bool, /// Interval for background maintenance tasks pub maintenance_interval: std::time::Duration, }
```

Configuration for agent registry implementations.

---

### RegistryStats

**Source**: `registry/local.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RegistryStats { /// Total number of registered agents pub total_agents: usize, /// Number of active agents pub active_agents: usize, /// Number of busy agents pub busy_agents: usize, /// Number of idle agents pub idle_agents: usize, /// Number of offline agents pub offline_agents: usize, }
```

Statistics about the registry state.

---

### Router

**Source**: `dispatcher/router.rs`

```rust
pub struct Router { strategy: RoutingStrategy, round_robin_counter: AtomicUsize, }
```

Router for selecting agents based on routing strategies.

---

### SignerContextIntegration

**Source**: `src/integration.rs`

```rust
pub struct SignerContextIntegration;
```

Integration utilities for SignerContext management in multi-agent systems.

---

### SingleAgentBuilder

**Source**: `src/builder.rs`

```rust
pub struct SingleAgentBuilder { agent_id: Option<String>, capabilities: Vec<String>, metadata: std::collections::HashMap<String, serde_json::Value>, }
```

Builder for creating individual agents with common patterns.

---

### SystemHealth

**Source**: `src/builder.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SystemHealth { /// Registry health status pub registry_healthy: bool, /// Dispatcher health status pub dispatcher_healthy: bool, /// Communication health status pub communication_healthy: bool, /// Overall system health pub overall_healthy: bool, }
```

Health status of the agent system.

---

### SystemStats

**Source**: `src/builder.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SystemStats { /// Registry statistics pub registry_stats: crate::registry::local::RegistryStats, /// Dispatcher statistics pub dispatcher_stats: crate::dispatcher::DispatcherStats, /// Communication statistics pub communication_stats: crate::communication::CommunicationStats, }
```

Combined statistics from all system components.

---

### Task

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct Task { /// Unique task identifier pub id: String, /// Type of task pub task_type: TaskType, /// Task parameters pub parameters: serde_json::Value, /// Task priority pub priority: Priority, /// Maximum execution timeout pub timeout: Option<Duration>, /// Number of retry attempts allowed pub max_retries: u32, /// Current retry count pub retry_count: u32, /// Timestamp when task was created pub created_at: chrono::DateTime<chrono::Utc>, /// Optional deadline for task completion pub deadline: Option<chrono::DateTime<chrono::Utc>>, /// Metadata for the task pub metadata: HashMap<String, serde_json::Value>, }
```

A task to be executed by an agent.

---

### ToolExecutor

**Source**: `src/integration.rs`

```rust
pub struct ToolExecutor { agent: Arc<dyn Agent>, }
```

Wrapper for executing riglr-core tools within agent contexts.

---

## Functions

### agent

**Source**: `src/integration.rs`

```rust
pub fn agent(&self) -> &Arc<dyn Agent>
```

Get the underlying agent.

---

### agent_count

**Source**: `src/builder.rs`

```rust
pub async fn agent_count(&self) -> Result<usize>
```

Get the number of registered agents.

---

### agent_count

**Source**: `src/builder.rs`

```rust
pub async fn agent_count(&self) -> Result<usize>
```

Get the number of registered agents.

---

### agent_error_to_tool_error

**Source**: `src/integration.rs`

```rust
pub fn agent_error_to_tool_error(agent_error: AgentError) -> ToolError
```

Convert an AgentError to a ToolError for riglr-core integration.

# Arguments

* `agent_error` - The agent error to convert

# Returns

A ToolError representation suitable for riglr-core systems.

---

### agent_id

**Source**: `src/builder.rs`

```rust
pub fn agent_id(&self) -> Option<&str>
```

Get the configured agent ID.

---

### agent_not_found

**Source**: `src/error.rs`

```rust
pub fn agent_not_found(agent_id: impl Into<String>) -> Self
```

Create an agent not found error.

---

### agent_unavailable

**Source**: `src/error.rs`

```rust
pub fn agent_unavailable(agent_id: impl Into<String>, status: impl Into<String>) -> Self
```

Create an agent unavailable error.

---

### as_str

**Source**: `src/types.rs`

```rust
pub fn as_str(&self) -> &str
```

Get the inner string value.

---

### broadcast

**Source**: `src/types.rs`

```rust
pub fn broadcast(from: AgentId, message_type: String, payload: serde_json::Value) -> Self
```

Create a broadcast message (no specific recipient).

---

### build

**Source**: `src/builder.rs`

```rust
pub async fn build(self) -> Result<AgentSystem>
```

Build the agent system with the configured settings.

# Returns

A complete agent system ready for use.

---

### build_with_registry

**Source**: `src/builder.rs`

```rust
pub async fn build_with_registry<R: AgentRegistry + 'static>( self, registry: Arc<R>, ) -> Result<CustomAgentSystem<R>>
```

Build with a custom registry implementation.

# Arguments

* `registry` - Custom registry implementation to use

# Returns

A complete agent system with the custom registry.

---

### can_execute_tools

**Source**: `src/integration.rs`

```rust
pub fn can_execute_tools(agent: &dyn Agent, required_capabilities: &[String]) -> bool
```

Check if an agent can execute riglr-core tools.

This validates that the agent has the necessary capabilities
to execute tools from the riglr ecosystem.

# Arguments

* `agent` - The agent to check
* `required_capabilities` - List of required capabilities

# Returns

true if the agent can execute the tools, false otherwise.

---

### can_retry

**Source**: `src/types.rs`

```rust
pub fn can_retry(&self) -> bool
```

Check if the task can be retried.

---

### cancelled

**Source**: `src/types.rs`

```rust
pub fn cancelled(reason: String) -> Self
```

Create a cancelled task result.

---

### capabilities

**Source**: `src/builder.rs`

```rust
pub fn capabilities(&self) -> &[String]
```

Get the configured capabilities.

---

### cleanup

**Source**: `communication/channels.rs`

```rust
pub async fn cleanup(&self) -> Result<usize>
```

Clean up expired messages and closed channels.

---

### communication

**Source**: `src/error.rs`

```rust
pub fn communication(message: impl Into<String>) -> Self
```

Create a communication error.

---

### communication_with_source

**Source**: `src/error.rs`

```rust
pub fn communication_with_source<E>(message: impl Into<String>, source: E) -> Self where E: std::error::Error + Send + Sync + 'static,
```

Create a communication error with source.

---

### config

**Source**: `dispatcher/mod.rs`

```rust
pub fn config(&self) -> &DispatchConfig
```

Get the current configuration.

---

### config

**Source**: `registry/local.rs`

```rust
pub fn config(&self) -> &RegistryConfig
```

Get the current configuration.

---

### configuration

**Source**: `src/error.rs`

```rust
pub fn configuration(message: impl Into<String>) -> Self
```

Create a configuration error.

---

### data

**Source**: `src/types.rs`

```rust
pub fn data(&self) -> Option<&serde_json::Value>
```

Get the data from a successful result.

---

### dispatch_task

**Source**: `dispatcher/mod.rs`

```rust
pub async fn dispatch_task(&self, mut task: Task) -> Result<TaskResult>
```

Dispatch a task to an appropriate agent.

This method:
1. Finds suitable agents for the task
2. Selects the best agent using the routing strategy
3. Executes the task while preserving SignerContext
4. Handles retries on retriable failures

# Arguments

* `task` - The task to dispatch

# Returns

The task result from the executing agent.

---

### dispatch_tasks

**Source**: `dispatcher/mod.rs`

```rust
pub async fn dispatch_tasks(&self, tasks: Vec<Task>) -> Vec<Result<TaskResult>>
```

Dispatch multiple tasks concurrently.

# Arguments

* `tasks` - Vector of tasks to dispatch

# Returns

Vector of task results in the same order as input tasks.

---

### dispatcher

**Source**: `src/error.rs`

```rust
pub fn dispatcher(message: impl Into<String>) -> Self
```

Create a dispatcher error.

---

### dispatcher_with_source

**Source**: `src/error.rs`

```rust
pub fn dispatcher_with_source<E>(message: impl Into<String>, source: E) -> Self where E: std::error::Error + Send + Sync + 'static,
```

Create a dispatcher error with source.

---

### error

**Source**: `src/types.rs`

```rust
pub fn error(&self) -> Option<&str>
```

Get the error message from a failed result.

---

### execute_job

**Source**: `src/integration.rs`

```rust
pub async fn execute_job(&self, job: Job) -> Result<JobResult>
```

Execute a riglr-core job using the agent.

This method bridges riglr-core's Job system with the agent execution model.

# Arguments

* `job` - The riglr-core job to execute

# Returns

A riglr-core JobResult from the execution.

---

### execute_with_signer

**Source**: `src/integration.rs`

```rust
pub async fn execute_with_signer( signer: Arc<dyn TransactionSigner>, agent: Arc<dyn Agent>, task: Task, ) -> Result<TaskResult>
```

Execute an agent task within a signer context.

This method ensures that the agent's task execution maintains the
proper signer context isolation while integrating with riglr-core's
security model.

# Arguments

* `signer` - The signer to use for the task execution
* `agent` - The agent to execute the task
* `task` - The task to execute

# Returns

The task result from the agent execution.

---

### failure

**Source**: `src/types.rs`

```rust
pub fn failure(error: String, retriable: bool, duration: Duration) -> Self
```

Create a failed task result.

---

### generate

**Source**: `src/types.rs`

```rust
pub fn generate() -> Self
```

Generate a random agent ID.

---

### generic

**Source**: `src/error.rs`

```rust
pub fn generic(message: impl Into<String>) -> Self
```

Create a generic error.

---

### generic_with_source

**Source**: `src/error.rs`

```rust
pub fn generic_with_source<E>(message: impl Into<String>, source: E) -> Self where E: std::error::Error + Send + Sync + 'static,
```

Create a generic error with source.

---

### health_check

**Source**: `src/builder.rs`

```rust
pub async fn health_check(&self) -> Result<SystemHealth>
```

Perform a health check on all system components.

---

### health_check

**Source**: `src/builder.rs`

```rust
pub async fn health_check(&self) -> Result<SystemHealth>
```

Perform a health check on all system components.

---

### health_check

**Source**: `dispatcher/mod.rs`

```rust
pub async fn health_check(&self) -> Result<bool>
```

Health check for the dispatcher.

---

### increment_retry

**Source**: `src/types.rs`

```rust
pub fn increment_retry(&mut self)
```

Increment the retry count.

---

### invalid_routing_rule

**Source**: `src/error.rs`

```rust
pub fn invalid_routing_rule(rule: impl Into<String>) -> Self
```

Create an invalid routing rule error.

---

### is_expired

**Source**: `src/types.rs`

```rust
pub fn is_expired(&self) -> bool
```

Check if the message has expired.

---

### is_past_deadline

**Source**: `src/types.rs`

```rust
pub fn is_past_deadline(&self) -> bool
```

Check if the task has exceeded its deadline.

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if this error is retriable.

Some agent errors represent temporary conditions that may succeed on retry.

---

### is_retriable

**Source**: `src/types.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the result represents a retriable failure.

---

### is_success

**Source**: `src/types.rs`

```rust
pub fn is_success(&self) -> bool
```

Check if the result represents success.

---

### job_result_to_task_result

**Source**: `src/integration.rs`

```rust
pub fn job_result_to_task_result( job_result: &JobResult, duration: std::time::Duration, ) -> TaskResult
```

Convert a riglr-core JobResult to an agent TaskResult.

# Arguments

* `job_result` - The riglr-core job result to convert
* `duration` - The execution duration to include in the task result

# Returns

An agent TaskResult representation.

---

### job_to_task

**Source**: `src/integration.rs`

```rust
pub fn job_to_task(job: &Job) -> Result<Task>
```

Convert a riglr-core Job to an agent Task.

This enables riglr-core jobs to be executed by agents.

# Arguments

* `job` - The riglr-core job to convert

# Returns

An agent Task representation of the job.

---

### matches

**Source**: `src/types.rs`

```rust
pub fn matches(&self, other: &TaskType) -> bool
```

Check if this task type matches another (including wildcard matching).

---

### matches

**Source**: `src/types.rs`

```rust
pub fn matches(&self, task: &Task, agent_id: &AgentId, capabilities: &[String]) -> bool
```

Check if this rule matches the given task and agent context.

---

### matches

**Source**: `communication/mod.rs`

```rust
pub fn matches(&self, message: &AgentMessage) -> bool
```

Check if a message matches this filter.

---

### message_delivery_failed

**Source**: `src/error.rs`

```rust
pub fn message_delivery_failed( message_id: impl Into<String>, agent_id: impl Into<String>, ) -> Self
```

Create a message delivery failed error.

---

### metadata

**Source**: `src/builder.rs`

```rust
pub fn metadata(&self) -> &std::collections::HashMap<String, serde_json::Value>
```

Get the configured metadata.

---

### new

**Source**: `src/builder.rs`

```rust
pub fn new() -> Self
```

Create a new agent builder with default configuration.

---

### new

**Source**: `src/builder.rs`

```rust
pub fn new() -> Self
```

Create a new single agent builder.

---

### new

**Source**: `src/integration.rs`

```rust
pub fn new(agent: Arc<dyn Agent>) -> Self
```

Create a new tool executor for the given agent.

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(id: impl Into<String>) -> Self
```

Create a new agent ID from a string.

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(task_type: TaskType, parameters: serde_json::Value) -> Self
```

Create a new task with the given type and parameters.

---

### new

**Source**: `src/types.rs`

```rust
pub fn new( from: AgentId, to: Option<AgentId>, message_type: String, payload: serde_json::Value, ) -> Self
```

Create a new message.

---

### new

**Source**: `src/types.rs`

```rust
pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self
```

Create a new capability.

---

### new

**Source**: `communication/channels.rs`

```rust
pub fn new() -> Self
```

Create a new channel-based communication system.

---

### new

**Source**: `dispatcher/mod.rs`

```rust
pub fn new(registry: Arc<R>) -> Self
```

Create a new agent dispatcher with the given registry.

---

### new

**Source**: `dispatcher/router.rs`

```rust
pub fn new(strategy: RoutingStrategy) -> Self
```

Create a new router with the specified strategy.

---

### new

**Source**: `registry/distributed.rs`

```rust
pub fn new(_redis_url: String) -> Result<Self>
```

Create a new distributed agent registry.

**Note**: This is currently a stub implementation.

---

### new

**Source**: `registry/local.rs`

```rust
pub fn new() -> Self
```

Create a new local agent registry with default configuration.

---

### no_suitable_agent

**Source**: `src/error.rs`

```rust
pub fn no_suitable_agent(task_type: impl Into<String>) -> Self
```

Create a no suitable agent error.

---

### register_agent

**Source**: `src/builder.rs`

```rust
pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()>
```

Register an agent in the system.

---

### register_agent

**Source**: `src/builder.rs`

```rust
pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()>
```

Register an agent in the system.

---

### register_agents

**Source**: `src/builder.rs`

```rust
pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()>
```

Register multiple agents in the system.

---

### register_agents

**Source**: `src/builder.rs`

```rust
pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()>
```

Register multiple agents in the system.

---

### registry

**Source**: `src/error.rs`

```rust
pub fn registry(operation: impl Into<String>) -> Self
```

Create a registry error.

---

### registry_with_source

**Source**: `src/error.rs`

```rust
pub fn registry_with_source<E>(operation: impl Into<String>, source: E) -> Self where E: std::error::Error + Send + Sync + 'static,
```

Create a registry error with source.

---

### retry_delay

**Source**: `src/error.rs`

```rust
pub fn retry_delay(&self) -> Option<std::time::Duration>
```

Get the retry delay for retriable errors.

---

### select_agent

**Source**: `dispatcher/router.rs`

```rust
pub async fn select_agent( &self, agents: &[Arc<dyn Agent>], task: &Task, ) -> Result<Arc<dyn Agent>>
```

Select an agent from the available agents using the configured strategy.

# Arguments

* `agents` - Available agents to choose from
* `task` - The task to be executed (used for routing decisions)

# Returns

The selected agent.

---

### set_strategy

**Source**: `dispatcher/router.rs`

```rust
pub fn set_strategy(&mut self, strategy: RoutingStrategy)
```

Change the routing strategy.

---

### stats

**Source**: `src/builder.rs`

```rust
pub async fn stats(&self) -> Result<SystemStats>
```

Get system statistics.

---

### stats

**Source**: `communication/channels.rs`

```rust
pub async fn stats(&self) -> CommunicationStats
```

Get current statistics.

---

### stats

**Source**: `dispatcher/mod.rs`

```rust
pub async fn stats(&self) -> Result<DispatcherStats>
```

Get statistics about the dispatcher.

---

### stats

**Source**: `registry/local.rs`

```rust
pub async fn stats(&self) -> RegistryStats
```

Get statistics about the registry.

---

### strategy

**Source**: `dispatcher/router.rs`

```rust
pub fn strategy(&self) -> RoutingStrategy
```

Get the current routing strategy.

---

### success

**Source**: `src/types.rs`

```rust
pub fn success(data: serde_json::Value, tx_hash: Option<String>, duration: Duration) -> Self
```

Create a successful task result.

---

### task_cancelled

**Source**: `src/error.rs`

```rust
pub fn task_cancelled(task_id: impl Into<String>, reason: impl Into<String>) -> Self
```

Create a task cancelled error.

---

### task_execution

**Source**: `src/error.rs`

```rust
pub fn task_execution(message: impl Into<String>) -> Self
```

Create a task execution error.

---

### task_execution_with_source

**Source**: `src/error.rs`

```rust
pub fn task_execution_with_source<E>(message: impl Into<String>, source: E) -> Self where E: std::error::Error + Send + Sync + 'static,
```

Create a task execution error with source.

---

### task_result_to_job_result

**Source**: `src/integration.rs`

```rust
pub fn task_result_to_job_result(task_result: &TaskResult) -> Result<JobResult>
```

Convert an agent TaskResult to a riglr-core JobResult.

# Arguments

* `task_result` - The agent task result to convert

# Returns

A riglr-core JobResult representation.

---

### task_timeout

**Source**: `src/error.rs`

```rust
pub fn task_timeout(task_id: impl Into<String>, duration: std::time::Duration) -> Self
```

Create a task timeout error.

---

### task_to_job

**Source**: `src/integration.rs`

```rust
pub fn task_to_job(task: &Task) -> Result<Job>
```

Convert an agent task to a riglr-core Job.

This enables agents to be integrated with riglr-core's job queue system.

# Arguments

* `task` - The agent task to convert

# Returns

A riglr-core Job representation of the task.

---

### timeout

**Source**: `src/types.rs`

```rust
pub fn timeout(duration: Duration) -> Self
```

Create a timeout task result.

---

### tool_error_to_agent_error

**Source**: `src/integration.rs`

```rust
pub fn tool_error_to_agent_error(tool_error: ToolError) -> AgentError
```

Convert a ToolError to an AgentError.

# Arguments

* `tool_error` - The tool error to convert

# Returns

An AgentError representation.

---

### validate

**Source**: `src/builder.rs`

```rust
pub fn validate(&self) -> Result<()>
```

Validate the builder configuration.

---

### validate_agent_compatibility

**Source**: `src/integration.rs`

```rust
pub fn validate_agent_compatibility(agent: &dyn Agent) -> Result<()>
```

Validate that an agent is compatible with riglr-core patterns.

This performs various compatibility checks to ensure the agent
can work properly within the riglr ecosystem.

# Arguments

* `agent` - The agent to validate

# Returns

Ok(()) if the agent is compatible, Err with details otherwise.

---

### with_capabilities

**Source**: `src/builder.rs`

```rust
pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self
```

Add multiple capabilities to the agent.

---

### with_capability

**Source**: `src/builder.rs`

```rust
pub fn with_capability(mut self, capability: impl Into<String>) -> Self
```

Add a capability to the agent.

---

### with_channel_buffer_size

**Source**: `src/builder.rs`

```rust
pub fn with_channel_buffer_size(mut self, size: usize) -> Self
```

Set the channel buffer size for communication.

---

### with_config

**Source**: `communication/channels.rs`

```rust
pub fn with_config(config: CommunicationConfig) -> Self
```

Create a new channel-based communication system with configuration.

---

### with_config

**Source**: `dispatcher/mod.rs`

```rust
pub fn with_config(registry: Arc<R>, config: DispatchConfig) -> Self
```

Create a new agent dispatcher with custom configuration.

---

### with_config

**Source**: `registry/distributed.rs`

```rust
pub fn with_config(_redis_url: String, config: RegistryConfig) -> Result<Self>
```

Create a new distributed agent registry with configuration.

**Note**: This is currently a stub implementation.

---

### with_config

**Source**: `registry/local.rs`

```rust
pub fn with_config(config: RegistryConfig) -> Self
```

Create a new local agent registry with custom configuration.

---

### with_deadline

**Source**: `src/types.rs`

```rust
pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self
```

Set the task deadline.

---

### with_expiration

**Source**: `src/types.rs`

```rust
pub fn with_expiration(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self
```

Set message expiration.

---

### with_health_checks

**Source**: `src/builder.rs`

```rust
pub fn with_health_checks(mut self, enabled: bool) -> Self
```

Enable or disable registry health checks.

---

### with_id

**Source**: `src/builder.rs`

```rust
pub fn with_id(mut self, id: impl Into<String>) -> Self
```

Set the agent ID.

---

### with_load_balancing

**Source**: `src/builder.rs`

```rust
pub fn with_load_balancing(mut self, enabled: bool) -> Self
```

Enable or disable load balancing.

---

### with_maintenance_interval

**Source**: `src/builder.rs`

```rust
pub fn with_maintenance_interval(mut self, interval: Duration) -> Self
```

Set the registry maintenance interval.

---

### with_max_agents

**Source**: `src/builder.rs`

```rust
pub fn with_max_agents(mut self, max_agents: usize) -> Self
```

Set the maximum number of agents in the registry.

---

### with_max_concurrent_tasks

**Source**: `src/builder.rs`

```rust
pub fn with_max_concurrent_tasks(mut self, max_tasks: u32) -> Self
```

Set the maximum number of concurrent tasks per agent.

---

### with_max_pending_messages

**Source**: `src/builder.rs`

```rust
pub fn with_max_pending_messages(mut self, max_messages: usize) -> Self
```

Set the maximum number of pending messages per agent.

---

### with_max_retries

**Source**: `src/builder.rs`

```rust
pub fn with_max_retries(mut self, max_retries: u32) -> Self
```

Set the maximum number of retry attempts for failed tasks.

---

### with_max_retries

**Source**: `src/types.rs`

```rust
pub fn with_max_retries(mut self, max_retries: u32) -> Self
```

Set the maximum retry count.

---

### with_max_subscriptions

**Source**: `src/builder.rs`

```rust
pub fn with_max_subscriptions(mut self, max_subs: usize) -> Self
```

Set the maximum number of communication subscriptions.

---

### with_message_persistence

**Source**: `src/builder.rs`

```rust
pub fn with_message_persistence(mut self, enabled: bool) -> Self
```

Enable or disable message persistence.

---

### with_message_ttl

**Source**: `src/builder.rs`

```rust
pub fn with_message_ttl(mut self, ttl: Duration) -> Self
```

Set the message time-to-live.

---

### with_metadata

**Source**: `src/builder.rs`

```rust
pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
```

Add metadata to the agent.

---

### with_metadata

**Source**: `src/types.rs`

```rust
pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
```

Add metadata to the task.

---

### with_parameter

**Source**: `src/types.rs`

```rust
pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self
```

Add a parameter to the capability.

---

### with_priority

**Source**: `src/types.rs`

```rust
pub fn with_priority(mut self, priority: Priority) -> Self
```

Set the task priority.

---

### with_priority

**Source**: `src/types.rs`

```rust
pub fn with_priority(mut self, priority: Priority) -> Self
```

Set message priority.

---

### with_registry_timeout

**Source**: `src/builder.rs`

```rust
pub fn with_registry_timeout(mut self, timeout: Duration) -> Self
```

Set the registry operation timeout.

---

### with_retry_delay

**Source**: `src/builder.rs`

```rust
pub fn with_retry_delay(mut self, delay: Duration) -> Self
```

Set the delay between retry attempts.

---

### with_routing_strategy

**Source**: `src/builder.rs`

```rust
pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self
```

Set the routing strategy for task dispatch.

---

### with_task_timeout

**Source**: `src/builder.rs`

```rust
pub fn with_task_timeout(mut self, timeout: Duration) -> Self
```

Set the default task execution timeout.

---

### with_timeout

**Source**: `src/types.rs`

```rust
pub fn with_timeout(mut self, timeout: Duration) -> Self
```

Set the task timeout.

---

## Enums

### AgentError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum AgentError { /// Agent not found in registry #[error("Agent '{agent_id}' not found in registry")] AgentNotFound { agent_id: String }, /// No suitable agent found for task #[error("No agent found capable of handling task type '{task_type}'")] NoSuitableAgent { task_type: String }, /// Agent is not available #[error("Agent '{agent_id}' is not available (status: {status})")] AgentUnavailable { agent_id: String, status: String }, /// Task execution failed #[error("Task execution failed: {message}")] TaskExecution { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>>, }, /// Task timeout #[error("Task '{task_id}' timed out after {duration:?}")] TaskTimeout { task_id: String, duration: std::time::Duration, }, /// Task cancelled #[error("Task '{task_id}' was cancelled: {reason}")] TaskCancelled { task_id: String, reason: String }, /// Invalid routing rule #[error("Invalid routing rule: {rule}")] InvalidRoutingRule { rule: String }, /// Communication error #[error("Communication error: {message}")] Communication { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>>, }, /// Message delivery failed #[error("Failed to deliver message '{message_id}' to agent '{agent_id}'")] MessageDeliveryFailed { message_id: String, agent_id: String, }, /// Registry operation failed #[error("Registry operation failed: {operation}")] Registry { operation: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>>, }, /// Dispatcher error #[error("Dispatcher error: {message}")] Dispatcher { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>>, }, /// Configuration error #[error("Configuration error: {message}")] Configuration { message: String }, /// Serialization error #[error("Serialization error")] Serialization { #[from] source: serde_json::Error, }, /// Tool error (from riglr-core) #[error("Tool error")] Tool { #[from] source: ToolError, }, /// Generic error #[error("Agent system error: {message}")] Generic { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>>, }, }
```

Main error type for riglr-agents operations.

**Variants**:

- `AgentNotFound`
- `NoSuitableAgent`
- `AgentUnavailable`
- `TaskExecution`
- `message`
- `source`
- `TaskTimeout`
- `task_id`
- `duration`
- `TaskCancelled`
- `InvalidRoutingRule`
- `Communication`
- `message`
- `source`
- `MessageDeliveryFailed`
- `message_id`
- `agent_id`
- `Registry`
- `operation`
- `source`
- `Dispatcher`
- `message`
- `source`
- `Configuration`
- `Serialization`
- `source`
- `Tool`
- `source`
- `Generic`
- `message`
- `source`

---

### AgentState

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
```

```rust
pub enum AgentState { /// Agent is active and ready to accept tasks Active, /// Agent is busy but can accept more tasks Busy, /// Agent is at capacity Full, /// Agent is idle #[default] Idle, /// Agent is offline/unavailable Offline, /// Agent is in maintenance mode Maintenance, }
```

Agent state enumeration.

**Variants**:

- `Active`
- `Busy`
- `Full`
- `Idle`
- `Offline`
- `Maintenance`

---

### MessageFilter

**Source**: `communication/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum MessageFilter { /// Accept all messages All, /// Filter by message type MessageType(String), /// Filter by sender Sender(AgentId), /// Filter by priority Priority(crate::types::Priority), /// Combine multiple filters with AND logic And(Vec<MessageFilter>), /// Combine multiple filters with OR logic Or(Vec<MessageFilter>), }
```

Message filter for selective message reception.

**Variants**:

- `All`
- `MessageType(String)`
- `Sender(AgentId)`
- `Priority(crate::types::Priority)`
- `And(Vec<MessageFilter>)`
- `Or(Vec<MessageFilter>)`

---

### Priority

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
```

```rust
pub enum Priority { /// Low priority tasks Low = 1, /// Normal priority tasks #[default] Normal = 2, /// High priority tasks High = 3, /// Critical priority tasks (emergency) Critical = 4, }
```

Priority levels for task execution.

**Variants**:

- `Low`
- `Normal`
- `High`
- `Critical`

---

### RoutingRule

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub enum RoutingRule { /// Route based on task type TaskType(TaskType), /// Route based on agent capability Capability(String), /// Route to specific agent Agent(AgentId), /// Round-robin routing among matching agents RoundRobin, /// Route to least loaded agent LeastLoaded, /// Route based on priority Priority(Priority), /// Custom routing logic Custom(String), /// Combination of rules (ALL must match) All(Vec<RoutingRule>), /// Alternative rules (ANY can match) Any(Vec<RoutingRule>), }
```

Rules for routing tasks to agents.

**Variants**:

- `TaskType(TaskType)`
- `Capability(String)`
- `Agent(AgentId)`
- `RoundRobin`
- `LeastLoaded`
- `Priority(Priority)`
- `Custom(String)`
- `All(Vec<RoutingRule>)`
- `Any(Vec<RoutingRule>)`

---

### RoutingStrategy

**Source**: `dispatcher/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum RoutingStrategy { /// Route based on agent capabilities Capability, /// Round-robin among capable agents RoundRobin, /// Route to least loaded agent LeastLoaded, /// Route to random capable agent Random, /// Route to specific agent (useful for directed tasks) Direct, }
```

Routing strategies for task dispatch.

**Variants**:

- `Capability`
- `RoundRobin`
- `LeastLoaded`
- `Random`
- `Direct`

---

### TaskResult

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub enum TaskResult { /// Task completed successfully Success { /// Result data data: serde_json::Value, /// Optional transaction hash tx_hash: Option<String>, /// Execution duration duration: Duration, }, /// Task failed with error Failure { /// Error message error: String, /// Whether the failure is retriable retriable: bool, /// Execution duration before failure duration: Duration, }, /// Task was cancelled Cancelled { /// Cancellation reason reason: String, }, /// Task timed out Timeout { /// Timeout duration duration: Duration, }, }
```

Result of task execution.

**Variants**:

- `Success`
- `data`
- `tx_hash`
- `duration`
- `Failure`
- `error`
- `retriable`
- `duration`
- `Cancelled`
- `reason`
- `Timeout`
- `duration`

---

### TaskType

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

```rust
pub enum TaskType { /// Trading-related operations Trading, /// Research and analysis tasks Research, /// Risk assessment and management RiskAnalysis, /// Portfolio management Portfolio, /// Market monitoring Monitoring, /// Custom task type Custom(String), }
```

Types of tasks that can be executed by agents.

**Variants**:

- `Trading`
- `Research`
- `RiskAnalysis`
- `Portfolio`
- `Monitoring`
- `Custom(String)`

---

## Traits

### Agent

**Source**: `src/lib.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait Agent: Send + Sync { ... }
```

Core trait that all agents must implement.

Agents are autonomous units that can execute tasks and participate in
multi-agent workflows. They maintain their own state and capabilities
while respecting the SignerContext security model.

**Methods**:

#### `execute_task`

```rust
async fn execute_task(&self, task: Task) -> Result<TaskResult>;
```

#### `id`

```rust
fn id(&self) -> &AgentId;
```

#### `capabilities`

```rust
fn capabilities(&self) -> Vec<String>;
```

#### `status`

```rust
fn status(&self) -> AgentStatus {
```

#### `before_task`

```rust
async fn before_task(&self, _task: &Task) -> Result<()> {
```

#### `after_task`

```rust
async fn after_task(&self, _task: &Task, _result: &TaskResult) -> Result<()> {
```

#### `handle_message`

```rust
async fn handle_message(&self, _message: AgentMessage) -> Result<()> {
```

#### `can_handle`

```rust
fn can_handle(&self, task: &Task) -> bool {
```

#### `load`

```rust
fn load(&self) -> f64 {
```

#### `is_available`

```rust
fn is_available(&self) -> bool {
```

---

### AgentCommunication

**Source**: `communication/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait AgentCommunication: Send + Sync { ... }
```

Trait for agent communication implementations.

The communication system enables agents to send messages to each other
for coordination, status updates, and data sharing. Implementations
can use various transport mechanisms (channels, queues, etc.).

**Methods**:

#### `send_message`

```rust
async fn send_message(&self, message: AgentMessage) -> Result<()>;
```

#### `broadcast_message`

```rust
async fn broadcast_message(&self, message: AgentMessage) -> Result<()>;
```

#### `subscribe`

```rust
async fn subscribe(&self, agent_id: &AgentId) -> Result<Box<dyn MessageReceiver>>;
```

#### `unsubscribe`

```rust
async fn unsubscribe(&self, agent_id: &AgentId) -> Result<()>;
```

#### `subscription_count`

```rust
async fn subscription_count(&self) -> Result<usize>;
```

#### `health_check`

```rust
async fn health_check(&self) -> Result<bool>;
```

---

### AgentRegistry

**Source**: `registry/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait AgentRegistry: Send + Sync { ... }
```

Trait for agent registry implementations.

Registries manage the lifecycle and discovery of agents in the system.
They provide methods to register new agents, discover existing agents,
and query agent status and capabilities.

**Methods**:

#### `register_agent`

```rust
async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()>;
```

#### `unregister_agent`

```rust
async fn unregister_agent(&self, agent_id: &AgentId) -> Result<()>;
```

#### `get_agent`

```rust
async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>>;
```

#### `list_agents`

```rust
async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>>;
```

#### `find_agents_by_capability`

```rust
async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>>;
```

#### `get_agent_status`

```rust
async fn get_agent_status(&self, agent_id: &AgentId) -> Result<Option<AgentStatus>>;
```

#### `update_agent_status`

```rust
async fn update_agent_status(&self, status: AgentStatus) -> Result<()>;
```

#### `list_agent_statuses`

```rust
async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>>;
```

#### `is_agent_registered`

```rust
async fn is_agent_registered(&self, agent_id: &AgentId) -> Result<bool> {
```

#### `agent_count`

```rust
async fn agent_count(&self) -> Result<usize> {
```

#### `health_check`

```rust
async fn health_check(&self) -> Result<bool> {
```

---

### IntoAgent

**Source**: `src/lib.rs`

```rust
pub trait IntoAgent { ... }
```

Trait for objects that can be converted into an Agent.

This allows for flexible agent creation and registration patterns.

**Methods**:

#### `into_agent`

```rust
fn into_agent(self) -> Arc<dyn Agent>;
```

---

### MessageReceiver

**Source**: `communication/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait MessageReceiver: Send + Sync { ... }
```

Trait for receiving messages from the communication system.

**Methods**:

#### `receive`

```rust
async fn receive(&mut self) -> Option<AgentMessage>;
```

#### `try_receive`

```rust
fn try_receive(&mut self) -> Option<AgentMessage>;
```

#### `close`

```rust
async fn close(&mut self);
```

#### `is_closed`

```rust
fn is_closed(&self) -> bool;
```

---


---

*This documentation was automatically generated from the source code.*