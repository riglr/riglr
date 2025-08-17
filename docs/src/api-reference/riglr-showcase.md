# riglr-showcase API Reference

Comprehensive API documentation for the `riglr-showcase` crate.

## Table of Contents

### Structs

- [`ConsoleChannel`](#consolechannel)
- [`DiscordChannel`](#discordchannel)
- [`DistillationProcessor`](#distillationprocessor)
- [`HtmlFormatter`](#htmlformatter)
- [`JsonFormatter`](#jsonformatter)
- [`MarkdownFormatter`](#markdownformatter)
- [`MarketIntelligenceAgent`](#marketintelligenceagent)
- [`MockDistiller`](#mockdistiller)
- [`MultiFormatProcessor`](#multiformatprocessor)
- [`NotificationResult`](#notificationresult)
- [`NotificationRouter`](#notificationrouter)
- [`Order`](#order)
- [`Position`](#position)
- [`PrivySignerFactory`](#privysignerfactory)
- [`PrivyUserData`](#privyuserdata)
- [`ProcessedOutput`](#processedoutput)
- [`ProcessorPipeline`](#processorpipeline)
- [`RiskManagementAgent`](#riskmanagementagent)
- [`RoutingInfo`](#routinginfo)
- [`RoutingRule`](#routingrule)
- [`SmartDistiller`](#smartdistiller)
- [`TelegramChannel`](#telegramchannel)
- [`ToolOutput`](#tooloutput)
- [`TradeExecutionAgent`](#tradeexecutionagent)
- [`TradingState`](#tradingstate)
- [`WebhookChannel`](#webhookchannel)

### Enums

- [`NotificationPriority`](#notificationpriority)
- [`OrderSide`](#orderside)
- [`OrderStatus`](#orderstatus)
- [`OutputFormat`](#outputformat)
- [`RoutingCondition`](#routingcondition)

### Functions (trading_coordination)

- [`demonstrate_trading_coordination`](#demonstrate_trading_coordination)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)

### Functions (privy)

- [`new`](#new)

### Functions (graph)

- [`run_demo`](#run_demo)

### Functions (solana)

- [`run_demo`](#run_demo)

### Functions (web)

- [`run_demo`](#run_demo)

### Functions (evm)

- [`run_demo`](#run_demo)

### Functions (cross_chain)

- [`run_demo`](#run_demo)

### Functions (agents)

- [`run_demo`](#run_demo)

### Functions (interactive)

- [`run_chat`](#run_chat)

### Functions (notifier)

- [`add_channel`](#add_channel)
- [`add_routing_rule`](#add_routing_rule)
- [`matches`](#matches)
- [`matches`](#matches)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`set_default_channel`](#set_default_channel)
- [`with_format_template`](#with_format_template)
- [`with_header`](#with_header)
- [`with_html_mode`](#with_html_mode)
- [`with_identity`](#with_identity)
- [`without_colors`](#without_colors)

### Traits

- [`NotificationChannel`](#notificationchannel)
- [`OutputProcessor`](#outputprocessor)

### Functions (formatter)

- [`add_format`](#add_format)
- [`compact`](#compact)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`standard_formats`](#standard_formats)
- [`with_css_classes`](#with_css_classes)
- [`with_field_mapping`](#with_field_mapping)
- [`with_options`](#with_options)
- [`with_template`](#with_template)
- [`without_metadata`](#without_metadata)
- [`without_styles`](#without_styles)

### Functions (mod)

- [`add_processor`](#add_processor)
- [`error_output`](#error_output)
- [`info`](#info)
- [`new`](#new)
- [`process`](#process)
- [`success_output`](#success_output)
- [`user_friendly_error`](#user_friendly_error)
- [`with_metadata`](#with_metadata)
- [`with_timing`](#with_timing)

### Functions (distiller)

- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`with_config`](#with_config)
- [`with_response`](#with_response)

## Structs

### ConsoleChannel

**Source**: `processors/notifier.rs`

```rust
pub struct ConsoleChannel { /// Whether to use ANSI color codes in console output use_colors: bool, }
```

Console/log notification channel for debugging

---

### DiscordChannel

**Source**: `processors/notifier.rs`

```rust
pub struct DiscordChannel { /// Discord webhook URL for sending messages #[allow(dead_code)]
```

Discord webhook notification channel

---

### DistillationProcessor

**Source**: `processors/distiller.rs`

```rust
pub struct DistillationProcessor { model: String, max_tokens: Option<u32>, temperature: Option<f32>, #[allow(dead_code)]
```

LLM-based output distiller

Uses a separate LLM call to summarize complex tool outputs.
This pattern is useful for making technical outputs more accessible to users.

---

### HtmlFormatter

**Source**: `processors/formatter.rs`

```rust
pub struct HtmlFormatter { css_classes: HashMap<String, String>, include_styles: bool, }
```

HTML formatter for tool outputs

---

### JsonFormatter

**Source**: `processors/formatter.rs`

```rust
pub struct JsonFormatter { pretty_print: bool, include_metadata: bool, field_mappings: HashMap<String, String>, }
```

JSON formatter that can restructure and clean up outputs

---

### MarkdownFormatter

**Source**: `processors/formatter.rs`

```rust
pub struct MarkdownFormatter { include_metadata: bool, include_timing: bool, custom_templates: HashMap<String, String>, }
```

Markdown formatter for tool outputs

Converts tool outputs into clean, readable Markdown format.
Useful for documentation, reports, or display in Markdown-aware interfaces.

---

### MarketIntelligenceAgent

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct MarketIntelligenceAgent { /// Unique identifier for this agent id: AgentId, /// Communication channel for inter-agent messaging communication: Arc<ChannelCommunication>, /// Configuration settings _config: Config, /// Shared trading state reference _trading_state: Arc<Mutex<TradingState>>, }
```

Market research agent that uses real data sources

---

### MockDistiller

**Source**: `processors/distiller.rs`

```rust
pub struct MockDistiller { responses: std::collections::HashMap<String, String>, }
```

Mock distiller for testing without API calls

---

### MultiFormatProcessor

**Source**: `processors/formatter.rs`

```rust
pub struct MultiFormatProcessor { formats: Vec<Box<dyn OutputProcessor>>, }
```

Multi-format processor that can output in multiple formats simultaneously

---

### NotificationResult

**Source**: `processors/notifier.rs`

**Attributes**:
```rust
#[derive(Clone, serde::Serialize, serde::Deserialize)]
```

```rust
pub struct NotificationResult { /// Name of the channel that was used for the notification pub channel: String, /// Whether the notification was sent successfully pub success: bool, /// Optional message ID returned by the notification service pub message_id: Option<String>, /// Optional error message if the notification failed pub error: Option<String>, }
```

Result of a notification attempt

---

### NotificationRouter

**Source**: `processors/notifier.rs`

```rust
pub struct NotificationRouter { /// Map of channel names to notification channel implementations channels: HashMap<String, Box<dyn NotificationChannel>>, /// List of routing rules that determine which channels to use routing_rules: Vec<RoutingRule>, /// Default channel to use when no routing rules match default_channel: Option<String>, }
```

Notification router that sends outputs to various channels

---

### Order

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct Order { /// Unique identifier for this order pub id: String, /// Token symbol or address being traded pub symbol: String, /// Buy or sell side of the order pub side: OrderSide, /// Amount of tokens to trade pub amount: f64, /// Target price (None for market orders)
```

Represents a trading order (pending or executed)

---

### Position

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct Position { /// Token symbol or address pub symbol: String, /// Amount of tokens held pub amount: f64, /// Average entry price per token pub avg_price: f64, /// Current market price per token pub current_price: f64, /// Profit and loss for this position pub pnl: f64, /// Blockchain network where the position exists pub network: String, /// Timestamp when the position was opened pub entry_time: chrono::DateTime<chrono::Utc>, }
```

Represents an active trading position

---

### PrivySignerFactory

**Source**: `auth/privy.rs`

```rust
pub struct PrivySignerFactory { privy_app_id: String, privy_app_secret: String, }
```

Privy-specific signer factory implementation

---

### PrivyUserData

**Source**: `auth/privy.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct PrivyUserData { pub id: String, pub solana_address: Option<String>, pub evm_address: Option<String>, pub evm_wallet_id: Option<String>, pub verified: bool, }
```

Privy user data structure

---

### ProcessedOutput

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ProcessedOutput { /// The original tool output before processing pub original: ToolOutput, /// The result after processing (transformed, formatted, etc.)
```

Processed output after going through a processor

---

### ProcessorPipeline

**Source**: `processors/mod.rs`

```rust
pub struct ProcessorPipeline { processors: Vec<Box<dyn OutputProcessor>>, }
```

A pipeline that chains multiple processors together

This demonstrates the composability pattern - processors can be chained
to create complex processing workflows.

---

### RiskManagementAgent

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct RiskManagementAgent { /// Unique identifier for this agent id: AgentId, /// Communication channel for inter-agent messaging communication: Arc<ChannelCommunication>, /// Configuration settings _config: Config, /// Shared trading state reference trading_state: Arc<Mutex<TradingState>>, /// Maximum position size as percentage of portfolio max_position_size: f64, /// Maximum daily loss as percentage of portfolio max_daily_loss: f64, }
```

Risk management agent with real portfolio tracking

---

### RoutingInfo

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RoutingInfo { /// The target channel for notification routing pub channel: String, /// List of recipients to notify pub recipients: Vec<String>, /// Priority level for the notification pub priority: NotificationPriority, }
```

Routing information for notifications

---

### RoutingRule

**Source**: `processors/notifier.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct RoutingRule { /// Human-readable name for this rule pub name: String, /// Condition that determines when this rule applies pub condition: RoutingCondition, /// List of channel names to route to when condition matches pub channels: Vec<String>, }
```

Routing rule for determining which channels to use

---

### SmartDistiller

**Source**: `processors/distiller.rs`

```rust
pub struct SmartDistiller { processors: Vec<DistillationProcessor>, }
```

Smart distiller that chooses different strategies based on output type

---

### TelegramChannel

**Source**: `processors/notifier.rs`

```rust
pub struct TelegramChannel { /// Telegram bot token for API authentication #[allow(dead_code)]
```

Telegram bot notification channel

---

### ToolOutput

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct ToolOutput { /// Name of the tool that produced this output pub tool_name: String, /// Whether the tool execution was successful pub success: bool, /// The actual result data from the tool execution pub result: serde_json::Value, /// Error message if the tool execution failed pub error: Option<String>, /// Execution time in milliseconds pub execution_time_ms: u64, /// Additional metadata about the tool execution pub metadata: HashMap<String, String>, }
```

Represents the output from a tool execution

---

### TradeExecutionAgent

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct TradeExecutionAgent { /// Unique identifier for this agent id: AgentId, /// Communication channel for inter-agent messaging communication: Arc<ChannelCommunication>, /// Configuration settings _config: Config, /// Shared trading state reference _trading_state: Arc<Mutex<TradingState>>, }
```

Execution agent that performs real blockchain trades

---

### TradingState

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TradingState { /// Map of symbol to active trading positions pub active_positions: HashMap<String, Position>, /// List of pending orders awaiting execution pub pending_orders: Vec<Order>, /// Total portfolio value in USD pub portfolio_value_usd: f64, /// Available ETH balance for trading pub available_balance_eth: f64, /// Available SOL balance for trading pub available_balance_sol: f64, /// Daily profit and loss percentage pub daily_pnl: f64, /// Current risk exposure as percentage of portfolio pub risk_exposure: f64, }
```

Shared trading state across all agents

---

### WebhookChannel

**Source**: `processors/notifier.rs`

```rust
pub struct WebhookChannel { /// Human-readable name for this webhook name: String, /// Webhook URL endpoint #[allow(dead_code)]
```

Generic webhook notification channel

---

## Enums

### NotificationPriority

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub enum NotificationPriority { /// Low priority notification Low, /// Normal priority notification Normal, /// High priority notification High, /// Critical priority notification Critical, }
```

Notification priority levels

**Variants**:

- `Low`
- `Normal`
- `High`
- `Critical`

---

### OrderSide

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum OrderSide { /// Buy order (acquire tokens) Buy, /// Sell order (dispose of tokens) Sell, }
```

Specifies the side of a trading order

**Variants**:

- `Buy`
- `Sell`

---

### OrderStatus

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum OrderStatus { /// Order is waiting to be executed Pending, /// Order is currently being processed Executing, /// Order has been successfully executed Completed, /// Order execution failed Failed, }
```

Represents the current status of a trading order

**Variants**:

- `Pending`
- `Executing`
- `Completed`
- `Failed`

---

### OutputFormat

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub enum OutputFormat { /// JSON format output Json, /// Markdown format output Markdown, /// Plain text format output PlainText, /// HTML format output Html, /// Custom format with specified type name Custom(String), }
```

Output format types

**Variants**:

- `Json`
- `Markdown`
- `PlainText`
- `Html`
- `Custom(String)`

---

### RoutingCondition

**Source**: `processors/notifier.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub enum RoutingCondition { /// Always matches any output Always, /// Matches only successful outputs OnSuccess, /// Matches only failed outputs OnError, /// Matches outputs from a specific tool by exact name ToolName(String), /// Matches outputs from tools whose name contains the given substring ToolNameContains(String), /// Matches outputs that took longer than the specified number of milliseconds ExecutionTimeOver(u64), // milliseconds /// Matches outputs that have metadata with the specified key HasMetadata(String), /// Matches when all nested conditions match And(Vec<RoutingCondition>), /// Matches when any nested condition matches Or(Vec<RoutingCondition>), /// Matches when the nested condition does not match Not(Box<RoutingCondition>), }
```

Conditions for routing decisions

**Variants**:

- `Always`
- `OnSuccess`
- `OnError`
- `ToolName(String)`
- `ToolNameContains(String)`
- `ExecutionTimeOver(u64)`
- `HasMetadata(String)`
- `And(Vec<RoutingCondition>)`
- `Or(Vec<RoutingCondition>)`
- `Not(Box<RoutingCondition>)`

---

## Functions (trading_coordination)

### demonstrate_trading_coordination

**Source**: `agents/trading_coordination.rs`

```rust
pub async fn demonstrate_trading_coordination( config: Config, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

Demonstration function that shows the complete trading coordination workflow

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

Creates a new market intelligence agent

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

Creates a new risk management agent

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

Creates a new trade execution agent

---

## Functions (privy)

### new

**Source**: `auth/privy.rs`

```rust
pub fn new(app_id: String, app_secret: String) -> Self
```

Create a new Privy signer factory

# Arguments
* `app_id` - Privy application ID
* `app_secret` - Privy application secret

---

## Functions (graph)

### run_demo

**Source**: `commands/graph.rs`

```rust
pub async fn run_demo(_config: Arc<Config>, init: bool, query: Option<String>) -> Result<()>
```

Run the graph memory demo.

---

## Functions (solana)

### run_demo

**Source**: `commands/solana.rs`

```rust
pub async fn run_demo(config: Arc<Config>, address: Option<String>) -> Result<()>
```

Run the Solana tools demo.

---

## Functions (web)

### run_demo

**Source**: `commands/web.rs`

```rust
pub async fn run_demo(config: Arc<Config>, query: String) -> Result<()>
```

Run the web tools demo.

---

## Functions (evm)

### run_demo

**Source**: `commands/evm.rs`

```rust
pub async fn run_demo(config: Arc<Config>, address: Option<String>, chain_id: u64) -> Result<()>
```

Run the EVM tools demo.

---

## Functions (cross_chain)

### run_demo

**Source**: `commands/cross_chain.rs`

```rust
pub async fn run_demo(config: Arc<Config>, token: String) -> Result<()>
```

Run the cross-chain analysis demo.

---

## Functions (agents)

### run_demo

**Source**: `commands/agents.rs`

```rust
pub async fn run_demo(config: Arc<Config>, scenario: String) -> Result<()>
```

Runs a multi-agent coordination demonstration based on the specified scenario.

This function demonstrates the riglr-agents framework through various predefined scenarios:
- `"trading"`: Real-world trading coordination with blockchain operations
- `"risk"`: Risk management system with coordinated assessment across multiple agents
- `"basic"`: Fundamental multi-agent communication and workflow patterns

# Arguments
* `config` - Shared configuration for all agents and blockchain operations
* `scenario` - The demonstration scenario to execute

# Returns
Returns `Ok(())` on successful demonstration completion, or an error if the scenario
is unknown or the demonstration fails.

# Examples
```
use std::sync::Arc;
use riglr_config::Config;

# async fn example() -> anyhow::Result<()> {
let config = Arc::new(Config::default());
run_demo(config, "basic".to_string()).await?;
# Ok(())
# }
```

---

## Functions (interactive)

### run_chat

**Source**: `commands/interactive.rs`

```rust
pub async fn run_chat(config: Arc<Config>) -> Result<()>
```

Run interactive chat mode.

---

## Functions (notifier)

### add_channel

**Source**: `processors/notifier.rs`

```rust
pub fn add_channel<C: NotificationChannel + 'static>(mut self, name: &str, channel: C) -> Self
```

Adds a notification channel with the given name

---

### add_routing_rule

**Source**: `processors/notifier.rs`

```rust
pub fn add_routing_rule(mut self, rule: RoutingRule) -> Self
```

Adds a routing rule for determining which channels to use

---

### matches

**Source**: `processors/notifier.rs`

```rust
pub fn matches(&self, output: &ToolOutput) -> bool
```

Checks if this routing rule matches the given tool output

---

### matches

**Source**: `processors/notifier.rs`

```rust
pub fn matches(&self, output: &ToolOutput) -> bool
```

Evaluates whether this condition matches the given tool output

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new() -> Self
```

Creates a new notification router with no channels or rules

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(name: &str, condition: RoutingCondition, channels: Vec<String>) -> Self
```

Creates a new routing rule with the given name, condition, and target channels

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(webhook_url: &str) -> Self
```

Creates a new Discord channel with the given webhook URL

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(bot_token: &str, chat_id: &str) -> Self
```

Creates a new Telegram channel with the given bot token and chat ID

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(name: &str, url: &str) -> Self
```

Creates a new webhook channel with the given name and URL

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new() -> Self
```

Creates a new console channel with color output enabled

---

### set_default_channel

**Source**: `processors/notifier.rs`

```rust
pub fn set_default_channel(mut self, name: &str) -> Self
```

Sets the default channel to use when no routing rules match

---

### with_format_template

**Source**: `processors/notifier.rs`

```rust
pub fn with_format_template(mut self, template: &str) -> Self
```

Sets a custom format template for webhook message bodies

---

### with_header

**Source**: `processors/notifier.rs`

```rust
pub fn with_header(mut self, key: &str, value: &str) -> Self
```

Adds an HTTP header to include with webhook requests

---

### with_html_mode

**Source**: `processors/notifier.rs`

```rust
pub fn with_html_mode(mut self) -> Self
```

Switches message formatting from Markdown to HTML mode

---

### with_identity

**Source**: `processors/notifier.rs`

```rust
pub fn with_identity(mut self, username: &str, avatar_url: Option<&str>) -> Self
```

Sets the bot username and optional avatar URL for Discord messages

---

### without_colors

**Source**: `processors/notifier.rs`

```rust
pub fn without_colors(mut self) -> Self
```

Disables color output for plain text logging

---

## Traits

### NotificationChannel

**Source**: `processors/notifier.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait NotificationChannel: Send + Sync { ... }
```

Trait for notification channels

**Methods**:

#### `send_notification`

```rust
async fn send_notification(&self, output: &ToolOutput) -> Result<String>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

#### `supports_formatting`

```rust
fn supports_formatting(&self) -> bool {
```

---

### OutputProcessor

**Source**: `processors/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait OutputProcessor: Send + Sync { ... }
```

Core trait for output processors

This trait defines the interface for all output processors.
Implementations can transform, summarize, format, or route outputs.

**Methods**:

#### `process`

```rust
async fn process(&self, input: ToolOutput) -> Result<ProcessedOutput>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

#### `can_process`

```rust
fn can_process(&self, _output: &ToolOutput) -> bool {
```

#### `config`

```rust
fn config(&self) -> serde_json::Value {
```

---

## Functions (formatter)

### add_format

**Source**: `processors/formatter.rs`

```rust
pub fn add_format<F: OutputProcessor + 'static>(mut self, formatter: F) -> Self
```

Add a formatter to the processor

---

### compact

**Source**: `processors/formatter.rs`

```rust
pub fn compact(mut self) -> Self
```

Configure formatter to output compact JSON

---

### new

**Source**: `processors/formatter.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new MarkdownFormatter with default settings

---

### new

**Source**: `processors/formatter.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new HtmlFormatter with default CSS classes and styles

---

### new

**Source**: `processors/formatter.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new JsonFormatter with default settings

---

### new

**Source**: `processors/formatter.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new MultiFormatProcessor with no formatters

---

### standard_formats

**Source**: `processors/formatter.rs`

```rust
pub fn standard_formats() -> Self
```

Create a MultiFormatProcessor with standard formatters (Markdown, HTML, JSON)

---

### with_css_classes

**Source**: `processors/formatter.rs`

```rust
pub fn with_css_classes(mut self, classes: HashMap<String, String>) -> Self
```

Add custom CSS classes, preserving existing defaults

---

### with_field_mapping

**Source**: `processors/formatter.rs`

```rust
pub fn with_field_mapping(mut self, from: &str, to: &str) -> Self
```

Add a field name mapping for JSON transformation

---

### with_options

**Source**: `processors/formatter.rs`

```rust
pub fn with_options(include_metadata: bool, include_timing: bool) -> Self
```

Create a MarkdownFormatter with custom metadata and timing options

---

### with_template

**Source**: `processors/formatter.rs`

```rust
pub fn with_template(mut self, tool_name: &str, template: &str) -> Self
```

Add a custom template for specific tool types

---

### without_metadata

**Source**: `processors/formatter.rs`

```rust
pub fn without_metadata(mut self) -> Self
```

Configure formatter to exclude metadata from output

---

### without_styles

**Source**: `processors/formatter.rs`

```rust
pub fn without_styles(mut self) -> Self
```

Disable inline CSS styles in output

---

## Functions (mod)

### add_processor

**Source**: `processors/mod.rs`

```rust
pub fn add_processor<P: OutputProcessor + 'static>(mut self, processor: P) -> Self
```

Add a processor to the pipeline

---

### error_output

**Source**: `processors/mod.rs`

```rust
pub fn error_output(tool_name: &str, error: &str) -> ToolOutput
```

Create a ToolOutput from a failed operation

---

### info

**Source**: `processors/mod.rs`

```rust
pub fn info(&self) -> Vec<serde_json::Value>
```

Get information about all processors in the pipeline

---

### new

**Source**: `processors/mod.rs`

```rust
pub fn new() -> Self
```

Create a new empty pipeline

---

### process

**Source**: `processors/mod.rs`

```rust
pub async fn process(&self, mut output: ToolOutput) -> Result<ProcessedOutput>
```

Process output through the entire pipeline

---

### success_output

**Source**: `processors/mod.rs`

```rust
pub fn success_output(tool_name: &str, result: serde_json::Value) -> ToolOutput
```

Create a ToolOutput from a successful operation

---

### user_friendly_error

**Source**: `processors/mod.rs`

```rust
pub fn user_friendly_error(output: &ToolOutput) -> String
```

Extract error message from a ToolOutput in a user-friendly way

---

### with_metadata

**Source**: `processors/mod.rs`

```rust
pub fn with_metadata(mut output: ToolOutput, key: &str, value: &str) -> ToolOutput
```

Add metadata to a ToolOutput

---

### with_timing

**Source**: `processors/mod.rs`

```rust
pub fn with_timing(mut output: ToolOutput, start_time: SystemTime) -> ToolOutput
```

Add timing information to a ToolOutput

---

## Functions (distiller)

### new

**Source**: `processors/distiller.rs`

```rust
pub fn new(model: &str) -> Self
```

Create a new distillation processor with a specific model

---

### new

**Source**: `processors/distiller.rs`

```rust
pub fn new() -> Self
```

Create a new SmartDistiller with default processors

---

### new

**Source**: `processors/distiller.rs`

**Attributes**:
```rust
#[must_use]
```

```rust
pub fn new() -> Self
```

Create a new MockDistiller with default responses

---

### with_config

**Source**: `processors/distiller.rs`

```rust
pub fn with_config( model: &str, max_tokens: Option<u32>, temperature: Option<f32>, system_prompt: Option<String>, ) -> Self
```

Create a processor with custom settings

---

### with_response

**Source**: `processors/distiller.rs`

```rust
pub fn with_response(mut self, tool_name: &str, response: &str) -> Self
```

Add a custom response for a specific tool name

---


---

*This documentation was automatically generated from the source code.*