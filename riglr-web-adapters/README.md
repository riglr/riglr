# riglr-web-adapters

[![Crates.io](https://img.shields.io/crates/v/riglr-web-adapters.svg)](https://crates.io/crates/riglr-web-adapters)
[![Documentation](https://docs.rs/riglr-web-adapters/badge.svg)](https://docs.rs/riglr-web-adapters)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Framework-agnostic web adapters for exposing riglr agents via HTTP APIs, with built-in support for Actix-web and Axum frameworks.

## Overview

riglr-web-adapters provides a library-first approach to building web APIs for riglr agents. It allows you to expose your blockchain automation agents through RESTful APIs and WebSocket connections, with support for multiple web frameworks and streaming responses.

## Key Features

- **Framework Agnostic**: Core logic independent of web framework
- **Multiple Frameworks**: Built-in support for Actix-web and Axum
- **Streaming Support**: Server-sent events (SSE) for real-time updates
- **WebSocket Ready**: Full WebSocket support for bidirectional communication
- **Type-Safe Handlers**: Strongly typed request/response handling
- **Authentication**: Built-in Web3 authentication support
- **Library-First**: Use as a library, not a framework

## Quick Start

Add riglr-web-adapters to your `Cargo.toml`:

```toml
[dependencies]
riglr-web-adapters = "0.3.0"
riglr-core = "0.3.0"

# Choose your framework
riglr-web-adapters = { version = "0.3.0", features = ["actix"] }
# OR
riglr-web-adapters = { version = "0.3.0", features = ["axum"] }
```

## Framework Support

### Actix-web
- High-performance async web framework
- Built-in WebSocket support
- SSE through actix-web-lab

### Axum
- Tower-based framework
- Excellent for microservices
- Native streaming support

## Documentation

For comprehensive documentation, integration examples, and API guides, see the documentation in the repository.

The documentation covers:
- Framework integration patterns
- Building RESTful APIs for agents
- Streaming and WebSocket implementation
- Authentication and security
- Request/response handling
- Error management
- Deployment considerations

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)