# riglr-web-adapters API Reference

Comprehensive API documentation for the `riglr-web-adapters` crate.

## Table of Contents

### Structs

- [`ActixRiglrAdapter`](#actixriglradapter)
- [`AuthenticationData`](#authenticationdata)
- [`AxumRiglrAdapter`](#axumriglradapter)
- [`CompletionResponse`](#completionresponse)
- [`CompositeSignerFactory`](#compositesignerfactory)
- [`PromptRequest`](#promptrequest)

### Functions

- [`add_factory`](#add_factory)
- [`completion_handler`](#completion_handler)
- [`completion_handler`](#completion_handler)
- [`get_registered_auth_types`](#get_registered_auth_types)
- [`handle_agent_completion`](#handle_agent_completion)
- [`handle_agent_stream`](#handle_agent_stream)
- [`health_handler`](#health_handler)
- [`health_handler`](#health_handler)
- [`info_handler`](#info_handler)
- [`info_handler`](#info_handler)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`register_factory`](#register_factory)
- [`sse_handler`](#sse_handler)
- [`sse_handler`](#sse_handler)

### Traits

- [`Agent`](#agent)
- [`SignerFactory`](#signerfactory)

### Type Aliases

- [`AgentStream`](#agentstream)

### Enums

- [`AgentEvent`](#agentevent)

## Structs

### ActixRiglrAdapter

**Source**: `src/actix.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct ActixRiglrAdapter { signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig, }
```

Actix Web adapter that uses SignerFactory for authentication

---

### AuthenticationData

**Source**: `src/factory.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct AuthenticationData { /// Type of authentication (e.g., "privy", "web3auth", "magic")
```

Authentication data extracted from HTTP requests

---

### AxumRiglrAdapter

**Source**: `src/axum.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct AxumRiglrAdapter { signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig, }
```

Axum adapter that uses SignerFactory for authentication

---

### CompletionResponse

**Source**: `src/core.rs`

**Attributes**:
```rust
#[derive(Serialize, Deserialize, Debug)]
```

```rust
pub struct CompletionResponse { /// The agent's response pub response: String, /// Model used for the response pub model: String, /// Conversation ID for tracking pub conversation_id: String, /// Request ID for tracing pub request_id: String, /// Response timestamp pub timestamp: chrono::DateTime<chrono::Utc>, }
```

Generic completion response structure

---

### CompositeSignerFactory

**Source**: `src/factory.rs`

```rust
pub struct CompositeSignerFactory { factories: HashMap<String, std::sync::Arc<dyn SignerFactory>>, }
```

Composite factory that can hold multiple authentication providers

---

### PromptRequest

**Source**: `src/core.rs`

**Attributes**:
```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
```

```rust
pub struct PromptRequest { /// The message/prompt to send to the agent pub text: String, /// Optional conversation ID for tracking pub conversation_id: Option<String>, /// Optional request ID for tracing pub request_id: Option<String>, }
```

Generic prompt request structure

---

## Functions

### add_factory

**Source**: `src/factory.rs`

```rust
pub fn add_factory(&mut self, auth_type: String, factory: std::sync::Arc<dyn SignerFactory>)
```

Convenience: add a factory wrapped in Arc

---

### completion_handler

**Source**: `src/actix.rs`

```rust
pub async fn completion_handler<A>( &self, req: &HttpRequest, agent: &A, prompt: PromptRequest, ) -> ActixResult<HttpResponse> where A: Agent + Clone + Send + Sync + 'static, A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
```

Completion handler using SignerFactory pattern

---

### completion_handler

**Source**: `src/axum.rs`

```rust
pub async fn completion_handler<A>( &self, headers: HeaderMap, agent: A, prompt: PromptRequest, ) -> Result<Json<CompletionResponse>, StatusCode> where A: Agent + Clone + Send + Sync + 'static, A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
```

Completion handler using SignerFactory pattern

---

### get_registered_auth_types

**Source**: `src/factory.rs`

```rust
pub fn get_registered_auth_types(&self) -> Vec<String>
```

Get all registered auth types

---

### handle_agent_completion

**Source**: `src/core.rs`

```rust
pub async fn handle_agent_completion<A>( agent: A, signer: std::sync::Arc<dyn TransactionSigner>, prompt: PromptRequest, ) -> Result<CompletionResponse, Box<dyn StdError + Send + Sync>> where A: Agent + Send + Sync + 'static,
```

Framework-agnostic handler for one-shot agent completion

This function executes an agent prompt within a SignerContext and returns
a completion response that can be serialized by any web framework.

# Arguments
* `agent` - The rig agent to execute
* `signer` - The signer to use for blockchain operations
* `prompt` - The prompt request

# Returns
A completion response with the agent's answer

---

### handle_agent_stream

**Source**: `src/core.rs`

```rust
pub async fn handle_agent_stream<A>( agent: A, signer: std::sync::Arc<dyn TransactionSigner>, prompt: PromptRequest, ) -> Result<AgentStream, Box<dyn StdError + Send + Sync>> where A: Agent + Send + Sync + 'static,
```

Framework-agnostic handler for agent streaming

This function executes an agent prompt within a SignerContext and returns
a stream of events that can be adapted to any web framework's SSE implementation.

# Arguments
* `agent` - The rig agent to execute
* `signer` - The signer to use for blockchain operations
* `prompt` - The prompt request

# Returns
A stream of formatted SSE events as JSON strings

---

### health_handler

**Source**: `src/actix.rs`

```rust
pub async fn health_handler() -> ActixResult<HttpResponse>
```

Health check handler

---

### health_handler

**Source**: `src/axum.rs`

```rust
pub async fn health_handler() -> Json<serde_json::Value>
```

Health check handler for Axum

---

### info_handler

**Source**: `src/actix.rs`

```rust
pub async fn info_handler() -> ActixResult<HttpResponse>
```

Information handler

---

### info_handler

**Source**: `src/axum.rs`

```rust
pub async fn info_handler() -> Json<serde_json::Value>
```

Information handler for Axum

---

### new

**Source**: `src/actix.rs`

```rust
pub fn new(signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig) -> Self
```

Create a new Actix adapter with the given signer factory and RPC config

---

### new

**Source**: `src/axum.rs`

```rust
pub fn new(signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig) -> Self
```

Create a new Axum adapter with the given signer factory and RPC config

---

### new

**Source**: `src/factory.rs`

```rust
pub fn new() -> Self
```

Create a new composite factory

---

### register_factory

**Source**: `src/factory.rs`

```rust
pub fn register_factory(&mut self, auth_type: String, factory: Box<dyn SignerFactory>)
```

Register a signer factory for a specific auth type

# Arguments
* `auth_type` - Authentication type identifier
* `factory` - Factory implementation for this auth type

---

### sse_handler

**Source**: `src/actix.rs`

```rust
pub async fn sse_handler<A>( &self, req: &HttpRequest, agent: &A, prompt: PromptRequest, ) -> ActixResult<HttpResponse> where A: Agent + Clone + Send + Sync + 'static, A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
```

SSE handler using SignerFactory pattern

---

### sse_handler

**Source**: `src/axum.rs`

```rust
pub async fn sse_handler<A>( &self, headers: HeaderMap, agent: A, prompt: PromptRequest, ) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, axum::Error>>>, StatusCode> where A: Agent + Clone + Send + Sync + 'static, A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
```

SSE handler using SignerFactory pattern

---

## Traits

### Agent

**Source**: `src/core.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait Agent: Clone + Send + Sync + 'static { ... }
```

Agent trait for framework-agnostic agent interactions
This trait allows any type to be used as an agent as long as it can
provide prompt responses and streaming capabilities.

**Methods**:

#### `prompt`

```rust
async fn prompt(&self, prompt: &str) -> Result<String, Self::Error>;
```

#### `prompt_stream`

```rust
async fn prompt_stream(&self, prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>;
```

#### `model_name`

```rust
fn model_name(&self) -> Option<String> {
```

---

### SignerFactory

**Source**: `src/factory.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait SignerFactory: Send + Sync { ... }
```

Abstract factory for creating signers from authentication data

**Methods**:

#### `create_signer`

```rust
async fn create_signer( &self, auth_data: AuthenticationData, config: &RpcConfig, ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>>;
```

#### `supported_auth_types`

```rust
fn supported_auth_types(&self) -> Vec<String>;
```

---

## Type Aliases

### AgentStream

**Source**: `src/core.rs`

```rust
type AgentStream = Pin<Box<dyn Stream<Item = Result<String, Box<dyn StdError + Send + Sync>>> + Send>>
```

Type alias for agent streaming responses

---

## Enums

### AgentEvent

**Source**: `src/core.rs`

**Attributes**:
```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
```

```rust
pub enum AgentEvent { /// Agent started processing #[serde(rename = "start")] Start { conversation_id: String, request_id: String, timestamp: chrono::DateTime<chrono::Utc>, }, /// Streaming content chunk #[serde(rename = "content")] Content { content: String, conversation_id: String, request_id: String, }, /// Agent finished processing #[serde(rename = "complete")] Complete { conversation_id: String, request_id: String, timestamp: chrono::DateTime<chrono::Utc>, }, /// Error occurred #[serde(rename = "error")] Error { error: String, conversation_id: String, request_id: String, timestamp: chrono::DateTime<chrono::Utc>, }, }
```

Server-Sent Event structure for streaming

**Variants**:

- `Start`
- `conversation_id`
- `request_id`
- `timestamp`
- `Content`
- `content`
- `conversation_id`
- `request_id`
- `Complete`
- `conversation_id`
- `request_id`
- `timestamp`
- `Error`
- `error`
- `conversation_id`
- `request_id`
- `timestamp`

---


---

*This documentation was automatically generated from the source code.*