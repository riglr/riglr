// riglr-agents/src/toolset.rs

use crate::adapter::RigToolAdapter; // Import the adapter
use riglr_core::{idempotency::IdempotencyStore, Tool, ToolWorker};
use std::collections::HashMap;
use std::sync::Arc;

/// A collection of tools that can be used by an agent.
/// This is the single source of truth for an agent's capabilities.
#[derive(Clone)]
pub struct Toolset {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for Toolset {
    fn default() -> Self {
        Self::new()
    }
}

impl Toolset {
    /// Creates a new, empty toolset.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Adds a tool to the set.
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }

    /// "Discovers" and adds all available tools from the `riglr-solana-tools` crate.
    /// This method is only available when the `solana-tools` feature is enabled.
    #[cfg(feature = "solana-tools")]
    pub fn with_solana_tools(self) -> Self {
        self.add_tool(riglr_solana_tools::balance::get_sol_balance_tool())
            .add_tool(riglr_solana_tools::transaction::transfer_sol_tool())
    }

    /// Registers all tools in this set with a `rig::AgentBuilder`.
    /// This now wraps each tool in the `RigToolAdapter` to ensure compatibility.
    pub(crate) fn register_with_brain<M: rig::completion::CompletionModel>(
        &self,
        mut builder: rig::agent::AgentBuilder<M>,
    ) -> rig::agent::AgentBuilder<M> {
        for tool in self.tools.values() {
            // THE FIX: Wrap our internal tool in the adapter before giving it to the rig brain.
            let adapted_tool = RigToolAdapter::new(tool.clone());
            builder = builder.tool(adapted_tool);
        }
        builder
    }

    /// Registers all tools in this set with a `ToolWorker`.
    pub(crate) async fn register_with_worker<S: IdempotencyStore + 'static>(
        &self,
        worker: &ToolWorker<S>,
    ) {
        for tool in self.tools.values() {
            worker.register_tool(tool.clone()).await;
        }
    }
}
