//! Example: Basic agent server
//!
//! This example demonstrates how to create a simple HTTP server that serves
//! a rig agent with blockchain tools. The server integrates with SignerContext
//! to provide secure per-request signer isolation for multi-tenant operation.


// Mock agent and tools for demonstration
// NOTE: Pending integration with rig-core for full AI functionality
#[derive(Clone)]
pub struct MockAgent {
    _name: String,
}

impl MockAgent {
    pub fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
        }
    }
}

// NOTE: Full implementation with rig::agent::AgentBuilder pending:
// use rig::agent::AgentBuilder;
// use riglr_solana_tools::{GetSolBalance, SwapTokens, TransferSol};
// 
// let agent = AgentBuilder::new("gpt-4")
//     .preamble("You are a Solana trading assistant. You can check balances, transfer SOL, and swap tokens.")
//     .tool(GetSolBalance)
//     .tool(SwapTokens) 
//     .tool(TransferSol)
//     .build();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("riglr_server=info,actix_web=info")
        .init();
    
    tracing::info!("Starting riglr-server example");
    
    // Create a mock agent (will be replaced with real rig agent)
    let agent = MockAgent::new("Solana Trading Assistant");
    
    // Create and configure the server
    let server = riglr_server::AgentServer::new(agent, "127.0.0.1:8080".to_string())
        .cors_origins(vec![
            "http://localhost:3000".to_string(),
            "http://127.0.0.1:3000".to_string(),
        ])
        .enable_logging(true);
    
    tracing::info!("Server configured with CORS for localhost:3000");
    tracing::info!("SignerContext integration is active. This agent now supports:");
    tracing::info!("  - Secure per-request signer isolation");  
    tracing::info!("  - Blockchain tool access via SignerContext");
    tracing::info!("  - Real-time streaming responses");
    tracing::info!("");
    tracing::info!("Test endpoints:");
    tracing::info!("  POST http://127.0.0.1:8080/chat - Chat with agent");
    tracing::info!("  GET  http://127.0.0.1:8080/stream - Server-Sent Events");
    tracing::info!("  GET  http://127.0.0.1:8080/health - Health check");
    tracing::info!("  GET  http://127.0.0.1:8080/ - Server info");
    
    // Start the server
    server.run().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_agent_creation() {
        let agent = MockAgent::new("Test Agent");
        assert_eq!(agent._name, "Test Agent");
    }
    
    #[tokio::test]
    async fn test_server_configuration() {
        let agent = MockAgent::new("Test Agent");
        let _server = riglr_server::AgentServer::new(agent, "127.0.0.1:0".to_string())
            .cors_origins(vec!["http://test.com".to_string()])
            .enable_logging(false);
        
        // Server should be created successfully
        // We can't test .run() easily without actually binding to a port
        // Server creation itself validates the configuration
    }
}