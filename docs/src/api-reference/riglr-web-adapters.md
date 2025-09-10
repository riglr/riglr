# riglr-web-adapters

{{#include ../../../riglr-web-adapters/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)

### Structs

> Core data structures and types.

#### `ActixRiglrAdapter`

Actix Web adapter that uses SignerFactory for authentication

---

#### `AuthenticationData`

Authentication data extracted from HTTP requests

---

#### `AxumRiglrAdapter`

Axum adapter that uses SignerFactory for authentication

---

#### `CompletionResponse`

Generic completion response structure

---

#### `CompositeSignerFactory`

Composite factory that can hold multiple authentication providers

---

#### `PromptRequest`

Generic prompt request structure

---

### Enums

> Enumeration types for representing variants.

#### `AgentEvent`

Server-Sent Event structure for streaming

**Variants:**

- `Start`
  - Agent started processing
- `Content`
  - Streaming content chunk
- `Complete`
  - Agent finished processing
- `Error`
  - Error occurred

---

### Traits

> Trait definitions for implementing common behaviors.

#### `Agent`

Agent trait for framework-agnostic agent interactions
This trait allows any type to be used as an agent as long as it can
provide prompt responses and streaming capabilities.

**Methods:**

- `prompt()`
  - Execute a single prompt and return a response
- `prompt_stream()`
  - Execute a prompt and return a streaming response
- `model_name()`
  - Get the model name used by this agent (optional)

---

#### `SignerFactory`

Abstract factory for creating signers from authentication data

**Methods:**

- `create_signer()`
  - Create a signer from authentication data
- `supported_auth_types()`
  - Get list of supported authentication types

---

### Functions

> Standalone functions and utilities.

#### `handle_agent_completion`

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

#### `handle_agent_stream`

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

#### `health_handler`

Health check handler

---

#### `info_handler`

Information handler for Axum

---

### Type Aliases

#### `AgentStream`

Type alias for agent streaming responses

**Type:** `<<_>>`

---
