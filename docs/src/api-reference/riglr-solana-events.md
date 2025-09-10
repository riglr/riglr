# riglr-solana-events

{{#include ../../../riglr-solana-events/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `AccountMeta`

Account metadata for swap

---

#### `BatchEventParser`

Batch processor for efficient parsing of multiple transactions

---

#### `BatchParserStats`

Statistics for batch parser performance monitoring

---

#### `BonkEventParser`

Bonk event parser

---

#### `BonkPoolCreateEvent`

Create pool event

---

#### `BonkTradeEvent`

Trade event

---

#### `BurnNftInstruction`

Metaplex BurnNft instruction data

---

#### `CacheStats`

Cache statistics

---

#### `ConstantCurve`

Constant product bonding curve parameters

---

#### `CreateMetadataAccountInstruction`

Metaplex CreateMetadataAccount instruction data

---

#### `CurveParams`

Container for different bonding curve configurations

---

#### `CustomDeserializer`

Custom deserializer for hot path parsing

---

#### `DepositInstruction`

Raydium V4 Deposit instruction data

---

#### `DlmmBin`

Meteora DLMM bin information

---

#### `DlmmPairConfig`

Meteora DLMM pair configuration

---

#### `EnrichmentConfig`

Configuration for event enrichment

---

#### `EventEnricher`

Event enricher with caching and external API integration

---

#### `EventParameters`

Common event parameters shared across all protocol events

---

#### `EventParserRegistry`

EventParserRegistry - the new async event parser registry

---

#### `FixedCurve`

Fixed price bonding curve parameters

---

#### `GenericEventParseConfig`

Generic event parser configuration

---

#### `GenericEventParser`

Generic event parser base class

---

#### `InnerInstructionParseParams`

Parameters for parsing events from inner instructions, reducing function parameter count

---

#### `InstructionParseParams`

Parameters for parsing events from instructions, reducing function parameter count

---

#### `JupiterAccountLayout`

Jupiter program account layout

---

#### `JupiterEventParser`

Jupiter event parser

---

#### `JupiterExactOutRouteInstruction`

Jupiter ExactOutRoute instruction data

---

#### `JupiterLiquidityEvent`

Jupiter liquidity provision event

---

#### `JupiterParser`

High-performance Jupiter parser

---

#### `JupiterParserFactory`

Factory for creating Jupiter parsers

---

#### `JupiterRouteAnalysis`

Jupiter route analysis result

---

#### `JupiterRouteData`

Combined Jupiter Route Data (instruction + analysis)

---

#### `JupiterRouteInstruction`

Jupiter Route instruction data

---

#### `JupiterSwapBorshEvent`

Jupiter swap event with borsh (for simple events)

---

#### `JupiterSwapData`

Jupiter swap event data (for UnifiedEvent)

---

#### `JupiterSwapEvent`

Jupiter swap event

---

#### `LinearCurve`

Linear bonding curve parameters

---

#### `LiquidityEventParams`

Parameters for creating liquidity events, reducing function parameter count

---

#### `MarginFiAccount`

MarginFi user account

---

#### `MarginFiBalance`

MarginFi account balance

---

#### `MarginFiBankConfig`

MarginFi bank configuration

---

#### `MarginFiBankState`

MarginFi bank state

---

#### `MarginFiBorrowData`

MarginFi borrow data

---

#### `MarginFiBorrowEvent`

MarginFi borrow event

---

#### `MarginFiDepositData`

MarginFi deposit data

---

#### `MarginFiDepositEvent`

MarginFi deposit event

---

#### `MarginFiEventParser`

MarginFi event parser

---

#### `MarginFiLendingAccount`

MarginFi lending account

---

#### `MarginFiLiquidationData`

MarginFi liquidation data

---

#### `MarginFiLiquidationEvent`

MarginFi liquidation event

---

#### `MarginFiRepayData`

MarginFi repay data

---

#### `MarginFiRepayEvent`

MarginFi repay event

---

#### `MarginFiWithdrawData`

MarginFi withdraw data

---

#### `MarginFiWithdrawEvent`

MarginFi withdraw event

---

#### `MemoryMappedParser`

High-performance parser using memory-mapped files for large transaction logs

---

#### `MetaplexEventAnalysis`

Metaplex marketplace event analysis

---

#### `MetaplexParser`

High-performance Metaplex parser

---

#### `MetaplexParserFactory`

Factory for creating Metaplex parsers

---

#### `MeteoraDynamicLiquidityData`

Meteora Dynamic liquidity data

---

#### `MeteoraDynamicLiquidityEvent`

Meteora Dynamic AMM liquidity event

---

#### `MeteoraDynamicPoolData`

Meteora Dynamic AMM pool data

---

#### `MeteoraEventParser`

Meteora event parser

---

#### `MeteoraLiquidityData`

Meteora DLMM liquidity data

---

#### `MeteoraLiquidityEvent`

Meteora DLMM liquidity event

---

#### `MeteoraSwapData`

Meteora DLMM swap data

---

#### `MeteoraSwapEvent`

Meteora DLMM swap event

---

#### `MintParams`

Parameters for minting new tokens in the BONK protocol

---

#### `OrcaEventParser`

Orca Whirlpool event parser

---

#### `OrcaLiquidityData`

Orca liquidity change data

---

#### `OrcaLiquidityEvent`

Orca liquidity event (increase/decrease)

---

#### `OrcaPositionData`

Orca position data

---

#### `OrcaPositionEvent`

Orca position event (open/close)

---

#### `OrcaSwapData`

Orca swap event data

---

#### `OrcaSwapEvent`

Orca Whirlpool swap event

---

#### `OwnedInnerInstructionParseParams`

Owned parameters for parsing events from inner instructions

---

#### `OwnedInstructionParseParams`

Owned parameters for parsing events from instructions

---

#### `ParserConfig`

Parser-specific configuration

---

#### `ParserMetrics`

Parser-specific metrics

---

#### `ParsingInput`

Input for the parsing pipeline

---

#### `ParsingMetrics`

Metrics for parsing performance

---

#### `ParsingOutput`

Output from the parsing pipeline

---

#### `ParsingPipeline`

High-performance parsing pipeline

---

#### `ParsingPipelineBuilder`

Builder for creating parsing pipelines

---

#### `ParsingPipelineConfig`

Configuration for the parsing pipeline

---

#### `PipelineStats`

Statistics for the parsing pipeline

---

#### `PositionRewardInfo`

Position reward information

---

#### `PriceData`

Price information for a token

---

#### `ProtocolEventParams`

Parameters for creating protocol events, reducing function parameter count

---

#### `PumpBuyInstruction`

PumpFun Buy instruction data

---

#### `PumpCreatePoolInstruction`

PumpFun CreatePool instruction data

---

#### `PumpDepositInstruction`

PumpFun Deposit instruction data

---

#### `PumpFunParser`

High-performance PumpFun parser

---

#### `PumpFunParserFactory`

Factory for creating PumpFun parsers

---

#### `PumpSellInstruction`

PumpFun Sell instruction data

---

#### `PumpSwapBuyEvent`

Buy event

---

#### `PumpSwapCreatePoolEvent`

Create pool event

---

#### `PumpSwapDepositEvent`

Deposit event

---

#### `PumpSwapEventParser`

PumpSwap event parser

---

#### `PumpSwapSellEvent`

Sell event

---

#### `PumpSwapWithdrawEvent`

Withdraw event

---

#### `PumpWithdrawInstruction`

PumpFun Withdraw instruction data

---

#### `RaydiumAmmV4DepositData`

Deposit event data for Raydium AMM V4

---

#### `RaydiumAmmV4DepositEvent`

Raydium AMM V4 deposit event

---

#### `RaydiumAmmV4EventParser`

Raydium AMM V4 event parser

---

#### `RaydiumAmmV4Initialize2Data`

Initialize2 event data for Raydium AMM V4

---

#### `RaydiumAmmV4Initialize2Event`

Raydium AMM V4 initialize2 event

---

#### `RaydiumAmmV4SwapData`

Swap event data for Raydium AMM V4

---

#### `RaydiumAmmV4SwapEvent`

Raydium AMM V4 swap event

---

#### `RaydiumAmmV4WithdrawData`

Withdraw event data for Raydium AMM V4

---

#### `RaydiumAmmV4WithdrawEvent`

Raydium AMM V4 withdraw event

---

#### `RaydiumAmmV4WithdrawPnlData`

WithdrawPnl event data for Raydium AMM V4

---

#### `RaydiumAmmV4WithdrawPnlEvent`

Raydium AMM V4 withdraw PNL event

---

#### `RaydiumClmmClosePositionEvent`

Raydium CLMM close position event

---

#### `RaydiumClmmCreatePoolEvent`

Raydium CLMM create pool event

---

#### `RaydiumClmmDecreaseLiquidityV2Event`

Raydium CLMM decrease liquidity V2 event

---

#### `RaydiumClmmEventParser`

Raydium CLMM event parser

---

#### `RaydiumClmmIncreaseLiquidityV2Event`

Raydium CLMM increase liquidity V2 event

---

#### `RaydiumClmmOpenPositionV2Event`

Raydium CLMM open position V2 event

---

#### `RaydiumClmmOpenPositionWithToken22NftEvent`

Raydium CLMM open position with Token-22 NFT event

---

#### `RaydiumClmmSwapEvent`

Raydium CLMM swap event

---

#### `RaydiumClmmSwapV2Event`

Raydium CLMM swap V2 event

---

#### `RaydiumCpmmDepositEvent`

Raydium CPMM Deposit event

---

#### `RaydiumCpmmEventParser`

Raydium CPMM event parser

---

#### `RaydiumCpmmSwapEvent`

Raydium CPMM Swap event

---

#### `RaydiumV4Parser`

High-performance Raydium V4 parser

---

#### `RaydiumV4ParserFactory`

Factory for creating Raydium V4 parsers

---

#### `RouteHop`

Simplified route hop representation

---

#### `RoutePlan`

Route plan information (simplified for event data)

---

#### `RoutePlanStep`

Route plan step for Jupiter swaps

---

#### `RpcConnectionPool`

Connection pool for RPC calls during parsing

---

#### `RuleResult`

Result from a validation rule

---

#### `SIMDPatternMatcher`

SIMD-optimized pattern matcher for instruction discriminators

---

#### `SharedAccountsExactOutRouteData`

Jupiter exact out route instruction data (after discriminator)

---

#### `SharedAccountsRouteData`

Jupiter shared accounts route instruction data (after discriminator)

---

#### `SolanaEvent`

A wrapper that implements the Event trait for Solana events

---

#### `SolanaEventMetadata`

Solana-specific event metadata that wraps core EventMetadata
and provides additional fields and trait implementations

---

#### `SolanaEventParser`

Solana event parser that bridges between legacy and new parsers

---

#### `SolanaInnerInstructionInput`

Input type for Solana inner instruction parsing

---

#### `SolanaInnerInstructionParser`

Inner instruction parser that implements the riglr-events-core EventParser trait

---

#### `StreamMetadata`

Metadata for streaming context

---

#### `SwapBaseInInstruction`

Raydium V4 SwapBaseIn instruction data

---

#### `SwapBaseOutInstruction`

Raydium V4 SwapBaseOut instruction data

---

#### `SwapData`

Data structure for token swap events

---

#### `SwapEventParams`

Parameters for creating swap events, reducing function parameter count

---

#### `SwapInfo`

Swap information within a route step

---

#### `TokenMetadata`

Token metadata information

---

#### `TransactionContext`

Transaction context information

---

#### `TransferData`

Data structure for token transfer events

---

#### `TransferInstruction`

Metaplex Transfer instruction data

---

#### `ValidationConfig`

Configuration for validation pipeline

---

#### `ValidationMetrics`

Validation metrics

---

#### `ValidationPipeline`

Data integrity validation pipeline

---

#### `ValidationResult`

Validation result for a single event

---

#### `ValidationStats`

Validation pipeline statistics

---

#### `VestingParams`

Parameters for token vesting schedules

---

#### `WhirlpoolAccount`

Orca Whirlpool account layout

---

#### `WhirlpoolRewardInfo`

Whirlpool reward information

---

#### `WithdrawInstruction`

Raydium V4 Withdraw instruction data

---

#### `ZeroCopyEvent`

Zero-copy base event that holds references to source data

---

#### `ZeroCopyLiquidityEvent`

Zero-copy liquidity event

---

#### `ZeroCopySwapEvent`

Specialized zero-copy swap event

---

### Enums

> Enumeration types for representing variants.

#### `EnrichmentError`

Error type for enrichment operations

**Variants:**

- `HttpError`
  - HTTP request failed
- `SerializationError`
  - Serialization error
- `TaskError`
  - Task execution error
- `CacheError`
  - Cache error
- `RateLimitExceeded`
  - API rate limit exceeded

---

#### `EventType`

Enumeration of DeFi event types supported across protocols

**Variants:**

- `Swap`
  - Token swap transaction
- `AddLiquidity`
  - Add liquidity to a pool
- `RemoveLiquidity`
  - Remove liquidity from a pool
- `LiquidityProvision`
  - Add liquidity to a pool (alternative name)
- `LiquidityRemoval`
  - Remove liquidity from a pool (alternative name)
- `Borrow`
  - Borrow funds from a lending protocol
- `Repay`
  - Repay borrowed funds
- `Liquidate`
  - Liquidate an undercollateralized position
- `Transfer`
  - Transfer tokens between accounts
- `Mint`
  - Mint new tokens
- `Burn`
  - Burn existing tokens
- `CreatePool`
  - Create a new liquidity pool
- `UpdatePool`
  - Update pool parameters
- `Transaction`
  - General transaction event
- `Block`
  - Block-level event
- `ContractEvent`
  - Smart contract execution event
- `PriceUpdate`
  - Price update event
- `OrderBook`
  - Order book update
- `Trade`
  - Trade execution
- `FeeUpdate`
  - Fee structure update
- `BonkBuyExactIn`
  - Bonk buy with exact input amount
- `BonkBuyExactOut`
  - Bonk buy with exact output amount
- `BonkSellExactIn`
  - Bonk sell with exact input amount
- `BonkSellExactOut`
  - Bonk sell with exact output amount
- `BonkInitialize`
  - Bonk pool initialization
- `BonkMigrateToAmm`
  - Bonk migration to AMM
- `BonkMigrateToCpswap`
  - Bonk migration to constant product swap
- `PumpSwapBuy`
  - PumpSwap buy transaction
- `PumpSwapSell`
  - PumpSwap sell transaction
- `PumpSwapCreatePool`
  - PumpSwap pool creation
- `PumpSwapDeposit`
  - PumpSwap deposit
- `PumpSwapWithdraw`
  - PumpSwap withdrawal
- `PumpSwapSetParams`
  - PumpSwap parameter update
- `RaydiumSwap`
  - Raydium swap transaction
- `RaydiumDeposit`
  - Raydium deposit
- `RaydiumWithdraw`
  - Raydium withdrawal
- `RaydiumAmmV4SwapBaseIn`
  - Raydium AMM V4 swap with base token input
- `RaydiumAmmV4SwapBaseOut`
  - Raydium AMM V4 swap with base token output
- `RaydiumAmmV4Deposit`
  - Raydium AMM V4 deposit
- `RaydiumAmmV4Initialize2`
  - Raydium AMM V4 second initialization
- `RaydiumAmmV4Withdraw`
  - Raydium AMM V4 withdrawal
- `RaydiumAmmV4WithdrawPnl`
  - Raydium AMM V4 profit and loss withdrawal
- `RaydiumClmmSwap`
  - Raydium CLMM swap
- `RaydiumClmmSwapV2`
  - Raydium CLMM swap version 2
- `RaydiumClmmCreatePool`
  - Raydium CLMM pool creation
- `RaydiumClmmOpenPositionV2`
  - Raydium CLMM open position version 2
- `RaydiumClmmIncreaseLiquidityV2`
  - Raydium CLMM increase liquidity version 2
- `RaydiumClmmDecreaseLiquidityV2`
  - Raydium CLMM decrease liquidity version 2
- `RaydiumClmmClosePosition`
  - Raydium CLMM close position
- `RaydiumClmmOpenPositionWithToken22Nft`
  - Raydium CLMM open position with Token22 NFT
- `RaydiumCpmmSwap`
  - Raydium CPMM swap
- `RaydiumCpmmSwapBaseInput`
  - Raydium CPMM swap with base input
- `RaydiumCpmmSwapBaseOutput`
  - Raydium CPMM swap with base output
- `RaydiumCpmmDeposit`
  - Raydium CPMM deposit
- `RaydiumCpmmWithdraw`
  - Raydium CPMM withdrawal
- `RaydiumCpmmCreatePool`
  - Raydium CPMM pool creation
- `OpenPosition`
  - Open a liquidity position
- `ClosePosition`
  - Close a liquidity position
- `IncreaseLiquidity`
  - Increase liquidity in position
- `DecreaseLiquidity`
  - Decrease liquidity in position
- `Deposit`
  - General deposit operation
- `Withdraw`
  - General withdrawal operation
- `Unknown`
  - Unknown or unclassified event type

---

#### `JupiterDiscriminator`

Jupiter instruction discriminators (using Anchor discriminators)

**Variants:**

- `Route`
  - Route discriminator: [229, 23, 203, 151, 122, 227, 173, 42]
- `RouteWithTokenLedger`
  - RouteWithTokenLedger discriminator: [206, 198, 71, 54, 47, 82, 194, 13]
- `ExactOutRoute`
  - ExactOutRoute discriminator: [123, 87, 17, 219, 42, 126, 197, 78]
- `SharedAccountsRoute`
  - SharedAccountsRoute discriminator: [95, 180, 10, 172, 84, 174, 232, 239]
- `SharedAccountsRouteWithTokenLedger`
  - SharedAccountsRouteWithTokenLedger discriminator: [18, 99, 149, 21, 45, 126, 144, 122]
- `SharedAccountsExactOutRoute`
  - SharedAccountsExactOutRoute discriminator: [77, 119, 2, 23, 198, 126, 79, 175]

---

#### `MarginFiAccountType`

MarginFi account types

**Variants:**

- `MarginfiGroup`
  - MarginFi group account containing global settings
- `MarginfiAccount`
  - Individual user account for lending positions
- `Bank`
  - Bank account representing a lending pool
- `Unknown`
  - Unknown or unrecognized account type

---

#### `MetaplexAuctionHouseDiscriminator`

Metaplex Auction House discriminators

**Variants:**

- `Buy`
  - Buy NFT from auction house
- `Sell`
  - Sell NFT on auction house
- `ExecuteSale`
  - Execute sale between buyer and seller
- `Deposit`
  - Deposit funds to auction house
- `Withdraw`
  - Withdraw funds from auction house
- `Cancel`
  - Cancel buy or sell order

---

#### `MetaplexTokenMetadataDiscriminator`

Metaplex instruction discriminators for Token Metadata program

**Variants:**

- `CreateMetadataAccount`
  - Create metadata account for NFT
- `UpdateMetadataAccount`
  - Update existing metadata account
- `DeprecatedCreateMasterEdition`
  - Deprecated create master edition instruction
- `DeprecatedMintNewEditionFromMasterEditionViaPrintingToken`
  - Deprecated mint new edition from master edition via printing token
- `UpdatePrimarySaleHappenedViaToken`
  - Update primary sale happened via token
- `DeprecatedSetReservationList`
  - Deprecated set reservation list instruction
- `DeprecatedCreateReservationList`
  - Deprecated create reservation list instruction
- `SignMetadata`
  - Sign metadata for creator verification
- `DeprecatedMintPrintingTokensViaToken`
  - Deprecated mint printing tokens via token
- `DeprecatedMintPrintingTokens`
  - Deprecated mint printing tokens instruction
- `CreateMasterEdition`
  - Create master edition for limited editions
- `MintNewEditionFromMasterEditionViaToken`
  - Mint new edition from master edition via token
- `ConvertMasterEditionV1ToV2`
  - Convert master edition V1 to V2
- `MintNewEditionFromMasterEditionViaVaultProxy`
  - Mint new edition from master edition via vault proxy
- `PuffMetadata`
  - Puff metadata to increase size
- `UpdateMetadataAccountV2`
  - Update metadata account version 2
- `CreateMetadataAccountV2`
  - Create metadata account version 2
- `CreateMasterEditionV3`
  - Create master edition version 3
- `VerifyCollection`
  - Verify collection membership
- `Utilize`
  - Utilize NFT for specific use case
- `ApproveUseAuthority`
  - Approve use authority for NFT
- `RevokeUseAuthority`
  - Revoke use authority for NFT
- `UnverifyCollection`
  - Unverify collection membership
- `ApproveCollectionAuthority`
  - Approve collection authority
- `RevokeCollectionAuthority`
  - Revoke collection authority
- `SetAndVerifyCollection`
  - Set and verify collection in one transaction
- `FreezeDelegatedAccount`
  - Freeze delegated account
- `ThawDelegatedAccount`
  - Thaw delegated account
- `RemoveCreatorVerification`
  - Remove creator verification
- `BurnNft`
  - Burn NFT permanently
- `VerifyCreator`
  - Verify creator signature
- `UnverifyCreator`
  - Unverify creator signature
- `BubblegumSetCollectionSize`
  - Bubblegum set collection size
- `BurnEditionNft`
  - Burn edition NFT
- `CreateMetadataAccountV3`
  - Create metadata account version 3
- `SetCollectionSize`
  - Set collection size
- `SetTokenStandard`
  - Set token standard
- `BubblegumVerifyCreator`
  - Bubblegum verify creator
- `BubblegumUnverifyCreator`
  - Bubblegum unverify creator
- `BubblegumVerifyCollection`
  - Bubblegum verify collection
- `BubblegumUnverifyCollection`
  - Bubblegum unverify collection
- `BubblegumSetAndVerifyCollection`
  - Bubblegum set and verify collection
- `Transfer`
  - Transfer NFT ownership

---

#### `ParseError`

Error type for parsing operations

**Variants:**

- `InvalidInstructionData`
  - Invalid instruction data encountered during parsing
- `InsufficientData`
  - Insufficient data available for parsing operation
- `UnknownDiscriminator`
  - Unknown discriminator value encountered
- `DeserializationError`
  - Borsh deserialization error
- `MemoryMapError`
  - Memory mapping operation error

---

#### `PipelineError`

Error type for parsing pipeline operations

**Variants:**

- `SemaphoreError`
  - Error acquiring semaphore permit
- `ChannelError`
  - Error sending data through channel
- `ParseError`
  - Error during event parsing
- `ConfigError`
  - Invalid pipeline configuration

---

#### `PoolStatus`

Current status of a liquidity pool in the BONK protocol

**Variants:**

- `Fund`
  - Initial funding phase where liquidity is being raised
- `Migrate`
  - Migration phase where pool is transitioning to DEX
- `Trade`
  - Active trading phase where tokens can be traded

---

#### `Protocol`

Protocol enum for supported protocols

**Variants:**

- `OrcaWhirlpool`
  - Orca Whirlpool concentrated liquidity protocol
- `MeteoraDlmm`
  - Meteora Dynamic Liquidity Market Maker protocol
- `MarginFi`
  - MarginFi lending and borrowing protocol
- `Jupiter`
  - Jupiter swap aggregator protocol
- `RaydiumAmmV4`
  - Raydium Automated Market Maker V4 protocol
- `RaydiumClmm`
  - Raydium Concentrated Liquidity Market Maker protocol
- `RaydiumCpmm`
  - Raydium Constant Product Market Maker protocol
- `PumpFun`
  - PumpFun meme token creation protocol
- `PumpSwap`
  - PumpSwap trading protocol
- `Bonk`
  - Bonk token protocol
- `Custom`
  - Custom protocol with arbitrary name

---

#### `ProtocolType`

Enumeration of supported Solana DeFi protocols

**Variants:**

- `OrcaWhirlpool`
  - Orca Whirlpool concentrated liquidity pools
- `MeteoraDlmm`
  - Meteora Dynamic Liquidity Market Maker
- `MarginFi`
  - MarginFi lending protocol
- `Bonk`
  - Bonk decentralized exchange
- `PumpSwap`
  - PumpSwap automated market maker
- `RaydiumAmm`
  - Raydium automated market maker
- `RaydiumAmmV4`
  - Raydium AMM V4 implementation
- `RaydiumClmm`
  - Raydium concentrated liquidity market maker
- `RaydiumCpmm`
  - Raydium constant product market maker
- `Raydium`
  - General Raydium protocol
- `Jupiter`
  - Jupiter aggregator protocol
- `Serum`
  - Serum decentralized exchange
- `Other`
  - Other protocol not explicitly supported

---

#### `PumpFunDiscriminator`

PumpFun instruction discriminators

**Variants:**

- `Buy`
  - Buy tokens instruction
- `Sell`
  - Sell tokens instruction
- `CreatePool`
  - Create new pool instruction
- `Deposit`
  - Deposit liquidity instruction
- `Withdraw`
  - Withdraw liquidity instruction
- `SetParams`
  - Set pool parameters instruction

---

#### `RaydiumV4Discriminator`

Raydium AMM V4 instruction discriminators

**Variants:**

- `SwapBaseIn`
  - Swap with base input amount (discriminator 0x09)
- `SwapBaseOut`
  - Swap with base output amount (discriminator 0x0a)
- `Deposit`
  - Deposit liquidity to pool (discriminator 0x03)
- `Withdraw`
  - Withdraw liquidity from pool (discriminator 0x04)
- `Initialize2`
  - Initialize pool version 2 (discriminator 0x00)
- `WithdrawPnl`
  - Withdraw profit and loss (discriminator 0x05)

---

#### `SolanaTransactionInput`

Unified owned input type for Solana transaction parsing

**Variants:**

- `Instruction`
  - Regular instruction parsing input
- `InnerInstruction`
  - Inner instruction parsing input

---

#### `SwapDirection`

Orca swap direction

**Variants:**

- `AtoB`
  - Swap from token A to token B
- `BtoA`
  - Swap from token B to token A

---

#### `TradeDirection`

Direction of a trade operation in the BONK protocol

**Variants:**

- `Buy`
  - Buy operation - purchasing tokens
- `Sell`
  - Sell operation - selling tokens

---

#### `ValidationError`

Validation error types

**Variants:**

- `MissingField`
  - Missing required field
- `InvalidValue`
  - Invalid field value
- `Inconsistency`
  - Data inconsistency
- `BusinessLogicError`
  - Business logic violation
- `StaleEvent`
  - Event too old
- `Duplicate`
  - Duplicate event detected

---

#### `ValidationWarning`

Validation warning types

**Variants:**

- `UnusualValue`
  - Unusual but potentially valid value
- `DeprecatedField`
  - Deprecated field usage
- `PerformanceWarning`
  - Performance concern

---

### Traits

> Trait definitions for implementing common behaviors.

#### `ByteSliceEventParser`

Trait for zero-copy byte slice parsing

**Methods:**

- `parse_from_slice()`
  - Parse events from a byte slice without copying data
- `can_parse()`
  - Check if this parser can handle the given data
- `protocol_type()`
  - Get the protocol type this parser handles

---

#### `ProtocolParser`

Protocol-specific parser trait for Solana events
This trait is used internally by protocol parsers and doesn't conflict with
the core EventParser trait from riglr_events_core.

**Methods:**

- `inner_instruction_configs()`
  - Get inner instruction parsing configurations
- `instruction_configs()`
  - Get instruction parsing configurations
- `parse_events_from_inner_instruction()`
  - Parse event data from inner instruction
- `parse_events_from_instruction()`
  - Parse event data from instruction
- `should_handle()`
  - Check if this program ID should be handled
- `supported_program_ids()`
  - Get supported program ID list

---

#### `ToSolanaEvent`

Helper trait for converting legacy events to Solana events

**Methods:**

- `to_solana_event()`
  - Convert this event to a SolanaEvent for unified handling

---

#### `ValidationRule`

Trait for validation rules

**Methods:**

- `validate()`
  - Validate an event
- `name()`
  - Get rule name

---

### Functions

> Standalone functions and utilities.

#### `amount_to_shares`

Convert amount to shares using share value

---

#### `bin_id_to_price`

Convert bin ID to price for DLMM

---

#### `calculate_active_bin_price`

Calculate active bin price

---

#### `calculate_health_ratio`

Calculate health ratio for a MarginFi account

---

#### `calculate_interest_rate`

Calculate interest rate based on utilization

---

#### `calculate_liquidation_threshold`

Calculate liquidation threshold

---

#### `calculate_liquidity_distribution`

Calculate liquidity distribution across bins

---

#### `calculate_price_impact`

Calculate price impact for a swap

---

#### `create_core_metadata`

Create core EventMetadata without Solana-specific chain data

---

#### `create_metadata`

Create a SolanaEventMetadata from components

---

#### `create_solana_metadata`

Create a SolanaEventMetadata without duplication in chain_data

---

#### `decode_base58`

Decode base58 string to bytes

---

#### `discriminator_matches`

Checks if a hex data string starts with a given hex discriminator

Both `data` and `discriminator` must be valid hex strings starting with "0x".
Returns `true` if the data string begins with the discriminator string.

# Arguments

* `data` - The hex data string to check
* `discriminator` - The hex discriminator to match against

# Returns

`true` if the data starts with the discriminator, `false` otherwise

---

#### `encode_base58`

Encode bytes to base58 string

---

#### `extract_account_keys`

Extract account keys from accounts array with proper error handling

---

#### `extract_discriminator`

Extract discriminator from instruction data

---

#### `extract_required_accounts`

Extract multiple required accounts in one call

---

#### `format_token_amount`

Convert amount with decimals to human-readable format

---

#### `get_account_or_default`

Extract account key with optional fallback to default

---

#### `get_block_time`

Get block_time from Solana EventMetadata

---

#### `get_event_type`

Get event_type from Solana EventMetadata

---

#### `get_instruction_index`

Get instruction_index from Solana EventMetadata

---

#### `get_program_id`

Get program_id from Solana EventMetadata

---

#### `get_protocol_type`

Get protocol_type from Solana EventMetadata

---

#### `get_signature`

Get signature from Solana EventMetadata

---

#### `get_slot`

Get slot from Solana EventMetadata

---

#### `has_discriminator`

Check if instruction data starts with a specific discriminator

---

#### `is_jupiter_v6_program`

Check if the given pubkey is Jupiter V6 program

---

#### `is_marginfi_program`

Check if the given pubkey is MarginFi program

---

#### `is_meteora_dlmm_program`

Check if the given pubkey is Meteora DLMM program

---

#### `is_meteora_dynamic_program`

Check if the given pubkey is Meteora Dynamic program

---

#### `is_orca_whirlpool_program`

Check if the given pubkey is Orca Whirlpool program

---

#### `jupiter_v6_program_id`

Extract Jupiter program ID as Pubkey

---

#### `marginfi_bank_program_id`

Extract MarginFi bank program ID as Pubkey

---

#### `marginfi_program_id`

Extract MarginFi program ID as Pubkey

---

#### `meteora_dlmm_program_id`

Extract Meteora DLMM program ID as Pubkey

---

#### `meteora_dynamic_program_id`

Extract Meteora Dynamic program ID as Pubkey

---

#### `orca_whirlpool_program_id`

Extract Orca Whirlpool program ID as Pubkey

---

#### `parse_liquidity_amounts`

Parse liquidity amounts (common pattern for deposit/withdraw operations)

---

#### `parse_pubkey_from_bytes`

Parse a pubkey from bytes

---

#### `parse_swap_amounts`

Parse swap amounts (common pattern for buy/sell operations)

---

#### `parse_u128_le`

Parse a u128 from little-endian bytes

---

#### `parse_u16_le`

Parse a u16 from little-endian bytes

---

#### `parse_u32_le`

Parse a u32 from little-endian bytes

---

#### `parse_u64_le`

Parse a u64 from little-endian bytes

---

#### `read_bool`

Parse boolean from single byte with proper error handling

---

#### `read_i32_le`

Read i32 from little-endian bytes at offset

---

#### `read_option_bool`

Read optional bool from bytes at offset

---

#### `read_pubkey`

Parse Pubkey from 32 bytes with proper error handling

---

#### `read_u128_le`

Read u128 from little-endian bytes at offset

---

#### `read_u16_le`

Parse u16 from little-endian bytes with proper error handling

---

#### `read_u32_le`

Read u32 from little-endian bytes at offset

---

#### `read_u64_le`

Common utility functions for event parsing

Read u64 from little-endian bytes at offset

---

#### `read_u8_le`

Read u8 from bytes at offset

---

#### `safe_get_account`

Safely extract a single account key by index

---

#### `set_event_type`

Set event_type in Solana EventMetadata

---

#### `set_protocol_type`

Set protocol_type in Solana EventMetadata

---

#### `shares_to_amount`

Convert shares to amount using share value

---

#### `sqrt_price_to_price`

Convert sqrt price to price

---

#### `system_time_to_millis`

Convert SystemTime to milliseconds since epoch

---

#### `tick_index_to_price`

Convert tick index to price

---

#### `to_token_amount`

Convert human-readable amount to token amount with decimals

---

#### `validate_account_count`

Validate minimum account count with descriptive error message

---

#### `validate_data_length`

Validate minimum data length with descriptive error message

---

### Type Aliases

#### `InnerInstructionEventParser`

Inner instruction event parser

**Type:** `_`

---

#### `InstructionEventParser`

Instruction event parser

**Type:** `_`

---

#### `ParseResult`

Result type for parsing operations

**Type:** `<T, >`

---

### Constants

#### `ANCHOR_DISCRIMINATOR_SIZE`

Anchor discriminator size (8 bytes)

**Type:** `usize`

---

#### `BONK_PROGRAM_ID`

Bonk program ID

**Type:** ``

---

#### `BUY_EVENT`

String identifier for PumpSwap buy events

**Type:** `&str`

---

#### `BUY_EVENT_BYTES`

Byte array discriminator for PumpSwap buy events

**Type:** `&[u8]`

---

#### `BUY_EXACT_IN_IX`

Instruction discriminator for buy exact in operations

**Type:** `&[u8]`

---

#### `BUY_EXACT_OUT_IX`

Instruction discriminator for buy exact out operations

**Type:** `&[u8]`

---

#### `BUY_IX`

Instruction discriminator for PumpSwap buy operations

**Type:** `&[u8]`

---

#### `CLOSE_POSITION`

Instruction discriminator for closing liquidity positions

**Type:** `&[u8]`

---

#### `CLOSE_POSITION_DISCRIMINATOR`

Instruction discriminator for closing liquidity positions

**Type:** `[u8; 8]`

---

#### `CREATE_POOL`

Instruction discriminator for creating new pools

**Type:** `&[u8]`

---

#### `CREATE_POOL_EVENT`

String identifier for PumpSwap create pool events

**Type:** `&str`

---

#### `CREATE_POOL_EVENT_BYTES`

Byte array discriminator for PumpSwap create pool events

**Type:** `&[u8]`

---

#### `CREATE_POOL_IX`

Instruction discriminator for PumpSwap create pool operations

**Type:** `&[u8]`

---

#### `DECREASE_LIQUIDITY_DISCRIMINATOR`

Instruction discriminator for decreasing liquidity in positions

**Type:** `[u8; 8]`

---

#### `DECREASE_LIQUIDITY_V2`

Instruction discriminator for decreasing liquidity in positions (version 2)

**Type:** `&[u8]`

---

#### `DEPOSIT`

Instruction discriminator for depositing liquidity into the pool

**Type:** `&[u8]`

---

#### `DEPOSIT_EVENT`

String identifier for deposit events

**Type:** `&str`

---

#### `DEPOSIT_EVENT_BYTES`

Byte array discriminator for deposit events

**Type:** `&[u8]`

---

#### `DEPOSIT_IX`

Instruction discriminator for PumpSwap deposit operations

**Type:** `&[u8]`

---

#### `DLMM_ADD_LIQUIDITY_DISCRIMINATOR`

Discriminator for DLMM add liquidity instruction

**Type:** `[u8; 8]`

---

#### `DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR`

Discriminator for DLMM remove liquidity instruction

**Type:** `[u8; 8]`

---

#### `DLMM_SWAP_DISCRIMINATOR`

Meteora instruction discriminators
Discriminator for DLMM swap instruction

**Type:** `[u8; 8]`

---

#### `DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR`

Discriminator for Dynamic AMM add liquidity instruction

**Type:** `[u8; 8]`

---

#### `DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR`

Discriminator for Dynamic AMM remove liquidity instruction

**Type:** `[u8; 8]`

---

#### `EXACT_OUT_ROUTE_DISCRIMINATOR`

Discriminator for shared accounts exact out route instruction

**Type:** `[u8; 8]`

---

#### `INCREASE_LIQUIDITY_DISCRIMINATOR`

Instruction discriminator for increasing liquidity in positions

**Type:** `[u8; 8]`

---

#### `INCREASE_LIQUIDITY_V2`

Instruction discriminator for increasing liquidity in positions (version 2)

**Type:** `&[u8]`

---

#### `INITIALIZE2`

Instruction discriminator for initializing the AMM pool (version 2)

**Type:** `&[u8]`

---

#### `INITIALIZE_IX`

Instruction discriminator for pool initialization

**Type:** `&[u8]`

---

#### `INSTRUCTION_DATA_OFFSET`

Instruction data offset (after discriminator)

**Type:** `usize`

---

#### `JUPITER_MIN_INSTRUCTION_SIZE`

Jupiter minimum instruction size (discriminator + pubkeys + amounts)

**Type:** `usize`

---

#### `JUPITER_PROGRAM_ID`

Jupiter aggregator program ID

**Type:** `&str`

---

#### `JUPITER_V6_PROGRAM_ID`

Jupiter V6 program ID

**Type:** `&str`

---

#### `LEGACY_EXACT_OUT_DISCRIMINATOR`

Discriminator for legacy exact out route instruction

**Type:** `[u8; 8]`

---

#### `LEGACY_ROUTE_DISCRIMINATOR`

Discriminator for legacy route instruction

**Type:** `[u8; 8]`

---

#### `MARGINFI_BANK_PROGRAM_ID`

MarginFi Bank program ID (for lending pools)

**Type:** `&str`

---

#### `MARGINFI_BORROW_DISCRIMINATOR`

Discriminator for lending borrow instruction

**Type:** `[u8; 8]`

---

#### `MARGINFI_DEPOSIT_DISCRIMINATOR`

MarginFi instruction discriminators
Discriminator for lending deposit instruction

**Type:** `[u8; 8]`

---

#### `MARGINFI_LIQUIDATE_DISCRIMINATOR`

Discriminator for liquidation instruction

**Type:** `[u8; 8]`

---

#### `MARGINFI_PROGRAM_ID`

MarginFi program ID

**Type:** `&str`

---

#### `MARGINFI_REPAY_DISCRIMINATOR`

Discriminator for lending repay instruction

**Type:** `[u8; 8]`

---

#### `MARGINFI_WITHDRAW_DISCRIMINATOR`

Discriminator for lending withdraw instruction

**Type:** `[u8; 8]`

---

#### `METAPLEX_AUCTION_HOUSE_PROGRAM_ID`

Metaplex Auction House program ID

**Type:** `&str`

---

#### `METAPLEX_TOKEN_METADATA_PROGRAM_ID`

Metaplex Token Metadata program ID

**Type:** `&str`

---

#### `METEORA_DLMM_PROGRAM_ID`

Meteora DLMM program ID

**Type:** `&str`

---

#### `METEORA_DYNAMIC_PROGRAM_ID`

Meteora Dynamic program ID

**Type:** `&str`

---

#### `MIGRATE_TO_AMM_IX`

Instruction discriminator for migrating to AMM

**Type:** `&[u8]`

---

#### `MIGRATE_TO_CPSWAP_IX`

Instruction discriminator for migrating to constant product swap

**Type:** `&[u8]`

---

#### `MIN_COMPLEX_ACCOUNTS`

Minimum accounts for complex operations (AMM v4, etc.)

**Type:** `usize`

---

#### `MIN_LIQUIDITY_ACCOUNTS`

Minimum accounts for liquidity operations

**Type:** `usize`

---

#### `MIN_LIQUIDITY_INSTRUCTION_SIZE`

Minimum size for liquidity instructions (discriminator + 3 u64s)

**Type:** `usize`

---

#### `MIN_POSITION_ACCOUNTS`

Minimum accounts for position operations

**Type:** `usize`

---

#### `MIN_POSITION_INSTRUCTION_SIZE`

Minimum size for position instructions (discriminator + various fields)

**Type:** `usize`

---

#### `MIN_SWAP_ACCOUNTS`

Minimum accounts for basic swap operations

**Type:** `usize`

---

#### `MIN_SWAP_INSTRUCTION_SIZE`

Minimum size for swap instructions (discriminator + 2 u64s)

**Type:** `usize`

---

#### `OPEN_POSITION_DISCRIMINATOR`

Instruction discriminator for opening liquidity positions

**Type:** `[u8; 8]`

---

#### `OPEN_POSITION_V2`

Instruction discriminator for opening liquidity positions (version 2)

**Type:** `&[u8]`

---

#### `OPEN_POSITION_WITH_TOKEN_22_NFT`

Instruction discriminator for opening positions with Token-22 NFT

**Type:** `&[u8]`

---

#### `ORCA_WHIRLPOOL_PROGRAM_ID`

Orca Whirlpool program ID

**Type:** `&str`

---

#### `POOL_CREATE_EVENT`

String identifier for Bonk pool creation events

**Type:** `&str`

---

#### `POOL_CREATE_EVENT_BYTES`

Byte array discriminator for pool creation events, used for efficient on-chain parsing

**Type:** `&[u8]`

---

#### `PUBKEY_RANGE`

Byte range for parsing a single public key value

**Type:** `<usize>`

---

#### `PUBKEY_SIZE`

Size of a Solana public key in bytes

**Type:** `usize`

---

#### `PUMPSWAP_PROGRAM_ID`

PumpSwap program ID

**Type:** ``

---

#### `PUMP_FUN_PROGRAM_ID`

PumpFun program ID

**Type:** `&str`

---

#### `RAYDIUM_AMM_V4_PROGRAM_ID`

Raydium AMM V4 program ID

**Type:** ``

---

#### `RAYDIUM_CLMM_PROGRAM_ID`

Raydium CLMM program ID

**Type:** ``

---

#### `RAYDIUM_CPMM_PROGRAM_ID`

Raydium CPMM program ID

**Type:** ``

---

#### `ROUTE_DISCRIMINATOR`

Jupiter swap discriminators (calculated from Anchor's "global:<instruction_name>")
sharedAccountsRoute is the most common Jupiter V6 swap instruction

**Type:** `[u8; 8]`

---

#### `ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR`

Discriminator for route with token ledger instruction

**Type:** `[u8; 8]`

---

#### `SECOND_U64_RANGE`

Second u64 range (for parsing two consecutive u64 values)

**Type:** `<usize>`

---

#### `SELL_EVENT`

String identifier for PumpSwap sell events

**Type:** `&str`

---

#### `SELL_EVENT_BYTES`

Byte array discriminator for PumpSwap sell events

**Type:** `&[u8]`

---

#### `SELL_EXACT_IN_IX`

Instruction discriminator for sell exact in operations

**Type:** `&[u8]`

---

#### `SELL_EXACT_OUT_IX`

Instruction discriminator for sell exact out operations

**Type:** `&[u8]`

---

#### `SELL_IX`

Instruction discriminator for PumpSwap sell operations

**Type:** `&[u8]`

---

#### `SWAP`

Instruction discriminator for swap operations

**Type:** `&[u8]`

---

#### `SWAP_BASE_IN`

Instruction discriminator for swapping with base token as input

**Type:** `&[u8]`

---

#### `SWAP_BASE_INPUT_IX`

Instruction discriminator for swap with base input

**Type:** `&[u8]`

---

#### `SWAP_BASE_OUT`

Instruction discriminator for swapping with base token as output

**Type:** `&[u8]`

---

#### `SWAP_BASE_OUTPUT_IX`

Instruction discriminator for swap with base output

**Type:** `&[u8]`

---

#### `SWAP_DISCRIMINATOR`

Discriminator for swap instruction

**Type:** `[u8; 8]`

---

#### `SWAP_EVENT`

String identifier for swap events

**Type:** `&str`

---

#### `SWAP_EVENT_BYTES`

Byte array discriminator for swap events

**Type:** `&[u8]`

---

#### `SWAP_V2`

Instruction discriminator for swap operations (version 2)

**Type:** `&[u8]`

---

#### `THIRD_U64_RANGE`

Third u64 range (for parsing three consecutive u64 values)

**Type:** `<usize>`

---

#### `TRADE_EVENT`

String identifier for Bonk trade events

**Type:** `&str`

---

#### `TRADE_EVENT_BYTES`

Byte array discriminator for trade events, used for efficient on-chain parsing

**Type:** `&[u8]`

---

#### `U128_RANGE`

Byte range for parsing a single u128 value

**Type:** `<usize>`

---

#### `U128_SIZE`

Size of a u128 in bytes

**Type:** `usize`

---

#### `U16_RANGE`

Byte range for parsing a single u16 value

**Type:** `<usize>`

---

#### `U16_SIZE`

Size of a u16 in bytes

**Type:** `usize`

---

#### `U32_RANGE`

Byte range for parsing a single u32 value

**Type:** `<usize>`

---

#### `U32_SIZE`

Size of a u32 in bytes

**Type:** `usize`

---

#### `U64_RANGE`

Byte range for parsing a single u64 value

**Type:** `<usize>`

---

#### `U64_SIZE`

Size of a u64 in bytes

**Type:** `usize`

---

#### `U8_RANGE`

Common byte ranges for parsing

**Type:** `<usize>`

---

#### `U8_SIZE`

Size constants for basic data types

**Type:** `usize`

---

#### `WITHDRAW`

Instruction discriminator for withdrawing liquidity from the pool

**Type:** `&[u8]`

---

#### `WITHDRAW_EVENT`

String identifier for PumpSwap withdraw events

**Type:** `&str`

---

#### `WITHDRAW_EVENT_BYTES`

Byte array discriminator for PumpSwap withdraw events

**Type:** `&[u8]`

---

#### `WITHDRAW_IX`

Instruction discriminator for withdraw operations

**Type:** `&[u8]`

---

#### `WITHDRAW_PNL`

Instruction discriminator for withdrawing profit and loss from the pool

**Type:** `&[u8]`

---
