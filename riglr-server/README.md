# riglr-server

HTTP server for serving rig agents over REST API with Server-Sent Events streaming support.

## Features

- **Generic Agent Support**: Works with any type that implements rig agent traits
- **SignerContext Integration**: Per-request signer isolation for secure multi-tenant operation (coming soon)
- **Server-Sent Events**: Real-time streaming responses for chat interactions
- **CORS Support**: Cross-origin resource sharing for web clients
- **Health Checks**: Built-in health endpoint for monitoring

## Quick Start

```bash
cargo run --example agent_server
```

This will start a server on `http://127.0.0.1:8080` with the following endpoints:

- `GET /` - Server information
- `POST /chat` - Single-turn chat with agent
- `GET /stream` - Server-Sent Events streaming
- `GET /health` - Health check

## API Endpoints

### POST /chat

Send a message to the agent and receive a complete response.

**Request:**
```json
{
  "message": "Hello, agent!",
  "signer_config": {
    "solana_rpc_url": "https://api.devnet.solana.com",
    "evm_rpc_url": "https://eth.public-rpc.com",
    "user_id": "user123",
    "locale": "en"
  },
  "conversation_id": "optional-conv-id",
  "request_id": "optional-req-id"
}
```

**Response:**
```json
{
  "response": "Hello! How can I help you?",
  "tool_calls": [],
  "conversation_id": "conv-123",
  "request_id": "req-456",
  "timestamp": "2025-08-10T12:00:00Z"
}
```

### GET /stream

Server-Sent Events endpoint for real-time streaming responses.

**Query Parameters:**
- `message` - The message to send to the agent
- `signer_config` - Optional signer configuration (JSON-encoded)
- `conversation_id` - Optional conversation ID

**Response:**
Stream of JSON events with type `start`, `content`, `tool_call`, `tool_result`, `complete`, or `error`.

### GET /health

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-08-10T12:00:00Z",
  "version": "0.1.0"
}
```

## Current Status

This implementation provides the foundational HTTP server infrastructure. The server currently returns placeholder responses while waiting for the complete SignerContext implementation from Phase I of the riglr migration plan.

Once SignerContext is fully implemented, the server will:
- Accept rig agents with blockchain tools
- Execute tools within isolated signer contexts per request
- Stream real-time agent responses with tool execution
- Support secure multi-tenant operation

## Development

Run tests:
```bash
cargo test -p riglr-server
```

Check the example:
```bash
cargo check -p riglr-server --examples
```

## Integration

Once ready, the server can be used with any rig agent:

```rust
// This will work once SignerContext is implemented
use rig::agent::AgentBuilder;
use riglr_server::AgentServer;
use riglr_solana_tools::GetSolBalance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = AgentBuilder::new("gpt-4")
        .tool(GetSolBalance)
        .build();
    
    let server = AgentServer::new(agent, "127.0.0.1:8080".to_string());
    server.run().await?;
    
    Ok(())
}
```