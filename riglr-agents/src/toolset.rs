// riglr-agents/src/toolset.rs

use std::sync::Arc;

/// A collection of tools that can be used by an agent.
/// This is the single source of truth for an agent's capabilities.
///
/// Since rig::tool::Tool is not object-safe, we can't store different tool types
/// in a collection. Instead, the Toolset acts as a builder pattern that applies
/// tools directly to the rig::AgentBuilder when needed.
#[derive(Clone, Debug)]
pub struct Toolset {
    context: Arc<riglr_core::provider::ApplicationContext>,
}

impl Toolset {
    /// Creates a new Toolset with the given ApplicationContext
    pub fn new(context: Arc<riglr_core::provider::ApplicationContext>) -> Self {
        Self { context }
    }

    /// Registers tools with a `rig::AgentBuilder`.
    ///
    /// Since we can't store heterogeneous tool types, tools must be added
    /// directly to the builder. This method exists to provide a place to
    /// configure which tools are available.
    pub(crate) fn register_with_brain<M: rig::completion::CompletionModel>(
        self,
        mut builder: rig::agent::AgentBuilder<M>,
    ) -> rig::agent::AgentBuilder<M> {
        // Add Solana tools if the feature is enabled
        #[cfg(feature = "solana-tools")]
        {
            builder = builder
                .tool(riglr_solana_tools::balance::get_sol_balance_tool(
                    self.context.clone(),
                ))
                .tool(riglr_solana_tools::transaction::transfer_sol_tool(
                    self.context.clone(),
                ));
        }

        // Add EVM tools if the feature is enabled
        #[cfg(feature = "riglr-evm-tools")]
        {
            // TODO: Add EVM tools when they're updated
        }

        builder
    }
}
