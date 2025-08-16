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

### Functions

- [`add_channel`](#add_channel)
- [`add_format`](#add_format)
- [`add_processor`](#add_processor)
- [`add_routing_rule`](#add_routing_rule)
- [`compact`](#compact)
- [`demonstrate_trading_coordination`](#demonstrate_trading_coordination)
- [`error_output`](#error_output)
- [`info`](#info)
- [`matches`](#matches)
- [`matches`](#matches)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`process`](#process)
- [`run_chat`](#run_chat)
- [`run_demo`](#run_demo)
- [`run_demo`](#run_demo)
- [`run_demo`](#run_demo)
- [`run_demo`](#run_demo)
- [`run_demo`](#run_demo)
- [`run_demo`](#run_demo)
- [`set_default_channel`](#set_default_channel)
- [`standard_formats`](#standard_formats)
- [`success_output`](#success_output)
- [`user_friendly_error`](#user_friendly_error)
- [`with_config`](#with_config)
- [`with_css_classes`](#with_css_classes)
- [`with_field_mapping`](#with_field_mapping)
- [`with_format_template`](#with_format_template)
- [`with_header`](#with_header)
- [`with_html_mode`](#with_html_mode)
- [`with_identity`](#with_identity)
- [`with_metadata`](#with_metadata)
- [`with_options`](#with_options)
- [`with_response`](#with_response)
- [`with_template`](#with_template)
- [`with_timing`](#with_timing)
- [`without_colors`](#without_colors)
- [`without_metadata`](#without_metadata)
- [`without_styles`](#without_styles)

### Traits

- [`NotificationChannel`](#notificationchannel)
- [`OutputProcessor`](#outputprocessor)

## Structs

### ConsoleChannel

**Source**: `processors/notifier.rs`

```rust
pub struct ConsoleChannel { use_colors: bool, }
```

Console/log notification channel for debugging

---

### DiscordChannel

**Source**: `processors/notifier.rs`

```rust
pub struct DiscordChannel { #[allow(dead_code)]
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
pub struct MarketIntelligenceAgent { id: AgentId, communication: Arc<ChannelCommunication>, _config: Config, _trading_state: Arc<Mutex<TradingState>>, }
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
pub struct NotificationResult { pub channel: String, pub success: bool, pub message_id: Option<String>, pub error: Option<String>, }
```

Result of a notification attempt

---

### NotificationRouter

**Source**: `processors/notifier.rs`

```rust
pub struct NotificationRouter { channels: HashMap<String, Box<dyn NotificationChannel>>, routing_rules: Vec<RoutingRule>, default_channel: Option<String>, }
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
pub struct Order { pub id: String, pub symbol: String, pub side: OrderSide, pub amount: f64, pub price: Option<f64>, pub network: String, pub status: OrderStatus, }
```

---

### Position

**Source**: `agents/trading_coordination.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct Position { pub symbol: String, pub amount: f64, pub avg_price: f64, pub current_price: f64, pub pnl: f64, pub network: String, pub entry_time: chrono::DateTime<chrono::Utc>, }
```

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
pub struct ProcessedOutput { pub original: ToolOutput, pub processed_result: serde_json::Value, pub format: OutputFormat, pub summary: Option<String>, pub routing_info: Option<RoutingInfo>, }
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
pub struct RiskManagementAgent { id: AgentId, communication: Arc<ChannelCommunication>, _config: Config, trading_state: Arc<Mutex<TradingState>>, max_position_size: f64, max_daily_loss: f64, }
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
pub struct RoutingInfo { pub channel: String, pub recipients: Vec<String>, pub priority: NotificationPriority, }
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
pub struct RoutingRule { pub name: String, pub condition: RoutingCondition, pub channels: Vec<String>, }
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
pub struct TelegramChannel { #[allow(dead_code)]
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
pub struct ToolOutput { pub tool_name: String, pub success: bool, pub result: serde_json::Value, pub error: Option<String>, pub execution_time_ms: u64, pub metadata: HashMap<String, String>, }
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
pub struct TradeExecutionAgent { id: AgentId, communication: Arc<ChannelCommunication>, _config: Config, _trading_state: Arc<Mutex<TradingState>>, }
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
pub struct TradingState { pub active_positions: HashMap<String, Position>, pub pending_orders: Vec<Order>, pub portfolio_value_usd: f64, pub available_balance_eth: f64, pub available_balance_sol: f64, pub daily_pnl: f64, pub risk_exposure: f64, }
```

Shared trading state across all agents

---

### WebhookChannel

**Source**: `processors/notifier.rs`

```rust
pub struct WebhookChannel { name: String, #[allow(dead_code)]
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
pub enum NotificationPriority { Low, Normal, High, Critical, }
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
pub enum OrderSide { Buy, Sell, }
```

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
pub enum OrderStatus { Pending, Executing, Completed, Failed, }
```

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
pub enum OutputFormat { Json, Markdown, PlainText, Html, Custom(String), }
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
pub enum RoutingCondition { Always, OnSuccess, OnError, ToolName(String), ToolNameContains(String), ExecutionTimeOver(u64), // milliseconds HasMetadata(String), And(Vec<RoutingCondition>), Or(Vec<RoutingCondition>), Not(Box<RoutingCondition>), }
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

## Functions

### add_channel

**Source**: `processors/notifier.rs`

```rust
pub fn add_channel<C: NotificationChannel + 'static>(mut self, name: &str, channel: C) -> Self
```

---

### add_format

**Source**: `processors/formatter.rs`

```rust
pub fn add_format<F: OutputProcessor + 'static>(mut self, formatter: F) -> Self
```

---

### add_processor

**Source**: `processors/mod.rs`

```rust
pub fn add_processor<P: OutputProcessor + 'static>(mut self, processor: P) -> Self
```

Add a processor to the pipeline

---

### add_routing_rule

**Source**: `processors/notifier.rs`

```rust
pub fn add_routing_rule(mut self, rule: RoutingRule) -> Self
```

---

### compact

**Source**: `processors/formatter.rs`

```rust
pub fn compact(mut self) -> Self
```

---

### demonstrate_trading_coordination

**Source**: `agents/trading_coordination.rs`

```rust
pub async fn demonstrate_trading_coordination( config: Config, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

Demonstration function that shows the complete trading coordination workflow

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

### matches

**Source**: `processors/notifier.rs`

```rust
pub fn matches(&self, output: &ToolOutput) -> bool
```

---

### matches

**Source**: `processors/notifier.rs`

```rust
pub fn matches(&self, output: &ToolOutput) -> bool
```

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

---

### new

**Source**: `agents/trading_coordination.rs`

```rust
pub fn new( id: &str, communication: Arc<ChannelCommunication>, config: Config, trading_state: Arc<Mutex<TradingState>>, ) -> Self
```

---

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

### new

**Source**: `processors/formatter.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/formatter.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/formatter.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/formatter.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/mod.rs`

```rust
pub fn new() -> Self
```

Create a new empty pipeline

---

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

---

### new

**Source**: `processors/distiller.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(name: &str, condition: RoutingCondition, channels: Vec<String>) -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(webhook_url: &str) -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(bot_token: &str, chat_id: &str) -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new(name: &str, url: &str) -> Self
```

---

### new

**Source**: `processors/notifier.rs`

```rust
pub fn new() -> Self
```

---

### process

**Source**: `processors/mod.rs`

```rust
pub async fn process(&self, mut output: ToolOutput) -> Result<ProcessedOutput>
```

Process output through the entire pipeline

---

### run_chat

**Source**: `commands/interactive.rs`

```rust
pub async fn run_chat(config: Arc<Config>) -> Result<()>
```

Run interactive chat mode.

---

### run_demo

**Source**: `commands/cross_chain.rs`

```rust
pub async fn run_demo(config: Arc<Config>, token: String) -> Result<()>
```

Run the cross-chain analysis demo.

---

### run_demo

**Source**: `commands/evm.rs`

```rust
pub async fn run_demo(config: Arc<Config>, address: Option<String>, chain_id: u64) -> Result<()>
```

Run the EVM tools demo.

---

### run_demo

**Source**: `commands/graph.rs`

```rust
pub async fn run_demo(_config: Arc<Config>, init: bool, query: Option<String>) -> Result<()>
```

Run the graph memory demo.

---

### run_demo

**Source**: `commands/solana.rs`

```rust
pub async fn run_demo(config: Arc<Config>, address: Option<String>) -> Result<()>
```

Run the Solana tools demo.

---

### run_demo

**Source**: `commands/web.rs`

```rust
pub async fn run_demo(config: Arc<Config>, query: String) -> Result<()>
```

Run the web tools demo.

---

### run_demo

**Source**: `commands/agents.rs`

```rust
pub async fn run_demo(config: Arc<Config>, scenario: String) -> Result<()>
```

---

### set_default_channel

**Source**: `processors/notifier.rs`

```rust
pub fn set_default_channel(mut self, name: &str) -> Self
```

---

### standard_formats

**Source**: `processors/formatter.rs`

```rust
pub fn standard_formats() -> Self
```

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

### with_config

**Source**: `processors/distiller.rs`

```rust
pub fn with_config( model: &str, max_tokens: Option<u32>, temperature: Option<f32>, system_prompt: Option<String>, ) -> Self
```

Create a processor with custom settings

---

### with_css_classes

**Source**: `processors/formatter.rs`

```rust
pub fn with_css_classes(mut self, classes: HashMap<String, String>) -> Self
```

---

### with_field_mapping

**Source**: `processors/formatter.rs`

```rust
pub fn with_field_mapping(mut self, from: &str, to: &str) -> Self
```

---

### with_format_template

**Source**: `processors/notifier.rs`

```rust
pub fn with_format_template(mut self, template: &str) -> Self
```

---

### with_header

**Source**: `processors/notifier.rs`

```rust
pub fn with_header(mut self, key: &str, value: &str) -> Self
```

---

### with_html_mode

**Source**: `processors/notifier.rs`

```rust
pub fn with_html_mode(mut self) -> Self
```

---

### with_identity

**Source**: `processors/notifier.rs`

```rust
pub fn with_identity(mut self, username: &str, avatar_url: Option<&str>) -> Self
```

---

### with_metadata

**Source**: `processors/mod.rs`

```rust
pub fn with_metadata(mut output: ToolOutput, key: &str, value: &str) -> ToolOutput
```

Add metadata to a ToolOutput

---

### with_options

**Source**: `processors/formatter.rs`

```rust
pub fn with_options(include_metadata: bool, include_timing: bool) -> Self
```

---

### with_response

**Source**: `processors/distiller.rs`

```rust
pub fn with_response(mut self, tool_name: &str, response: &str) -> Self
```

---

### with_template

**Source**: `processors/formatter.rs`

```rust
pub fn with_template(mut self, tool_name: &str, template: &str) -> Self
```

Add a custom template for specific tool types

---

### with_timing

**Source**: `processors/mod.rs`

```rust
pub fn with_timing(mut output: ToolOutput, start_time: SystemTime) -> ToolOutput
```

Add timing information to a ToolOutput

---

### without_colors

**Source**: `processors/notifier.rs`

```rust
pub fn without_colors(mut self) -> Self
```

---

### without_metadata

**Source**: `processors/formatter.rs`

```rust
pub fn without_metadata(mut self) -> Self
```

---

### without_styles

**Source**: `processors/formatter.rs`

```rust
pub fn without_styles(mut self) -> Self
```

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


---

*This documentation was automatically generated from the source code.*