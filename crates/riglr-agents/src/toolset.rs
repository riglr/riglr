// riglr-agents/src/toolset.rs

extern crate alloc;

use alloc::sync::Arc;
use rig::agent::AgentBuilder;
use rig::completion::CompletionModel;
use riglr_core::provider::ApplicationContext;

#[cfg(feature = "solana-tools")]
use riglr_solana_tools::{balance::GetSolTool, transaction::TransferSolTool};

/// A collection of tools that can be used by an agent.
/// This is the single source of truth for an agent's capabilities.
///
/// Since `rig::tool::Tool` is not object-safe, we can't store different tool types
/// in a collection. Instead, the Toolset acts as a builder pattern that applies
/// tools directly to the `rig::AgentBuilder` when needed.
#[derive(Clone, Debug)]
pub struct Toolset {
    /// The application context used to configure tools
    context: Arc<ApplicationContext>,
}

impl Toolset {
    /// Creates a new Toolset with the given `ApplicationContext`
    #[must_use]
    #[inline]
    pub const fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }

    /// Registers tools with a `rig::AgentBuilder`.
    ///
    /// Since we can't store heterogeneous tool types, tools must be added
    /// directly to the builder. This method exists to provide a place to
    /// configure which tools are available.
    pub(crate) fn register_with_brain<M: CompletionModel>(
        self,
        mut builder: AgentBuilder<M>,
    ) -> AgentBuilder<M> {
        // Add Solana tools if the feature is enabled
        #[cfg(feature = "solana-tools")]
        {
            builder = builder
                .tool(GetSolTool {
                    context: Arc::<ApplicationContext>::clone(&self.context),
                })
                .tool(TransferSolTool {
                    context: self.context,
                });
        };

        // Add EVM tools if the feature is enabled
        #[cfg(feature = "riglr-evm-tools")]
        {
            // TODO: Add EVM tools when they're updated
        }

        builder
    }
}
