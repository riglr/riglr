# Request Lifecycle: From Brain to Blockchain

This document outlines the complete architectural flow from a natural language command to a verified blockchain transaction, detailing how riglr transforms user intent into secure on-chain operations.

## Core Concepts

The riglr ecosystem establishes a clear separation between the AI-driven decision-making "brain" and the blockchain-interaction "body":

- **The `rig` Ecosystem (The Brain)**: Responsible for understanding natural language, making decisions, and selecting appropriate tools
- **The `riglr` Ecosystem (The Body)**: Acts as the execution engine, securely interacting with blockchains to perform operations
- **Tools (The Hands)**: Specific functions like `transfer_sol` or `get_sol_balance` that can be called to perform actions

## Architectural Blueprint

```
========================================================================================================================
|   riglr-agents Crate (The Factory Supervisor & Worker)                                                               |
|                                                                                                                      |
|   [ User Prompt ]                                                                                                    |
|        |                                                                                                             |
|        | 1. Task (e.g., "Send 0.01 SOL to...")                                                                       |
|        v                                                                                                             |
|   +---------------------+                                                                                            |
|   | AgentDispatcher   |                                                                                            |
|   +---------------------+                                                                                            |
|        |                                                                                                             |
|        | 2. Finds capable agent                                                                                      |
|        v                                                                                                             |
|   +---------------------+      +----------------------------------------------------------------------------------+  |
|   | AgentRegistry     |----->| riglr_agents::Agent (LiveToolAgent)                                                |  |
|   +---------------------+      |                                                                                  |  |
|                              |   (The Orchestrator / The "Body")                                                |  |
|                              |                                                                                  |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |   |  rig-core Crate (The "Brain")                                              |   |  |
|                              |   |                                                                            |   |  |
|                              |   |   +---------------------+      4. Prompt      +------------------------+  |   |  |
|                              |   |   | rig::agent::Agent   |--------------------->|   LLM Provider (API)   |  |   |  |
|                              |   |   +---------------------+      (w/ Tool Schemas) +------------------------+  |   |  |
|                              |   |             ^                                              |               |   |  |
|                              |   |             | 5. JSON Tool Call ("transfer_sol", {..})     |               |   |  |
|                              |   |             +----------------------------------------------+               |   |  |
|                              |   |                                                                            |   |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |                                      |                                          |  |
|                              | 6. Receives Tool Call               | 7. Creates Job                           |  |
|                              |                                      v                                          |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |   |  riglr-core Crate (The Execution Engine / The "Foundation")                |   |  |
|                              |   |                                                                            |   |  |
|                              |   |   +---------------------+      8. process_job()      +-----------------+ |   |  |
|                              |   |   |    ToolWorker       |--------------------------->| riglr_core::Tool| |   |  |
|                              |   |   +---------------------+                            +-----------------+ |   |  |
|                              |   |             ^                                              |               |   |  |
|                              |   |             | 13. JobResult (w/ signature)                 | 9. execute()  |   |  |
|                              |   |             +----------------------------------------------+               |   |  |
|                              |   |                                                                            |   |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |                                                                                  |  |
|                              +----------------------------------------------------------------------------------+  |
|        ^                                                                                                             |
|        | 14. TaskResult                                                                                            |
|        +-----------------------------------------------------------------------------------------------------------+
|                                                                                                                      |
========================================================================================================================
                                                               |
                                                               | 10. Uses Contexts
                                                               v
+----------------------------------------------------------------------------------------------------------------------+
| riglr-core Contexts                                                                                                  |
|                                                                                                                      |
| +----------------------+     +-----------------------+     +-------------------------------------------------------+  |
| | ApplicationContext   |     |    SignerContext      |     | riglr-solana-tools Crate (The "Hands")                |  |
| |                      |     |                       |     |                                                       |  |
| | - RPC Client         |---->| - Secure Signer       |---->|   #[tool]                                             |  |
| | - Config             | 11. |                       | 12. |   async fn transfer_sol(...) { ... }                  |  |
| +----------------------+     +-----------------------+     |       |                                               |  |
|                                                            |       | 13. Sends REAL Transaction                    |  |
|                                                            |       v                                               |  |
|                                                            |   [ Blockchain ]                                      |  |
|                                                            +-------------------------------------------------------+  |
+----------------------------------------------------------------------------------------------------------------------+
```

## Request Lifecycle: Step-by-Step Analysis

### Phase 1: Task Reception and Dispatch

#### 1. Task Creation
A user or automated system initiates a `Task` containing a natural language description:
```rust
let task = Task::new(
    TaskType::Custom("tool_calling".to_string()),
    serde_json::json!({ "prompt": "Send 0.001 SOL to 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM" })
);
```

#### 2. Agent Dispatch
The `AgentDispatcher` receives the task and queries the `AgentRegistry` to identify an agent with the required capabilities:

```rust
let dispatcher = AgentDispatcher::new(registry);
let result = dispatcher.dispatch_task(task).await?;
```

#### 3. Agent Selection
The `AgentRegistry` identifies and provides the appropriate agent (e.g., `ToolCallingAgent`) based on:
- Agent capabilities (e.g., `solana_operations`)
- Load balancing strategy
- Agent availability

### Phase 2: AI-Driven Decision Making

#### 4. Prompting the Brain
The selected agent forwards the natural language prompt to its internal `rig::Agent` instance along with schemas of all available tools:

```rust
// Agent provides tool schemas to the LLM
let available_tools = self.toolset.get_tool_schemas();
let completion = self.rig_agent
    .prompt(&user_prompt)
    .with_tools(available_tools)
    .await?;
```

#### 5. LLM Processing
The Large Language Model:
- Analyzes the prompt and available tool schemas
- Determines the most suitable tool and parameters
- Returns structured JSON: `{"tool_name": "transfer_sol", "parameters": {"to_address": "...", "amount_sol": 0.001}}`

#### 6. Structured Response Parsing
The `rig::Agent` parses the LLM response into a `ToolCall` object containing the tool name and validated parameters.

### Phase 3: Secure Execution Handoff

#### 7. Job Creation
The `ToolCallingAgent` receives the `ToolCall` and translates it into a generic `Job`:

```rust
let job = Job {
    id: Uuid::new_v4(),
    tool_name: tool_call.name,
    parameters: tool_call.arguments,
    context: serde_json::to_value(&execution_context)?,
};
```

#### 8. Delegation to Worker
The agent delegates the job to the `ToolWorker`:

```rust
let result = self.tool_worker.process_job(job).await?;
```

This is the critical handoff to the `riglr-core` execution engine. The agent's orchestration role is now complete.

### Phase 4: Blockchain Interaction

#### 9. Tool Identification and Execution
The `ToolWorker` locates the corresponding tool and invokes its `execute` method:

```rust
let tool = self.tools.get(&job.tool_name)
    .ok_or_else(|| ToolError::permanent_string("Tool not found"))?;

let result = tool.execute(job.parameters, &self.context).await?;
```

#### 10. Context Access
The tool function accesses necessary resources:

**Application Context** (step 11):
```rust
#[tool]
async fn transfer_sol(
    to_address: String,
    amount_sol: f64,
    context: &ApplicationContext,
) -> Result<TransactionResult, ToolError> {
    // Retrieve RPC client from context
    let client = context.get_extension::<Arc<solana_client::rpc_client::RpcClient>>()?;
```

**Signer Context** (step 12):
```rust
    // Securely access the isolated signer
    let signer = SignerContext::current_as_solana().await?;
    
    // Create and sign transaction
    let transaction = create_transfer_transaction(&signer, &to_address, amount_sol)?;
    let signature = client.send_and_confirm_transaction(&transaction).await?;
```

#### 11-12. Resource Access
- **ApplicationContext**: Provides read-only access to RPC clients, configuration, and API keys
- **SignerContext**: Provides secure, isolated access to cryptographic signers for the current request

#### 13. On-Chain Transaction
The tool:
- Constructs the blockchain transaction
- Signs it using the secure signer
- Broadcasts it to the network
- Receives confirmation with transaction signature

### Phase 5: Result Propagation

#### 14. Result Flow Back
The transaction signature propagates back through the call stack:

```
TransactionResult → JobResult → TaskResult → HTTP Response
```

Each layer adds appropriate context and handles errors according to their responsibility.

## Component Responsibilities

| Ecosystem | Component | Primary Responsibility |
|:----------|:----------|:----------------------|
| **`rig`** | `rig::agent::Agent` | **Decides what to do** by interpreting natural language and selecting tools via LLM |
| **`rig`** | `rig::tool::Tool` | **Describes capabilities** and parameters to the AI brain |
| **`riglr`** | `riglr_agents::Agent` | **Orchestrates workflow** by communicating with brain and delegating to worker |
| **`riglr`** | `riglr_core::ToolWorker` | **Executes requested work** by finding and running appropriate concrete tools |
| **`riglr`** | `riglr_core::Tool` | **Defines execution** in standardized way for the execution engine |
| **`riglr`** | `riglr-solana-tools` | **Provides implementations** (the "hands") that interact with blockchains |

## The #[tool] Macro: Bridging Ecosystems

The `#[tool]` macro is the key component unifying the `rig` and `riglr` ecosystems. It simultaneously implements:

- **`rig::tool::Tool` trait**: For description and AI understanding
- **`riglr_core::Tool` trait**: For execution by the worker

```rust
#[tool]
async fn transfer_sol(
    context: &ApplicationContext,
    to_address: String,
    amount_sol: f64,
    priority_fee: Option<u64>,
    memo: Option<String>,
) -> Result<TransactionResult, ToolError> {
    // Single function serves both as:
    // 1. Tool description for LLM (via generated schema)
    // 2. Executable implementation for worker
}
```

This dual implementation enables seamless integration without code duplication.

## Security Throughout the Lifecycle

### Multi-Tenant Isolation
Each request gets its own `SignerContext`, ensuring complete isolation:

```rust
// User A's request
SignerContext::with_signer(user_a_signer, async {
    dispatcher.dispatch_task(task_a).await
}).await

// User B's request (completely isolated)  
SignerContext::with_signer(user_b_signer, async {
    dispatcher.dispatch_task(task_b).await
}).await
```

### Key Security Properties
1. **No Key Exposure**: Private keys never passed directly to tools
2. **Thread Safety**: Each async task has isolated signer context  
3. **Automatic Cleanup**: Keys cleared when context exits
4. **Input Validation**: All parameters validated before execution
5. **Transaction Safety**: Slippage protection, deadlines, simulation

## Error Handling Throughout Lifecycle

### Error Classification
Errors are classified at each layer for appropriate handling:

```rust
// Tool layer: Chain-specific errors
SolanaToolError::InsufficientBalance

// Core layer: Behavioral classification  
ToolError::Permanent(source_error)

// Agent layer: Task-level errors
TaskResult::Error(retriable: false, error_message)

// Server layer: HTTP status codes
500 Internal Server Error (if retriable)
400 Bad Request (if permanent)
```

### Retry Logic
The `ToolWorker` implements intelligent retry based on error classification:

- **Retriable errors**: Exponential backoff retry
- **Rate limit errors**: Respect retry-after headers
- **Permanent errors**: No retry, immediate failure
- **Invalid input**: No retry, return validation error

## Performance Optimizations

### Concurrent Processing
- Multiple agents can process tasks simultaneously
- Tools use connection pooling for RPC clients
- Stream processing handles high-throughput events

### Caching Strategy
- RPC call results cached with appropriate TTL
- Tool schemas cached for LLM interactions
- Configuration loaded once at startup

### Resource Management
- Arc-based sharing of read-only resources
- Bounded queues prevent memory exhaustion
- Graceful degradation under load

This end-to-end architecture ensures that riglr can handle everything from simple token transfers to complex multi-step DeFi operations while maintaining security, reliability, and performance at scale.