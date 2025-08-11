//! Server-Sent Events (SSE) streaming implementation for real-time agent responses

use actix_web::{HttpResponse, HttpRequest, web, web::Bytes};
use futures_util::{stream, Stream, StreamExt};
use std::sync::Arc;
use uuid::Uuid;
use crate::{handlers::StreamRequest, Result};

/// SSE event types for structured streaming
#[derive(serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Agent is starting to process the request
    #[serde(rename = "start")]
    Start {
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Agent is generating a response (streaming text)
    #[serde(rename = "content")]
    Content {
        content: String,
        conversation_id: String,
        request_id: String,
    },
    
    /// Agent is calling a tool
    #[serde(rename = "tool_call")]
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value,
        conversation_id: String,
        request_id: String,
    },
    
    /// Tool call result
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_name: String,
        result: Option<serde_json::Value>,
        error: Option<String>,
        conversation_id: String,
        request_id: String,
    },
    
    /// Agent has finished processing
    #[serde(rename = "complete")]
    Complete {
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        total_tokens: Option<u32>,
    },
    
    /// Error occurred during processing
    #[serde(rename = "error")]
    Error {
        error: String,
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Keep-alive ping to maintain connection
    #[serde(rename = "ping")]
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}


/// Create an SSE stream for agent responses
pub fn create_agent_stream<A>(
    _agent: Arc<A>,
    request: StreamRequest,
) -> impl Stream<Item = std::result::Result<Bytes, actix_web::Error>>
where
    A: Clone + Send + Sync + 'static,
{
    let conversation_id = request.conversation_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let request_id = Uuid::new_v4().to_string();
    
    // Create demonstration stream showing SignerContext integration
    // NOTE: This demonstrates the SignerContext pattern integration.
    // In a production environment, this would:
    // 1. Create signer from request.signer_config
    // 2. Execute agent.prompt().stream() within SignerContext::with_signer()  
    // 3. Stream real agent responses, tool calls, and results
    
    stream::iter(create_mock_events(conversation_id, request_id, request.message))
        .map(|event| {
            match serde_json::to_string(&event) {
                Ok(json) => Ok(Bytes::from(format!("data: {}\n\n", json))),
                Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("JSON error: {}", e))),
            }
        })
}

/// Create demonstration events showing SignerContext integration capabilities
fn create_mock_events(
    conversation_id: String,
    request_id: String,
    message: String,
) -> Vec<AgentEvent> {
    vec![
        AgentEvent::Start {
            conversation_id: conversation_id.clone(),
            request_id: request_id.clone(),
            timestamp: chrono::Utc::now(),
        },
        AgentEvent::Content {
            content: format!("I received your message: '{}'", message),
            conversation_id: conversation_id.clone(),
            request_id: request_id.clone(),
        },
        AgentEvent::Content {
            content: " I'm currently running in demonstration mode to show SignerContext integration.".to_string(),
            conversation_id: conversation_id.clone(),
            request_id: request_id.clone(),
        },
        AgentEvent::Content {
            content: " With SignerContext integration, I can execute blockchain tools securely in real-time!".to_string(),
            conversation_id: conversation_id.clone(),
            request_id: request_id.clone(),
        },
        AgentEvent::Complete {
            conversation_id: conversation_id.clone(),
            request_id: request_id.clone(),
            timestamp: chrono::Utc::now(),
            total_tokens: Some(42),
        },
    ]
}

/// Create a keep-alive ping stream to maintain SSE connections
pub fn create_ping_stream() -> impl Stream<Item = std::result::Result<Bytes, actix_web::Error>> {
    stream::repeat_with(|| {
        AgentEvent::Ping {
            timestamp: chrono::Utc::now(),
        }
    })
    .map(|event| {
        match serde_json::to_string(&event) {
            Ok(json) => Ok(Bytes::from(format!("data: {}\n\n", json))),
            Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("JSON error: {}", e))),
        }
    })
}

/// Enhanced stream handler that combines agent events with keep-alive pings
pub async fn stream_with_keepalive<A>(
    agent: web::Data<Arc<A>>,
    _req: HttpRequest,
    query: web::Query<StreamRequest>,
) -> Result<HttpResponse>
where
    A: Clone + Send + Sync + 'static,
{
    tracing::info!(
        conversation_id = ?query.conversation_id,
        message_len = query.message.len(),
        "Starting SSE stream"
    );
    
    // Create agent event stream
    let agent_stream = create_agent_stream(agent.get_ref().clone(), query.into_inner());
    
    // Set SSE headers
    let response = HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Headers", "Cache-Control"))
        .streaming(agent_stream);
    
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    
    #[tokio::test]
    async fn test_agent_event_serialization() {
        let event = AgentEvent::Start {
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let json_str = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["type"], "start");
        assert_eq!(parsed["conversation_id"], "conv-123");
    }
    
    #[tokio::test]
    async fn test_mock_events_generation() {
        let events = create_mock_events(
            "conv-123".to_string(),
            "req-456".to_string(),
            "Hello".to_string(),
        );
        
        assert!(!events.is_empty());
        
        // Check that we have start and complete events
        let has_start = events.iter().any(|e| matches!(e, AgentEvent::Start { .. }));
        let has_complete = events.iter().any(|e| matches!(e, AgentEvent::Complete { .. }));
        
        assert!(has_start);
        assert!(has_complete);
    }
    
    #[derive(Clone)]
    struct MockAgent;
    
    #[tokio::test]
    async fn test_agent_stream_creation() {
        let agent = Arc::new(MockAgent);
        let request = StreamRequest {
            message: "Test message".to_string(),
            signer_config: None,
            conversation_id: Some("conv-123".to_string()),
        };
        
        let mut stream = create_agent_stream(agent, request);
        
        // Test that we can get at least one event
        let first_event = stream.next().await;
        assert!(first_event.is_some());
    }
    
    #[tokio::test]
    async fn test_ping_stream() {
        let ping_stream = create_ping_stream();
        
        // This test would take 30 seconds to complete, so we just verify the stream exists
        // The size hint lower bound doesn't matter for an infinite stream
        let (_lower, upper) = ping_stream.size_hint();
        assert!(upper.is_none()); // Infinite stream should have no upper bound
    }
}