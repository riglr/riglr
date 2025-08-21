// riglr-agents/src/adapter.rs

use rig::completion::request::ToolDefinition;
use rig::tool::{Tool, ToolError as RigToolError};
use riglr_core::Tool as RiglrTool;
use std::future::Future;
use std::sync::Arc;

/// An adapter that wraps a `riglr_core::Tool` to make it compatible
/// with the `rig::tool::Tool` trait required by the `rig` agent brain.
#[derive(Clone)]
pub struct RigToolAdapter {
    inner: Arc<dyn RiglrTool>,
}

// The rig::tool::Tool trait requires Debug.
impl std::fmt::Debug for RigToolAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RigToolAdapter")
            .field("name", &self.inner.name())
            .finish()
    }
}

impl RigToolAdapter {
    /// Creates a new RigToolAdapter wrapping the given riglr_core::Tool
    pub fn new(tool: Arc<dyn RiglrTool>) -> Self {
        Self { inner: tool }
    }
}

impl Tool for RigToolAdapter {
    const NAME: &'static str = "riglr_dynamic_tool";

    type Error = RigToolError;
    type Args = serde_json::Value;
    type Output = serde_json::Value;

    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    fn definition(&self, _prompt: String) -> impl Future<Output = ToolDefinition> + Send + Sync {
        let tool_name = self.inner.name().to_string();
        let tool_description = self.inner.description().to_string();

        async move {
            ToolDefinition {
                name: tool_name,
                description: tool_description,
                parameters: schemars::schema_for!(serde_json::Value).into(),
            }
        }
    }

    fn call(
        &self,
        _args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync {
        let tool_name = self.inner.name().to_string();

        async move {
            let err_msg = format!(
                "Tool '{}' must be executed by the Riglr ToolWorker, not called directly by the rig agent.",
                tool_name
            );
            Err(RigToolError::ToolCallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                err_msg,
            ))))
        }
    }
}
