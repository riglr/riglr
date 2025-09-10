# riglr-showcase

{{#include ../../../riglr-showcase/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)

### Structs

> Core data structures and types.

#### `ConsoleChannel`

Console/log notification channel for debugging

---

#### `CoordinatorAgent`

A coordinator agent that orchestrates multi-step workflows across multiple agents.

This agent manages complex workflows by breaking them down into sequential steps
and broadcasting coordination messages to worker agents. It serves as the central
orchestration point for multi-agent collaboration scenarios.

---

#### `DiscordChannel`

Discord webhook notification channel

---

#### `DistillationProcessor`

LLM-based output distiller

Uses a separate LLM call to summarize complex tool outputs.
This pattern is useful for making technical outputs more accessible to users.

---

#### `HtmlFormatter`

HTML formatter for tool outputs

---

#### `JsonFormatter`

JSON formatter that can restructure and clean up outputs

---

#### `MarkdownFormatter`

Markdown formatter for tool outputs

Converts tool outputs into clean, readable Markdown format.
Useful for documentation, reports, or display in Markdown-aware interfaces.

---

#### `MarketIntelligenceAgent`

Market research agent that uses real data sources

---

#### `MockDistiller`

Mock distiller for testing without API calls

---

#### `MultiFormatProcessor`

Multi-format processor that can output in multiple formats simultaneously

---

#### `NotificationResult`

Result of a notification attempt

---

#### `NotificationRouter`

Notification router that sends outputs to various channels

---

#### `Order`

Represents a trading order (pending or executed)

---

#### `Position`

Represents an active trading position

---

#### `PrivySignerFactory`

Privy-specific signer factory implementation

---

#### `PrivyUserData`

Privy user data structure

---

#### `ProcessedOutput`

Processed output after going through a processor

---

#### `ProcessorPipeline`

A pipeline that chains multiple processors together

This demonstrates the composability pattern - processors can be chained
to create complex processing workflows.

---

#### `RiskManagementAgent`

Risk management agent with real portfolio tracking

---

#### `RoutingInfo`

Routing information for notifications

---

#### `RoutingRule`

Routing rule for determining which channels to use

---

#### `SimpleRiskAgent`

A simple risk assessment agent that evaluates trade risk based on amount thresholds.

This agent provides basic risk analysis by calculating a risk score based on the trade amount
and applying threshold-based approval logic. Trades with risk scores below 0.5 are approved.

---

#### `SmartDistiller`

Smart distiller that chooses different strategies based on output type

---

#### `TelegramChannel`

Telegram bot notification channel

---

#### `ToolOutput`

Represents the output from a tool execution

---

#### `TradeExecutionAgent`

Execution agent that performs real blockchain trades

---

#### `TradingState`

Shared trading state across all agents

---

#### `WebhookChannel`

Generic webhook notification channel

---

#### `WorkerAgent`

A worker agent that performs specialized tasks and responds to coordination messages.

Worker agents handle specific task types such as research and monitoring while
participating in coordinated workflows by responding to messages from coordinator agents.
They can execute tasks independently or as part of larger orchestrated processes.

---

### Enums

> Enumeration types for representing variants.

#### `NotificationPriority`

Notification priority levels

**Variants:**

- `Low`
  - Low priority notification
- `Normal`
  - Normal priority notification
- `High`
  - High priority notification
- `Critical`
  - Critical priority notification

---

#### `OrderSide`

Specifies the side of a trading order

**Variants:**

- `Buy`
  - Buy order (acquire tokens)
- `Sell`
  - Sell order (dispose of tokens)

---

#### `OrderStatus`

Represents the current status of a trading order

**Variants:**

- `Pending`
  - Order is waiting to be executed
- `Executing`
  - Order is currently being processed
- `Completed`
  - Order has been successfully executed
- `Failed`
  - Order execution failed

---

#### `OutputFormat`

Output format types

**Variants:**

- `Json`
  - JSON format output
- `Markdown`
  - Markdown format output
- `PlainText`
  - Plain text format output
- `Html`
  - HTML format output
- `Custom`
  - Custom format with specified type name

---

#### `RoutingCondition`

Conditions for routing decisions

**Variants:**

- `Always`
  - Always matches any output
- `OnSuccess`
  - Matches only successful outputs
- `OnError`
  - Matches only failed outputs
- `ToolName`
  - Matches outputs from a specific tool by exact name
- `ToolNameContains`
  - Matches outputs from tools whose name contains the given substring
- `ExecutionTimeOver`
  - Matches outputs that took longer than the specified number of milliseconds
- `HasMetadata`
  - Matches outputs that have metadata with the specified key
- `And`
  - Matches when all nested conditions match
- `Or`
  - Matches when any nested condition matches
- `Not`
  - Matches when the nested condition does not match

---

### Traits

> Trait definitions for implementing common behaviors.

#### `NotificationChannel`

Trait for notification channels

**Methods:**

- `send_notification()`
  - Sends a notification for the given tool output and returns a message ID
- `name()`
  - Returns the human-readable name of this channel
- `supports_formatting()`
  - Returns whether this channel supports rich formatting (default: true)

---

#### `OutputProcessor`

Core trait for output processors

This trait defines the interface for all output processors.
Implementations can transform, summarize, format, or route outputs.

**Methods:**

- `process()`
  - Process a tool output and return the processed result
- `name()`
  - Get the processor name for debugging and logging
- `can_process()`
  - Check if this processor can handle the given output type
- `config()`
  - Get processor configuration as JSON for debugging

---

### Functions

> Standalone functions and utilities.

#### `demonstrate_trading_coordination`

Demonstration function that shows the complete trading coordination workflow

---

#### `error_output`

Create a ToolOutput from a failed operation

---

#### `run_chat`

Run interactive chat mode.

---

#### `run_demo`

Runs a multi-agent coordination demonstration based on the specified scenario.

This function demonstrates the riglr-agents framework through various predefined scenarios:
- `"trading"`: Real-world trading coordination with blockchain operations
- `"risk"`: Risk management system with coordinated assessment across multiple agents  
- `"basic"`: Fundamental multi-agent communication and workflow patterns

# Arguments
* `context` - Shared application context containing configuration and resources for all agents
* `scenario` - The demonstration scenario to execute

# Returns
Returns `Ok(())` on successful demonstration completion, or an error if the scenario
is unknown or the demonstration fails.

# Examples
```rust,ignore
use std::sync::Arc;
use riglr_core::provider::ApplicationContext;
use riglr_config::Config;
use riglr_showcase::commands::agents::run_demo;

# async fn example() -> anyhow::Result<()> {
let config = Config::default();
let context = Arc::new(ApplicationContext::from_config(&config));
run_demo(context, "basic".to_string()).await?;
# Ok(())
# }
```

---

#### `run_demo_with_options`

Run the Solana tools demo with options for testing.

---

#### `success_output`

Create a ToolOutput from a successful operation

---

#### `user_friendly_error`

Extract error message from a ToolOutput in a user-friendly way

---

#### `with_metadata`

Add metadata to a ToolOutput

---

#### `with_timing`

Add timing information to a ToolOutput

---
