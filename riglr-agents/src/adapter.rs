// riglr-agents/src/adapter.rs

use rig::completion::request::ToolDefinition;
use rig::tool::{Tool, ToolError as RigToolError};
use riglr_core::Tool as RiglrTool;
use std::future::Future;
use std::sync::Arc;

/// Bridge adapter between riglr_core::Tool and rig::tool::Tool traits.
///
/// The RigToolAdapter serves as a crucial bridge between riglr's internal tool system
/// and the rig framework's agent brain requirements. This adapter pattern allows
/// riglr tools to be seamlessly integrated into rig agents without modification.
///
/// # Architecture
///
/// The adapter wraps a `riglr_core::Tool` and implements `rig::tool::Tool`, handling:
/// - Parameter conversion from rig's format to riglr's format
/// - Error translation between the two systems
/// - Context propagation through the ApplicationContext
/// - SignerContext preservation through spawn_with_context
///
/// # Example
///
/// ```rust,ignore
/// use riglr_agents::adapter::RigToolAdapter;
/// use riglr_agents::toolset::Toolset;
/// use riglr_core::Tool;
/// use std::sync::Arc;
///
/// // Given a riglr tool
/// let riglr_tool: Arc<dyn Tool> = get_balance_tool();
/// let context = ApplicationContext::from_env();
///
/// // Wrap it in the adapter
/// let rig_compatible_tool = RigToolAdapter::new(riglr_tool, context);
///
/// // Now it can be used in a Toolset for rig agents
/// let toolset = Toolset::builder()
///     .tool(rig_compatible_tool)
///     .build();
///
/// // The toolset can be given to a rig agent
/// let agent = rig::agent::Agent::builder()
///     .tools(toolset)
///     .build();
/// ```
#[derive(Clone)]
pub struct RigToolAdapter {
    inner: Arc<dyn RiglrTool>,
    context: riglr_core::provider::ApplicationContext,
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
    pub fn new(
        tool: Arc<dyn RiglrTool>,
        context: riglr_core::provider::ApplicationContext,
    ) -> Self {
        Self {
            inner: tool,
            context,
        }
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
        let tool_schema = self.inner.schema();

        async move {
            ToolDefinition {
                name: tool_name,
                description: tool_description,
                parameters: tool_schema,
            }
        }
    }

    /// Execute the tool through the rig framework.
    ///
    /// ## Security Critical: `SignerContext` Propagation
    ///
    /// This method is responsible for propagating the `SignerContext` from the calling
    /// task into the new task spawned for tool execution. Because `SignerContext` relies
    /// on `tokio::task_local!`, it is **not** automatically inherited by spawned tasks.
    ///
    /// This implementation correctly captures the context *before* spawning and re-establishes
    /// it within the new task. However, this pattern relies on a critical assumption:
    ///
    /// **This `call` method MUST be invoked from within an async block that already has the
    /// correct `SignerContext` established.**
    ///
    /// Failure to do so will result in either no signer being available to the tool or,
    /// in a multi-tenant environment, the potential for capturing the wrong signer,
    /// which would be a critical security vulnerability. The end-to-end tests in
    /// `tests/e2e/signer_context.rs` validate this behavior.
    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send + Sync {
        // Clone everything we need before creating the async block
        let tool = self.inner.clone();
        let context = self.context.clone();

        async move {
            // First, try to get the current SignerContext before spawning
            // This is critical for maintaining the security model
            let maybe_signer = riglr_core::SignerContext::current().await.ok();

            // Spawn the task execution, propagating SignerContext if available
            let handle = tokio::spawn(async move {
                // If we captured a signer context, propagate it
                if let Some(signer) = maybe_signer {
                    // We have a signer, so wrap the execution with it
                    let result = riglr_core::SignerContext::with_signer(signer, async move {
                        tool.execute(args, &context)
                            .await
                            .map_err(|e| riglr_core::SignerError::TransactionFailed(e.to_string()))
                    })
                    .await;
                    // Convert back from Result<JobResult, SignerError> to Result<JobResult, ToolError>
                    result.map_err(|e| riglr_core::ToolError::permanent_string(e.to_string()))
                } else {
                    // No signer context, execute normally
                    tool.execute(args, &context).await
                }
            });

            // Wait for the spawned task to complete
            let job_result = match handle.await {
                Ok(Ok(job_result)) => job_result,
                Ok(Err(tool_error)) => {
                    return Err(RigToolError::ToolCallError(Box::new(
                        std::io::Error::other(tool_error.to_string()),
                    )))
                }
                Err(join_error) => {
                    return Err(RigToolError::ToolCallError(Box::new(
                        std::io::Error::other(format!("Task panicked: {}", join_error)),
                    )))
                }
            };

            // Convert JobResult to serde_json::Value
            match job_result {
                riglr_core::JobResult::Success { value, .. } => Ok(value),
                riglr_core::JobResult::Failure { error, .. } => Err(RigToolError::ToolCallError(
                    Box::new(std::io::Error::other(error)),
                )),
            }
        }
    }
}
