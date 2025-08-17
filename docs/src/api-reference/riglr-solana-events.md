# riglr-solana-events API Reference

Comprehensive API documentation for the `riglr-solana-events` crate.

## Table of Contents

### Constants

- [`ANCHOR_DISCRIMINATOR_SIZE`](#anchor_discriminator_size)
- [`BONK_PROGRAM_ID`](#bonk_program_id)
- [`BUY_EVENT`](#buy_event)
- [`BUY_EVENT_BYTES`](#buy_event_bytes)
- [`BUY_EXACT_IN_IX`](#buy_exact_in_ix)
- [`BUY_EXACT_OUT_IX`](#buy_exact_out_ix)
- [`BUY_IX`](#buy_ix)
- [`CLOSE_POSITION`](#close_position)
- [`CLOSE_POSITION_DISCRIMINATOR`](#close_position_discriminator)
- [`CREATE_POOL`](#create_pool)
- [`CREATE_POOL_EVENT`](#create_pool_event)
- [`CREATE_POOL_EVENT_BYTES`](#create_pool_event_bytes)
- [`CREATE_POOL_IX`](#create_pool_ix)
- [`DECREASE_LIQUIDITY_DISCRIMINATOR`](#decrease_liquidity_discriminator)
- [`DECREASE_LIQUIDITY_V2`](#decrease_liquidity_v2)
- [`DEPOSIT`](#deposit)
- [`DEPOSIT_EVENT`](#deposit_event)
- [`DEPOSIT_EVENT`](#deposit_event)
- [`DEPOSIT_EVENT_BYTES`](#deposit_event_bytes)
- [`DEPOSIT_EVENT_BYTES`](#deposit_event_bytes)
- [`DEPOSIT_IX`](#deposit_ix)
- [`DEPOSIT_IX`](#deposit_ix)
- [`DLMM_ADD_LIQUIDITY_DISCRIMINATOR`](#dlmm_add_liquidity_discriminator)
- [`DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR`](#dlmm_remove_liquidity_discriminator)
- [`DLMM_SWAP_DISCRIMINATOR`](#dlmm_swap_discriminator)
- [`DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR`](#dynamic_add_liquidity_discriminator)
- [`DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR`](#dynamic_remove_liquidity_discriminator)
- [`EXACT_OUT_ROUTE_DISCRIMINATOR`](#exact_out_route_discriminator)
- [`INCREASE_LIQUIDITY_DISCRIMINATOR`](#increase_liquidity_discriminator)
- [`INCREASE_LIQUIDITY_V2`](#increase_liquidity_v2)
- [`INITIALIZE2`](#initialize2)
- [`INITIALIZE_IX`](#initialize_ix)
- [`INITIALIZE_IX`](#initialize_ix)
- [`INSTRUCTION_DATA_OFFSET`](#instruction_data_offset)
- [`JUPITER_MIN_INSTRUCTION_SIZE`](#jupiter_min_instruction_size)
- [`JUPITER_PROGRAM_ID`](#jupiter_program_id)
- [`JUPITER_V6_PROGRAM_ID`](#jupiter_v6_program_id)
- [`LEGACY_EXACT_OUT_DISCRIMINATOR`](#legacy_exact_out_discriminator)
- [`LEGACY_ROUTE_DISCRIMINATOR`](#legacy_route_discriminator)
- [`MARGINFI_BANK_PROGRAM_ID`](#marginfi_bank_program_id)
- [`MARGINFI_BORROW_DISCRIMINATOR`](#marginfi_borrow_discriminator)
- [`MARGINFI_DEPOSIT_DISCRIMINATOR`](#marginfi_deposit_discriminator)
- [`MARGINFI_LIQUIDATE_DISCRIMINATOR`](#marginfi_liquidate_discriminator)
- [`MARGINFI_PROGRAM_ID`](#marginfi_program_id)
- [`MARGINFI_REPAY_DISCRIMINATOR`](#marginfi_repay_discriminator)
- [`MARGINFI_WITHDRAW_DISCRIMINATOR`](#marginfi_withdraw_discriminator)
- [`METAPLEX_AUCTION_HOUSE_PROGRAM_ID`](#metaplex_auction_house_program_id)
- [`METAPLEX_TOKEN_METADATA_PROGRAM_ID`](#metaplex_token_metadata_program_id)
- [`METEORA_DLMM_PROGRAM_ID`](#meteora_dlmm_program_id)
- [`METEORA_DYNAMIC_PROGRAM_ID`](#meteora_dynamic_program_id)
- [`MIGRATE_TO_AMM_IX`](#migrate_to_amm_ix)
- [`MIGRATE_TO_CPSWAP_IX`](#migrate_to_cpswap_ix)
- [`MIN_COMPLEX_ACCOUNTS`](#min_complex_accounts)
- [`MIN_LIQUIDITY_ACCOUNTS`](#min_liquidity_accounts)
- [`MIN_LIQUIDITY_INSTRUCTION_SIZE`](#min_liquidity_instruction_size)
- [`MIN_POSITION_ACCOUNTS`](#min_position_accounts)
- [`MIN_POSITION_INSTRUCTION_SIZE`](#min_position_instruction_size)
- [`MIN_SWAP_ACCOUNTS`](#min_swap_accounts)
- [`MIN_SWAP_INSTRUCTION_SIZE`](#min_swap_instruction_size)
- [`OPEN_POSITION_DISCRIMINATOR`](#open_position_discriminator)
- [`OPEN_POSITION_V2`](#open_position_v2)
- [`OPEN_POSITION_WITH_TOKEN_22_NFT`](#open_position_with_token_22_nft)
- [`ORCA_WHIRLPOOL_PROGRAM_ID`](#orca_whirlpool_program_id)
- [`POOL_CREATE_EVENT`](#pool_create_event)
- [`POOL_CREATE_EVENT_BYTES`](#pool_create_event_bytes)
- [`PUBKEY_RANGE`](#pubkey_range)
- [`PUBKEY_SIZE`](#pubkey_size)
- [`PUMPSWAP_PROGRAM_ID`](#pumpswap_program_id)
- [`PUMP_FUN_PROGRAM_ID`](#pump_fun_program_id)
- [`RAYDIUM_AMM_V4_PROGRAM_ID`](#raydium_amm_v4_program_id)
- [`RAYDIUM_AMM_V4_PROGRAM_ID`](#raydium_amm_v4_program_id)
- [`RAYDIUM_CLMM_PROGRAM_ID`](#raydium_clmm_program_id)
- [`RAYDIUM_CPMM_PROGRAM_ID`](#raydium_cpmm_program_id)
- [`ROUTE_DISCRIMINATOR`](#route_discriminator)
- [`ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR`](#route_with_token_ledger_discriminator)
- [`SECOND_U64_RANGE`](#second_u64_range)
- [`SELL_EVENT`](#sell_event)
- [`SELL_EVENT_BYTES`](#sell_event_bytes)
- [`SELL_EXACT_IN_IX`](#sell_exact_in_ix)
- [`SELL_EXACT_OUT_IX`](#sell_exact_out_ix)
- [`SELL_IX`](#sell_ix)
- [`SWAP`](#swap)
- [`SWAP_BASE_IN`](#swap_base_in)
- [`SWAP_BASE_INPUT_IX`](#swap_base_input_ix)
- [`SWAP_BASE_OUT`](#swap_base_out)
- [`SWAP_BASE_OUTPUT_IX`](#swap_base_output_ix)
- [`SWAP_DISCRIMINATOR`](#swap_discriminator)
- [`SWAP_DISCRIMINATOR`](#swap_discriminator)
- [`SWAP_EVENT`](#swap_event)
- [`SWAP_EVENT_BYTES`](#swap_event_bytes)
- [`SWAP_V2`](#swap_v2)
- [`THIRD_U64_RANGE`](#third_u64_range)
- [`TRADE_EVENT`](#trade_event)
- [`TRADE_EVENT_BYTES`](#trade_event_bytes)
- [`U128_RANGE`](#u128_range)
- [`U128_SIZE`](#u128_size)
- [`U16_RANGE`](#u16_range)
- [`U16_SIZE`](#u16_size)
- [`U32_RANGE`](#u32_range)
- [`U32_SIZE`](#u32_size)
- [`U64_RANGE`](#u64_range)
- [`U64_SIZE`](#u64_size)
- [`U8_RANGE`](#u8_range)
- [`U8_SIZE`](#u8_size)
- [`WITHDRAW`](#withdraw)
- [`WITHDRAW_EVENT`](#withdraw_event)
- [`WITHDRAW_EVENT_BYTES`](#withdraw_event_bytes)
- [`WITHDRAW_IX`](#withdraw_ix)
- [`WITHDRAW_IX`](#withdraw_ix)
- [`WITHDRAW_PNL`](#withdraw_pnl)

### Enums

- [`EnrichmentError`](#enrichmenterror)
- [`EventType`](#eventtype)
- [`JupiterDiscriminator`](#jupiterdiscriminator)
- [`MarginFiAccountType`](#marginfiaccounttype)
- [`MetaplexAuctionHouseDiscriminator`](#metaplexauctionhousediscriminator)
- [`MetaplexTokenMetadataDiscriminator`](#metaplextokenmetadatadiscriminator)
- [`ParseError`](#parseerror)
- [`ParseError`](#parseerror)
- [`PipelineError`](#pipelineerror)
- [`PoolStatus`](#poolstatus)
- [`Protocol`](#protocol)
- [`ProtocolType`](#protocoltype)
- [`PumpFunDiscriminator`](#pumpfundiscriminator)
- [`RaydiumV4Discriminator`](#raydiumv4discriminator)
- [`SwapDirection`](#swapdirection)
- [`SwapDirection`](#swapdirection)
- [`TradeDirection`](#tradedirection)
- [`ValidationError`](#validationerror)
- [`ValidationWarning`](#validationwarning)

### Functions (error)

- [`invalid_account_index`](#invalid_account_index)
- [`invalid_discriminator`](#invalid_discriminator)
- [`invalid_enum_variant`](#invalid_enum_variant)
- [`not_enough_bytes`](#not_enough_bytes)

### Functions (utils)

- [`calculate_price_impact`](#calculate_price_impact)
- [`decode_base58`](#decode_base58)
- [`discriminator_matches`](#discriminator_matches)
- [`encode_base58`](#encode_base58)
- [`extract_account_keys`](#extract_account_keys)
- [`extract_discriminator`](#extract_discriminator)
- [`format_token_amount`](#format_token_amount)
- [`has_discriminator`](#has_discriminator)
- [`parse_pubkey_from_bytes`](#parse_pubkey_from_bytes)
- [`parse_u128_le`](#parse_u128_le)
- [`parse_u16_le`](#parse_u16_le)
- [`parse_u32_le`](#parse_u32_le)
- [`parse_u64_le`](#parse_u64_le)
- [`read_i32_le`](#read_i32_le)
- [`read_option_bool`](#read_option_bool)
- [`read_u128_le`](#read_u128_le)
- [`read_u32_le`](#read_u32_le)
- [`read_u64_le`](#read_u64_le)
- [`read_u8_le`](#read_u8_le)
- [`system_time_to_millis`](#system_time_to_millis)
- [`to_token_amount`](#to_token_amount)

### Structs

- [`AccountMeta`](#accountmeta)
- [`BatchEventParser`](#batcheventparser)
- [`BatchParserStats`](#batchparserstats)
- [`BonkEventParser`](#bonkeventparser)
- [`BonkPoolCreateEvent`](#bonkpoolcreateevent)
- [`BonkTradeEvent`](#bonktradeevent)
- [`BurnNftInstruction`](#burnnftinstruction)
- [`CacheStats`](#cachestats)
- [`ConstantCurve`](#constantcurve)
- [`CreateMetadataAccountInstruction`](#createmetadataaccountinstruction)
- [`CurveParams`](#curveparams)
- [`CustomDeserializer`](#customdeserializer)
- [`DepositInstruction`](#depositinstruction)
- [`DlmmBin`](#dlmmbin)
- [`DlmmPairConfig`](#dlmmpairconfig)
- [`EnrichmentConfig`](#enrichmentconfig)
- [`EventEnricher`](#eventenricher)
- [`EventParameters`](#eventparameters)
- [`EventParameters`](#eventparameters)
- [`EventParameters`](#eventparameters)
- [`EventParameters`](#eventparameters)
- [`EventParserRegistry`](#eventparserregistry)
- [`FixedCurve`](#fixedcurve)
- [`GenericEventParseConfig`](#genericeventparseconfig)
- [`GenericEventParser`](#genericeventparser)
- [`InnerInstructionParseParams`](#innerinstructionparseparams)
- [`InstructionParseParams`](#instructionparseparams)
- [`JupiterAccountLayout`](#jupiteraccountlayout)
- [`JupiterEventParser`](#jupitereventparser)
- [`JupiterExactOutRouteInstruction`](#jupiterexactoutrouteinstruction)
- [`JupiterLiquidityEvent`](#jupiterliquidityevent)
- [`JupiterParser`](#jupiterparser)
- [`JupiterParserFactory`](#jupiterparserfactory)
- [`JupiterRouteAnalysis`](#jupiterrouteanalysis)
- [`JupiterRouteData`](#jupiterroutedata)
- [`JupiterRouteInstruction`](#jupiterrouteinstruction)
- [`JupiterSwapBorshEvent`](#jupiterswapborshevent)
- [`JupiterSwapData`](#jupiterswapdata)
- [`JupiterSwapEvent`](#jupiterswapevent)
- [`LinearCurve`](#linearcurve)
- [`LiquidityEventParams`](#liquidityeventparams)
- [`MarginFiAccount`](#marginfiaccount)
- [`MarginFiBalance`](#marginfibalance)
- [`MarginFiBankConfig`](#marginfibankconfig)
- [`MarginFiBankState`](#marginfibankstate)
- [`MarginFiBorrowData`](#marginfiborrowdata)
- [`MarginFiBorrowEvent`](#marginfiborrowevent)
- [`MarginFiDepositData`](#marginfidepositdata)
- [`MarginFiDepositEvent`](#marginfidepositevent)
- [`MarginFiEventParser`](#marginfieventparser)
- [`MarginFiLendingAccount`](#marginfilendingaccount)
- [`MarginFiLiquidationData`](#marginfiliquidationdata)
- [`MarginFiLiquidationEvent`](#marginfiliquidationevent)
- [`MarginFiRepayData`](#marginfirepaydata)
- [`MarginFiRepayEvent`](#marginfirepayevent)
- [`MarginFiWithdrawData`](#marginfiwithdrawdata)
- [`MarginFiWithdrawEvent`](#marginfiwithdrawevent)
- [`MemoryMappedParser`](#memorymappedparser)
- [`MetaplexEventAnalysis`](#metaplexeventanalysis)
- [`MetaplexParser`](#metaplexparser)
- [`MetaplexParserFactory`](#metaplexparserfactory)
- [`MeteoraDynamicLiquidityData`](#meteoradynamicliquiditydata)
- [`MeteoraDynamicLiquidityEvent`](#meteoradynamicliquidityevent)
- [`MeteoraDynamicPoolData`](#meteoradynamicpooldata)
- [`MeteoraEventParser`](#meteoraeventparser)
- [`MeteoraLiquidityData`](#meteoraliquiditydata)
- [`MeteoraLiquidityEvent`](#meteoraliquidityevent)
- [`MeteoraSwapData`](#meteoraswapdata)
- [`MeteoraSwapEvent`](#meteoraswapevent)
- [`MintParams`](#mintparams)
- [`OrcaEventParser`](#orcaeventparser)
- [`OrcaLiquidityData`](#orcaliquiditydata)
- [`OrcaLiquidityEvent`](#orcaliquidityevent)
- [`OrcaPositionData`](#orcapositiondata)
- [`OrcaPositionEvent`](#orcapositionevent)
- [`OrcaSwapData`](#orcaswapdata)
- [`OrcaSwapEvent`](#orcaswapevent)
- [`ParserConfig`](#parserconfig)
- [`ParserMetrics`](#parsermetrics)
- [`ParsingInput`](#parsinginput)
- [`ParsingMetrics`](#parsingmetrics)
- [`ParsingOutput`](#parsingoutput)
- [`ParsingPipeline`](#parsingpipeline)
- [`ParsingPipelineBuilder`](#parsingpipelinebuilder)
- [`ParsingPipelineConfig`](#parsingpipelineconfig)
- [`PipelineStats`](#pipelinestats)
- [`PositionRewardInfo`](#positionrewardinfo)
- [`PriceData`](#pricedata)
- [`ProtocolEventParams`](#protocoleventparams)
- [`PumpBuyInstruction`](#pumpbuyinstruction)
- [`PumpCreatePoolInstruction`](#pumpcreatepoolinstruction)
- [`PumpDepositInstruction`](#pumpdepositinstruction)
- [`PumpFunParser`](#pumpfunparser)
- [`PumpFunParserFactory`](#pumpfunparserfactory)
- [`PumpSellInstruction`](#pumpsellinstruction)
- [`PumpSwapBuyEvent`](#pumpswapbuyevent)
- [`PumpSwapCreatePoolEvent`](#pumpswapcreatepoolevent)
- [`PumpSwapDepositEvent`](#pumpswapdepositevent)
- [`PumpSwapEventParser`](#pumpswapeventparser)
- [`PumpSwapSellEvent`](#pumpswapsellevent)
- [`PumpSwapWithdrawEvent`](#pumpswapwithdrawevent)
- [`PumpWithdrawInstruction`](#pumpwithdrawinstruction)
- [`RaydiumAmmV4DepositEvent`](#raydiumammv4depositevent)
- [`RaydiumAmmV4EventParser`](#raydiumammv4eventparser)
- [`RaydiumAmmV4Initialize2Event`](#raydiumammv4initialize2event)
- [`RaydiumAmmV4SwapEvent`](#raydiumammv4swapevent)
- [`RaydiumAmmV4WithdrawEvent`](#raydiumammv4withdrawevent)
- [`RaydiumAmmV4WithdrawPnlEvent`](#raydiumammv4withdrawpnlevent)
- [`RaydiumClmmClosePositionEvent`](#raydiumclmmclosepositionevent)
- [`RaydiumClmmCreatePoolEvent`](#raydiumclmmcreatepoolevent)
- [`RaydiumClmmDecreaseLiquidityV2Event`](#raydiumclmmdecreaseliquidityv2event)
- [`RaydiumClmmEventParser`](#raydiumclmmeventparser)
- [`RaydiumClmmIncreaseLiquidityV2Event`](#raydiumclmmincreaseliquidityv2event)
- [`RaydiumClmmOpenPositionV2Event`](#raydiumclmmopenpositionv2event)
- [`RaydiumClmmOpenPositionWithToken22NftEvent`](#raydiumclmmopenpositionwithtoken22nftevent)
- [`RaydiumClmmSwapEvent`](#raydiumclmmswapevent)
- [`RaydiumClmmSwapV2Event`](#raydiumclmmswapv2event)
- [`RaydiumCpmmDepositEvent`](#raydiumcpmmdepositevent)
- [`RaydiumCpmmEventParser`](#raydiumcpmmeventparser)
- [`RaydiumCpmmSwapEvent`](#raydiumcpmmswapevent)
- [`RaydiumV4Parser`](#raydiumv4parser)
- [`RaydiumV4ParserFactory`](#raydiumv4parserfactory)
- [`RouteHop`](#routehop)
- [`RoutePlan`](#routeplan)
- [`RoutePlanStep`](#routeplanstep)
- [`RpcConnectionPool`](#rpcconnectionpool)
- [`RuleResult`](#ruleresult)
- [`SIMDPatternMatcher`](#simdpatternmatcher)
- [`SharedAccountsExactOutRouteData`](#sharedaccountsexactoutroutedata)
- [`SharedAccountsRouteData`](#sharedaccountsroutedata)
- [`SolanaEvent`](#solanaevent)
- [`SolanaEventMetadata`](#solanaeventmetadata)
- [`SolanaEventMetadata`](#solanaeventmetadata)
- [`SolanaEventParser`](#solanaeventparser)
- [`SolanaInnerInstructionInput`](#solanainnerinstructioninput)
- [`SolanaInnerInstructionParser`](#solanainnerinstructionparser)
- [`SolanaTransactionInput`](#solanatransactioninput)
- [`StreamMetadata`](#streammetadata)
- [`SwapBaseInInstruction`](#swapbaseininstruction)
- [`SwapBaseOutInstruction`](#swapbaseoutinstruction)
- [`SwapData`](#swapdata)
- [`SwapEventParams`](#swapeventparams)
- [`SwapInfo`](#swapinfo)
- [`TokenMetadata`](#tokenmetadata)
- [`TransactionContext`](#transactioncontext)
- [`TransferData`](#transferdata)
- [`TransferInstruction`](#transferinstruction)
- [`ValidationConfig`](#validationconfig)
- [`ValidationMetrics`](#validationmetrics)
- [`ValidationPipeline`](#validationpipeline)
- [`ValidationResult`](#validationresult)
- [`ValidationStats`](#validationstats)
- [`VestingParams`](#vestingparams)
- [`WhirlpoolAccount`](#whirlpoolaccount)
- [`WhirlpoolRewardInfo`](#whirlpoolrewardinfo)
- [`WithdrawInstruction`](#withdrawinstruction)
- [`ZeroCopyEvent`](#zerocopyevent)
- [`ZeroCopyLiquidityEvent`](#zerocopyliquidityevent)
- [`ZeroCopySwapEvent`](#zerocopyswapevent)

### Functions (solana_metadata)

- [`core`](#core)
- [`core_mut`](#core_mut)
- [`create_metadata`](#create_metadata)
- [`event_kind`](#event_kind)
- [`id`](#id)
- [`kind`](#kind)
- [`new`](#new)
- [`set_id`](#set_id)

### Functions (solana_parser)

- [`add_protocol_parser`](#add_protocol_parser)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`parse_inner_instruction`](#parse_inner_instruction)
- [`parse_instruction`](#parse_instruction)
- [`supports_program`](#supports_program)
- [`with_legacy_parser`](#with_legacy_parser)
- [`with_received_time`](#with_received_time)
- [`with_received_time`](#with_received_time)

### Functions (solana_events)

- [`extract_data`](#extract_data)
- [`liquidity`](#liquidity)
- [`new`](#new)
- [`protocol_event`](#protocol_event)
- [`swap`](#swap)
- [`with_data`](#with_data)
- [`with_transfer_data`](#with_transfer_data)

### Traits

- [`ByteSliceEventParser`](#bytesliceeventparser)
- [`EventParser`](#eventparser)
- [`ToSolanaEvent`](#tosolanaevent)
- [`ValidationRule`](#validationrule)

### Functions (types)

- [`amount_to_shares`](#amount_to_shares)
- [`bin_id_to_price`](#bin_id_to_price)
- [`calculate_active_bin_price`](#calculate_active_bin_price)
- [`calculate_health_ratio`](#calculate_health_ratio)
- [`calculate_interest_rate`](#calculate_interest_rate)
- [`calculate_liquidation_threshold`](#calculate_liquidation_threshold)
- [`calculate_liquidity_distribution`](#calculate_liquidity_distribution)
- [`create_solana_metadata`](#create_solana_metadata)
- [`from_event_kind`](#from_event_kind)
- [`is_jupiter_v6_program`](#is_jupiter_v6_program)
- [`is_marginfi_program`](#is_marginfi_program)
- [`is_meteora_dlmm_program`](#is_meteora_dlmm_program)
- [`is_meteora_dynamic_program`](#is_meteora_dynamic_program)
- [`is_orca_whirlpool_program`](#is_orca_whirlpool_program)
- [`jupiter_v6_program_id`](#jupiter_v6_program_id)
- [`marginfi_bank_program_id`](#marginfi_bank_program_id)
- [`marginfi_program_id`](#marginfi_program_id)
- [`meteora_dlmm_program_id`](#meteora_dlmm_program_id)
- [`meteora_dynamic_program_id`](#meteora_dynamic_program_id)
- [`orca_whirlpool_program_id`](#orca_whirlpool_program_id)
- [`shares_to_amount`](#shares_to_amount)
- [`sqrt_price_to_price`](#sqrt_price_to_price)
- [`tick_index_to_price`](#tick_index_to_price)
- [`to_event_kind`](#to_event_kind)

### Functions (metadata_helpers)

- [`create_solana_metadata`](#create_solana_metadata)
- [`event_type`](#event_type)
- [`get_block_time`](#get_block_time)
- [`get_event_type`](#get_event_type)
- [`get_instruction_index`](#get_instruction_index)
- [`get_program_id`](#get_program_id)
- [`get_protocol_type`](#get_protocol_type)
- [`get_signature`](#get_signature)
- [`get_slot`](#get_slot)
- [`index`](#index)
- [`into_inner`](#into_inner)
- [`new`](#new)
- [`protocol_type`](#protocol_type)
- [`set_event_type`](#set_event_type)
- [`set_event_type`](#set_event_type)
- [`set_protocol_type`](#set_protocol_type)
- [`set_protocol_type`](#set_protocol_type)
- [`signature`](#signature)
- [`slot`](#slot)

### Functions (factory)

- [`add_parser`](#add_parser)
- [`get_parser`](#get_parser)
- [`get_parser_for_program`](#get_parser_for_program)
- [`new`](#new)
- [`parse_events_from_inner_instruction`](#parse_events_from_inner_instruction)
- [`parse_events_from_instruction`](#parse_events_from_instruction)
- [`should_handle`](#should_handle)
- [`supported_program_ids`](#supported_program_ids)
- [`with_all_parsers`](#with_all_parsers)

### Functions (metaplex)

- [`create_fast`](#create_fast)
- [`create_zero_copy`](#create_zero_copy)
- [`event_type`](#event_type)
- [`from_byte`](#from_byte)
- [`new`](#new)
- [`new_fast`](#new_fast)

### Functions (jupiter)

- [`bytes`](#bytes)
- [`create_fast`](#create_fast)
- [`create_standard`](#create_standard)
- [`create_zero_copy`](#create_zero_copy)
- [`event_type`](#event_type)
- [`from_bytes`](#from_bytes)
- [`new`](#new)
- [`new_fast`](#new_fast)
- [`new_standard`](#new_standard)

### Functions (pump_fun)

- [`create_standard`](#create_standard)
- [`create_zero_copy`](#create_zero_copy)
- [`event_type`](#event_type)
- [`from_u64`](#from_u64)
- [`new`](#new)
- [`new_standard`](#new_standard)

### Functions (raydium_v4)

- [`create_standard`](#create_standard)
- [`create_zero_copy`](#create_zero_copy)
- [`event_type`](#event_type)
- [`from_byte`](#from_byte)
- [`new_standard`](#new_standard)

### Functions (enrichment)

- [`enrich_event`](#enrich_event)
- [`enrich_events`](#enrich_events)
- [`get_cache_stats`](#get_cache_stats)
- [`new`](#new)

### Functions (parsing)

- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`build`](#build)
- [`get_stats`](#get_stats)
- [`new`](#new)
- [`new`](#new)
- [`process_stream`](#process_stream)
- [`take_output_receiver`](#take_output_receiver)
- [`with_batch_size`](#with_batch_size)
- [`with_batch_timeout`](#with_batch_timeout)
- [`with_concurrency_limit`](#with_concurrency_limit)
- [`with_metrics`](#with_metrics)
- [`with_parser_config`](#with_parser_config)

### Functions (validation)

- [`add_rule`](#add_rule)
- [`get_stats`](#get_stats)
- [`new`](#new)
- [`validate_event`](#validate_event)
- [`validate_events`](#validate_events)

### Functions (events)

- [`amount_in`](#amount_in)
- [`amount_out`](#amount_out)
- [`block_number`](#block_number)
- [`event_type`](#event_type)
- [`get_json_data`](#get_json_data)
- [`get_parsed_data`](#get_parsed_data)
- [`id`](#id)
- [`index`](#index)
- [`input_mint`](#input_mint)
- [`lp_amount`](#lp_amount)
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
- [`new_borrowed`](#new_borrowed)
- [`new_owned`](#new_owned)
- [`output_mint`](#output_mint)
- [`pool_address`](#pool_address)
- [`protocol_type`](#protocol_type)
- [`raw_data`](#raw_data)
- [`set_json_data`](#set_json_data)
- [`set_parsed_data`](#set_parsed_data)
- [`signature`](#signature)
- [`slot`](#slot)
- [`timestamp`](#timestamp)
- [`to_owned`](#to_owned)
- [`token_a_amount`](#token_a_amount)
- [`token_b_amount`](#token_b_amount)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)

### Functions (parsers)

- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`add_pattern`](#add_pattern)
- [`data_slice`](#data_slice)
- [`find_matches`](#find_matches)
- [`from_file`](#from_file)
- [`get_client`](#get_client)
- [`get_stats`](#get_stats)
- [`match_discriminator`](#match_discriminator)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`parse_all`](#parse_all)
- [`parse_batch`](#parse_batch)
- [`position`](#position)
- [`read_bytes`](#read_bytes)
- [`read_pubkey`](#read_pubkey)
- [`read_u32_le`](#read_u32_le)
- [`read_u64_le`](#read_u64_le)
- [`read_u8`](#read_u8)
- [`remaining`](#remaining)
- [`remaining_data`](#remaining_data)
- [`size`](#size)
- [`size`](#size)
- [`skip`](#skip)

### Functions (traits)

- [`new`](#new)

### Functions (parser)

- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)

## Constants

### ANCHOR_DISCRIMINATOR_SIZE

**Source**: `src/constants.rs`

```rust
const ANCHOR_DISCRIMINATOR_SIZE: usize
```

Anchor discriminator size (8 bytes)

---

### BONK_PROGRAM_ID

**Source**: `bonk/parser.rs`

```rust
const BONK_PROGRAM_ID: Pubkey
```

Bonk program ID

---

### BUY_EVENT

**Source**: `pumpswap/events.rs`

```rust
const BUY_EVENT: &str
```

String identifier for PumpSwap buy events

---

### BUY_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const BUY_EVENT_BYTES: &[u8]
```

Byte array discriminator for PumpSwap buy events

---

### BUY_EXACT_IN_IX

**Source**: `bonk/events.rs`

```rust
const BUY_EXACT_IN_IX: &[u8]
```

Instruction discriminator for buy exact in operations

---

### BUY_EXACT_OUT_IX

**Source**: `bonk/events.rs`

```rust
const BUY_EXACT_OUT_IX: &[u8]
```

Instruction discriminator for buy exact out operations

---

### BUY_IX

**Source**: `pumpswap/events.rs`

```rust
const BUY_IX: &[u8]
```

Instruction discriminator for PumpSwap buy operations

---

### CLOSE_POSITION

**Source**: `raydium_clmm/discriminators.rs`

```rust
const CLOSE_POSITION: &[u8]
```

Instruction discriminator for closing liquidity positions

---

### CLOSE_POSITION_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const CLOSE_POSITION_DISCRIMINATOR: [u8; 8]
```

Instruction discriminator for closing liquidity positions

---

### CREATE_POOL

**Source**: `raydium_clmm/discriminators.rs`

```rust
const CREATE_POOL: &[u8]
```

Instruction discriminator for creating new pools

---

### CREATE_POOL_EVENT

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_EVENT: &str
```

String identifier for PumpSwap create pool events

---

### CREATE_POOL_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_EVENT_BYTES: &[u8]
```

Byte array discriminator for PumpSwap create pool events

---

### CREATE_POOL_IX

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_IX: &[u8]
```

Instruction discriminator for PumpSwap create pool operations

---

### DECREASE_LIQUIDITY_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const DECREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Instruction discriminator for decreasing liquidity in positions

---

### DECREASE_LIQUIDITY_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const DECREASE_LIQUIDITY_V2: &[u8]
```

Instruction discriminator for decreasing liquidity in positions (version 2)

---

### DEPOSIT

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const DEPOSIT: &[u8]
```

Instruction discriminator for depositing liquidity into the pool

---

### DEPOSIT_EVENT

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_EVENT: &str
```

String identifier for PumpSwap deposit events

---

### DEPOSIT_EVENT

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_EVENT: &str
```

String identifier for deposit events

---

### DEPOSIT_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_EVENT_BYTES: &[u8]
```

Byte array discriminator for PumpSwap deposit events

---

### DEPOSIT_EVENT_BYTES

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_EVENT_BYTES: &[u8]
```

Byte array discriminator for deposit events

---

### DEPOSIT_IX

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_IX: &[u8]
```

Instruction discriminator for PumpSwap deposit operations

---

### DEPOSIT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_IX: &[u8]
```

Instruction discriminator for deposit operations

---

### DLMM_ADD_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Discriminator for DLMM add liquidity instruction

---

### DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Discriminator for DLMM remove liquidity instruction

---

### DLMM_SWAP_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_SWAP_DISCRIMINATOR: [u8; 8]
```

Meteora instruction discriminators
Discriminator for DLMM swap instruction

---

### DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Discriminator for Dynamic AMM add liquidity instruction

---

### DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Discriminator for Dynamic AMM remove liquidity instruction

---

### EXACT_OUT_ROUTE_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8]
```

Discriminator for shared accounts exact out route instruction

---

### INCREASE_LIQUIDITY_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const INCREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

Instruction discriminator for increasing liquidity in positions

---

### INCREASE_LIQUIDITY_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const INCREASE_LIQUIDITY_V2: &[u8]
```

Instruction discriminator for increasing liquidity in positions (version 2)

---

### INITIALIZE2

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const INITIALIZE2: &[u8]
```

Instruction discriminator for initializing the AMM pool (version 2)

---

### INITIALIZE_IX

**Source**: `bonk/events.rs`

```rust
const INITIALIZE_IX: &[u8]
```

Instruction discriminator for pool initialization

---

### INITIALIZE_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const INITIALIZE_IX: &[u8]
```

Instruction discriminator for pool initialization

---

### INSTRUCTION_DATA_OFFSET

**Source**: `src/constants.rs`

```rust
const INSTRUCTION_DATA_OFFSET: usize
```

Instruction data offset (after discriminator)

---

### JUPITER_MIN_INSTRUCTION_SIZE

**Source**: `src/constants.rs`

```rust
const JUPITER_MIN_INSTRUCTION_SIZE: usize
```

Jupiter minimum instruction size (discriminator + pubkeys + amounts)

---

### JUPITER_PROGRAM_ID

**Source**: `parsers/jupiter.rs`

```rust
const JUPITER_PROGRAM_ID: &str
```

Jupiter aggregator program ID

---

### JUPITER_V6_PROGRAM_ID

**Source**: `jupiter/types.rs`

```rust
const JUPITER_V6_PROGRAM_ID: &str
```

Jupiter V6 program ID

---

### LEGACY_EXACT_OUT_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const LEGACY_EXACT_OUT_DISCRIMINATOR: [u8; 8]
```

Discriminator for legacy exact out route instruction

---

### LEGACY_ROUTE_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const LEGACY_ROUTE_DISCRIMINATOR: [u8; 8]
```

Discriminator for legacy route instruction

---

### MARGINFI_BANK_PROGRAM_ID

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_BANK_PROGRAM_ID: &str
```

MarginFi Bank program ID (for lending pools)

---

### MARGINFI_BORROW_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_BORROW_DISCRIMINATOR: [u8; 8]
```

Discriminator for lending borrow instruction

---

### MARGINFI_DEPOSIT_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_DEPOSIT_DISCRIMINATOR: [u8; 8]
```

MarginFi instruction discriminators
Discriminator for lending deposit instruction

---

### MARGINFI_LIQUIDATE_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_LIQUIDATE_DISCRIMINATOR: [u8; 8]
```

Discriminator for liquidation instruction

---

### MARGINFI_PROGRAM_ID

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_PROGRAM_ID: &str
```

MarginFi program ID

---

### MARGINFI_REPAY_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_REPAY_DISCRIMINATOR: [u8; 8]
```

Discriminator for lending repay instruction

---

### MARGINFI_WITHDRAW_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_WITHDRAW_DISCRIMINATOR: [u8; 8]
```

Discriminator for lending withdraw instruction

---

### METAPLEX_AUCTION_HOUSE_PROGRAM_ID

**Source**: `parsers/metaplex.rs`

```rust
const METAPLEX_AUCTION_HOUSE_PROGRAM_ID: &str
```

Metaplex Auction House program ID

---

### METAPLEX_TOKEN_METADATA_PROGRAM_ID

**Source**: `parsers/metaplex.rs`

```rust
const METAPLEX_TOKEN_METADATA_PROGRAM_ID: &str
```

Metaplex Token Metadata program ID

---

### METEORA_DLMM_PROGRAM_ID

**Source**: `meteora/types.rs`

```rust
const METEORA_DLMM_PROGRAM_ID: &str
```

Meteora DLMM program ID

---

### METEORA_DYNAMIC_PROGRAM_ID

**Source**: `meteora/types.rs`

```rust
const METEORA_DYNAMIC_PROGRAM_ID: &str
```

Meteora Dynamic program ID

---

### MIGRATE_TO_AMM_IX

**Source**: `bonk/events.rs`

```rust
const MIGRATE_TO_AMM_IX: &[u8]
```

Instruction discriminator for migrating to AMM

---

### MIGRATE_TO_CPSWAP_IX

**Source**: `bonk/events.rs`

```rust
const MIGRATE_TO_CPSWAP_IX: &[u8]
```

Instruction discriminator for migrating to constant product swap

---

### MIN_COMPLEX_ACCOUNTS

**Source**: `src/constants.rs`

```rust
const MIN_COMPLEX_ACCOUNTS: usize
```

Minimum accounts for complex operations (AMM v4, etc.)

---

### MIN_LIQUIDITY_ACCOUNTS

**Source**: `src/constants.rs`

```rust
const MIN_LIQUIDITY_ACCOUNTS: usize
```

Minimum accounts for liquidity operations

---

### MIN_LIQUIDITY_INSTRUCTION_SIZE

**Source**: `src/constants.rs`

```rust
const MIN_LIQUIDITY_INSTRUCTION_SIZE: usize
```

Minimum size for liquidity instructions (discriminator + 3 u64s)

---

### MIN_POSITION_ACCOUNTS

**Source**: `src/constants.rs`

```rust
const MIN_POSITION_ACCOUNTS: usize
```

Minimum accounts for position operations

---

### MIN_POSITION_INSTRUCTION_SIZE

**Source**: `src/constants.rs`

```rust
const MIN_POSITION_INSTRUCTION_SIZE: usize
```

Minimum size for position instructions (discriminator + various fields)

---

### MIN_SWAP_ACCOUNTS

**Source**: `src/constants.rs`

```rust
const MIN_SWAP_ACCOUNTS: usize
```

Minimum accounts for basic swap operations

---

### MIN_SWAP_INSTRUCTION_SIZE

**Source**: `src/constants.rs`

```rust
const MIN_SWAP_INSTRUCTION_SIZE: usize
```

Minimum size for swap instructions (discriminator + 2 u64s)

---

### OPEN_POSITION_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const OPEN_POSITION_DISCRIMINATOR: [u8; 8]
```

Instruction discriminator for opening liquidity positions

---

### OPEN_POSITION_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const OPEN_POSITION_V2: &[u8]
```

Instruction discriminator for opening liquidity positions (version 2)

---

### OPEN_POSITION_WITH_TOKEN_22_NFT

**Source**: `raydium_clmm/discriminators.rs`

```rust
const OPEN_POSITION_WITH_TOKEN_22_NFT: &[u8]
```

Instruction discriminator for opening positions with Token-22 NFT

---

### ORCA_WHIRLPOOL_PROGRAM_ID

**Source**: `orca/types.rs`

```rust
const ORCA_WHIRLPOOL_PROGRAM_ID: &str
```

Orca Whirlpool program ID

---

### POOL_CREATE_EVENT

**Source**: `bonk/events.rs`

```rust
const POOL_CREATE_EVENT: &str
```

String identifier for Bonk pool creation events

---

### POOL_CREATE_EVENT_BYTES

**Source**: `bonk/events.rs`

```rust
const POOL_CREATE_EVENT_BYTES: &[u8]
```

Byte array discriminator for pool creation events, used for efficient on-chain parsing

---

### PUBKEY_RANGE

**Source**: `src/constants.rs`

```rust
const PUBKEY_RANGE: std::ops::Range<usize>
```

Byte range for parsing a single public key value

---

### PUBKEY_SIZE

**Source**: `src/constants.rs`

```rust
const PUBKEY_SIZE: usize
```

Size of a Solana public key in bytes

---

### PUMPSWAP_PROGRAM_ID

**Source**: `pumpswap/parser.rs`

```rust
const PUMPSWAP_PROGRAM_ID: Pubkey
```

PumpSwap program ID

---

### PUMP_FUN_PROGRAM_ID

**Source**: `parsers/pump_fun.rs`

```rust
const PUMP_FUN_PROGRAM_ID: &str
```

PumpFun program ID

---

### RAYDIUM_AMM_V4_PROGRAM_ID

**Source**: `parsers/raydium_v4.rs`

```rust
const RAYDIUM_AMM_V4_PROGRAM_ID: &str
```

Raydium AMM V4 program ID

---

### RAYDIUM_AMM_V4_PROGRAM_ID

**Source**: `raydium_amm_v4/parser.rs`

```rust
const RAYDIUM_AMM_V4_PROGRAM_ID: Pubkey
```

Raydium AMM V4 program ID

---

### RAYDIUM_CLMM_PROGRAM_ID

**Source**: `raydium_clmm/parser.rs`

```rust
const RAYDIUM_CLMM_PROGRAM_ID: Pubkey
```

Raydium CLMM program ID

---

### RAYDIUM_CPMM_PROGRAM_ID

**Source**: `raydium_cpmm/parser.rs`

```rust
const RAYDIUM_CPMM_PROGRAM_ID: Pubkey
```

Raydium CPMM program ID

---

### ROUTE_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const ROUTE_DISCRIMINATOR: [u8; 8]
```

Jupiter swap discriminators (calculated from Anchor's "global:<instruction_name>")
sharedAccountsRoute is the most common Jupiter V6 swap instruction

---

### ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const ROUTE_WITH_TOKEN_LEDGER_DISCRIMINATOR: [u8; 8]
```

Discriminator for route with token ledger instruction

---

### SECOND_U64_RANGE

**Source**: `src/constants.rs`

```rust
const SECOND_U64_RANGE: std::ops::Range<usize>
```

Second u64 range (for parsing two consecutive u64 values)

---

### SELL_EVENT

**Source**: `pumpswap/events.rs`

```rust
const SELL_EVENT: &str
```

String identifier for PumpSwap sell events

---

### SELL_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const SELL_EVENT_BYTES: &[u8]
```

Byte array discriminator for PumpSwap sell events

---

### SELL_EXACT_IN_IX

**Source**: `bonk/events.rs`

```rust
const SELL_EXACT_IN_IX: &[u8]
```

Instruction discriminator for sell exact in operations

---

### SELL_EXACT_OUT_IX

**Source**: `bonk/events.rs`

```rust
const SELL_EXACT_OUT_IX: &[u8]
```

Instruction discriminator for sell exact out operations

---

### SELL_IX

**Source**: `pumpswap/events.rs`

```rust
const SELL_IX: &[u8]
```

Instruction discriminator for PumpSwap sell operations

---

### SWAP

**Source**: `raydium_clmm/discriminators.rs`

```rust
const SWAP: &[u8]
```

Raydium CLMM instruction discriminators
Instruction discriminator for swap operations

---

### SWAP_BASE_IN

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const SWAP_BASE_IN: &[u8]
```

Raydium AMM V4 instruction discriminators
Instruction discriminator for swapping with base token as input

---

### SWAP_BASE_INPUT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_BASE_INPUT_IX: &[u8]
```

Instruction discriminator for swap with base input

---

### SWAP_BASE_OUT

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const SWAP_BASE_OUT: &[u8]
```

Instruction discriminator for swapping with base token as output

---

### SWAP_BASE_OUTPUT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_BASE_OUTPUT_IX: &[u8]
```

Instruction discriminator for swap with base output

---

### SWAP_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const SWAP_DISCRIMINATOR: [u8; 8]
```

Discriminator for swap instruction

---

### SWAP_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const SWAP_DISCRIMINATOR: [u8; 8]
```

Orca Whirlpool instruction discriminators (calculated from Anchor's "global:<instruction_name>")
Instruction discriminator for swap operations

---

### SWAP_EVENT

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_EVENT: &str
```

String identifier for swap events

---

### SWAP_EVENT_BYTES

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_EVENT_BYTES: &[u8]
```

Byte array discriminator for swap events

---

### SWAP_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const SWAP_V2: &[u8]
```

Instruction discriminator for swap operations (version 2)

---

### THIRD_U64_RANGE

**Source**: `src/constants.rs`

```rust
const THIRD_U64_RANGE: std::ops::Range<usize>
```

Third u64 range (for parsing three consecutive u64 values)

---

### TRADE_EVENT

**Source**: `bonk/events.rs`

```rust
const TRADE_EVENT: &str
```

String identifier for Bonk trade events

---

### TRADE_EVENT_BYTES

**Source**: `bonk/events.rs`

```rust
const TRADE_EVENT_BYTES: &[u8]
```

Byte array discriminator for trade events, used for efficient on-chain parsing

---

### U128_RANGE

**Source**: `src/constants.rs`

```rust
const U128_RANGE: std::ops::Range<usize>
```

Byte range for parsing a single u128 value

---

### U128_SIZE

**Source**: `src/constants.rs`

```rust
const U128_SIZE: usize
```

Size of a u128 in bytes

---

### U16_RANGE

**Source**: `src/constants.rs`

```rust
const U16_RANGE: std::ops::Range<usize>
```

Byte range for parsing a single u16 value

---

### U16_SIZE

**Source**: `src/constants.rs`

```rust
const U16_SIZE: usize
```

Size of a u16 in bytes

---

### U32_RANGE

**Source**: `src/constants.rs`

```rust
const U32_RANGE: std::ops::Range<usize>
```

Byte range for parsing a single u32 value

---

### U32_SIZE

**Source**: `src/constants.rs`

```rust
const U32_SIZE: usize
```

Size of a u32 in bytes

---

### U64_RANGE

**Source**: `src/constants.rs`

```rust
const U64_RANGE: std::ops::Range<usize>
```

Byte range for parsing a single u64 value

---

### U64_SIZE

**Source**: `src/constants.rs`

```rust
const U64_SIZE: usize
```

Size of a u64 in bytes

---

### U8_RANGE

**Source**: `src/constants.rs`

```rust
const U8_RANGE: std::ops::Range<usize>
```

Common byte ranges for parsing

---

### U8_SIZE

**Source**: `src/constants.rs`

```rust
const U8_SIZE: usize
```

Size constants for basic data types

---

### WITHDRAW

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const WITHDRAW: &[u8]
```

Instruction discriminator for withdrawing liquidity from the pool

---

### WITHDRAW_EVENT

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_EVENT: &str
```

String identifier for PumpSwap withdraw events

---

### WITHDRAW_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_EVENT_BYTES: &[u8]
```

Byte array discriminator for PumpSwap withdraw events

---

### WITHDRAW_IX

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_IX: &[u8]
```

Instruction discriminator for PumpSwap withdraw operations

---

### WITHDRAW_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const WITHDRAW_IX: &[u8]
```

Instruction discriminator for withdraw operations

---

### WITHDRAW_PNL

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const WITHDRAW_PNL: &[u8]
```

Instruction discriminator for withdrawing profit and loss from the pool

---

## Enums

### EnrichmentError

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum EnrichmentError { /// HTTP request failed #[error("HTTP request failed: {0}")] HttpError(#[from] reqwest::Error), /// Serialization error #[error("Serialization error: {0}")] SerializationError(#[from] serde_json::Error), /// Task execution error #[error("Task execution error: {0}")] TaskError(String), /// Cache error #[error("Cache error: {0}")] CacheError(String), /// API rate limit exceeded #[error("API rate limit exceeded")] RateLimitExceeded, }
```

Error type for enrichment operations

**Variants**:

- `HttpError(#[from] reqwest::Error)`
- `SerializationError(#[from] serde_json::Error)`
- `TaskError(String)`
- `CacheError(String)`
- `RateLimitExceeded`

---

### EventType

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize, Default)]
```

```rust
pub enum EventType { /// Token swap transaction Swap, /// Add liquidity to a pool AddLiquidity, /// Remove liquidity from a pool RemoveLiquidity, /// Borrow funds from a lending protocol Borrow, /// Repay borrowed funds Repay, /// Liquidate an undercollateralized position Liquidate, /// Transfer tokens between accounts Transfer, /// Mint new tokens Mint, /// Burn existing tokens Burn, /// Create a new liquidity pool CreatePool, /// Update pool parameters UpdatePool, /// General transaction event Transaction, /// Block-level event Block, /// Smart contract execution event ContractEvent, /// Price update event PriceUpdate, /// Order book update OrderBook, /// Trade execution Trade, /// Fee structure update FeeUpdate, /// Bonk buy with exact input amount BonkBuyExactIn, /// Bonk buy with exact output amount BonkBuyExactOut, /// Bonk sell with exact input amount BonkSellExactIn, /// Bonk sell with exact output amount BonkSellExactOut, /// Bonk pool initialization BonkInitialize, /// Bonk migration to AMM BonkMigrateToAmm, /// Bonk migration to constant product swap BonkMigrateToCpswap, /// PumpSwap buy transaction PumpSwapBuy, /// PumpSwap sell transaction PumpSwapSell, /// PumpSwap pool creation PumpSwapCreatePool, /// PumpSwap deposit PumpSwapDeposit, /// PumpSwap withdrawal PumpSwapWithdraw, /// PumpSwap parameter update PumpSwapSetParams, /// Raydium swap transaction RaydiumSwap, /// Raydium deposit RaydiumDeposit, /// Raydium withdrawal RaydiumWithdraw, /// Raydium AMM V4 swap with base token input RaydiumAmmV4SwapBaseIn, /// Raydium AMM V4 swap with base token output RaydiumAmmV4SwapBaseOut, /// Raydium AMM V4 deposit RaydiumAmmV4Deposit, /// Raydium AMM V4 second initialization RaydiumAmmV4Initialize2, /// Raydium AMM V4 withdrawal RaydiumAmmV4Withdraw, /// Raydium AMM V4 profit and loss withdrawal RaydiumAmmV4WithdrawPnl, /// Raydium CLMM swap RaydiumClmmSwap, /// Raydium CLMM swap version 2 RaydiumClmmSwapV2, /// Raydium CLMM pool creation RaydiumClmmCreatePool, /// Raydium CLMM open position version 2 RaydiumClmmOpenPositionV2, /// Raydium CLMM increase liquidity version 2 RaydiumClmmIncreaseLiquidityV2, /// Raydium CLMM decrease liquidity version 2 RaydiumClmmDecreaseLiquidityV2, /// Raydium CLMM close position RaydiumClmmClosePosition, /// Raydium CLMM open position with Token22 NFT RaydiumClmmOpenPositionWithToken22Nft, /// Raydium CPMM swap RaydiumCpmmSwap, /// Raydium CPMM swap with base input RaydiumCpmmSwapBaseInput, /// Raydium CPMM swap with base output RaydiumCpmmSwapBaseOutput, /// Raydium CPMM deposit RaydiumCpmmDeposit, /// Raydium CPMM withdrawal RaydiumCpmmWithdraw, /// Raydium CPMM pool creation RaydiumCpmmCreatePool, /// Open a liquidity position OpenPosition, /// Close a liquidity position ClosePosition, /// Increase liquidity in position IncreaseLiquidity, /// Decrease liquidity in position DecreaseLiquidity, /// General deposit operation Deposit, /// General withdrawal operation Withdraw, /// Unknown or unclassified event type #[default] Unknown, }
```

Enumeration of DeFi event types supported across protocols

**Variants**:

- `Swap`
- `AddLiquidity`
- `RemoveLiquidity`
- `Borrow`
- `Repay`
- `Liquidate`
- `Transfer`
- `Mint`
- `Burn`
- `CreatePool`
- `UpdatePool`
- `Transaction`
- `Block`
- `ContractEvent`
- `PriceUpdate`
- `OrderBook`
- `Trade`
- `FeeUpdate`
- `BonkBuyExactIn`
- `BonkBuyExactOut`
- `BonkSellExactIn`
- `BonkSellExactOut`
- `BonkInitialize`
- `BonkMigrateToAmm`
- `BonkMigrateToCpswap`
- `PumpSwapBuy`
- `PumpSwapSell`
- `PumpSwapCreatePool`
- `PumpSwapDeposit`
- `PumpSwapWithdraw`
- `PumpSwapSetParams`
- `RaydiumSwap`
- `RaydiumDeposit`
- `RaydiumWithdraw`
- `RaydiumAmmV4SwapBaseIn`
- `RaydiumAmmV4SwapBaseOut`
- `RaydiumAmmV4Deposit`
- `RaydiumAmmV4Initialize2`
- `RaydiumAmmV4Withdraw`
- `RaydiumAmmV4WithdrawPnl`
- `RaydiumClmmSwap`
- `RaydiumClmmSwapV2`
- `RaydiumClmmCreatePool`
- `RaydiumClmmOpenPositionV2`
- `RaydiumClmmIncreaseLiquidityV2`
- `RaydiumClmmDecreaseLiquidityV2`
- `RaydiumClmmClosePosition`
- `RaydiumClmmOpenPositionWithToken22Nft`
- `RaydiumCpmmSwap`
- `RaydiumCpmmSwapBaseInput`
- `RaydiumCpmmSwapBaseOutput`
- `RaydiumCpmmDeposit`
- `RaydiumCpmmWithdraw`
- `RaydiumCpmmCreatePool`
- `OpenPosition`
- `ClosePosition`
- `IncreaseLiquidity`
- `DecreaseLiquidity`
- `Deposit`
- `Withdraw`
- `Unknown`

---

### JupiterDiscriminator

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum JupiterDiscriminator { /// Route discriminator: [229, 23, 203, 151, 122, 227, 173, 42] Route, /// RouteWithTokenLedger discriminator: [206, 198, 71, 54, 47, 82, 194, 13] RouteWithTokenLedger, /// ExactOutRoute discriminator: [123, 87, 17, 219, 42, 126, 197, 78] ExactOutRoute, /// SharedAccountsRoute discriminator: [95, 180, 10, 172, 84, 174, 232, 239] SharedAccountsRoute, /// SharedAccountsRouteWithTokenLedger discriminator: [18, 99, 149, 21, 45, 126, 144, 122] SharedAccountsRouteWithTokenLedger, /// SharedAccountsExactOutRoute discriminator: [77, 119, 2, 23, 198, 126, 79, 175] SharedAccountsExactOutRoute, }
```

Jupiter instruction discriminators (using Anchor discriminators)

**Variants**:

- `Route`
- `RouteWithTokenLedger`
- `ExactOutRoute`
- `SharedAccountsRoute`
- `SharedAccountsRouteWithTokenLedger`
- `SharedAccountsExactOutRoute`

---

### MarginFiAccountType

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
```

```rust
pub enum MarginFiAccountType { /// MarginFi group account containing global settings MarginfiGroup, /// Individual user account for lending positions MarginfiAccount, /// Bank account representing a lending pool Bank, /// Unknown or unrecognized account type Unknown, }
```

MarginFi account types

**Variants**:

- `MarginfiGroup`
- `MarginfiAccount`
- `Bank`
- `Unknown`

---

### MetaplexAuctionHouseDiscriminator

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum MetaplexAuctionHouseDiscriminator { /// Buy NFT from auction house Buy, /// Sell NFT on auction house Sell, /// Execute sale between buyer and seller ExecuteSale, /// Deposit funds to auction house Deposit, /// Withdraw funds from auction house Withdraw, /// Cancel buy or sell order Cancel, }
```

Metaplex Auction House discriminators

**Variants**:

- `Buy`
- `Sell`
- `ExecuteSale`
- `Deposit`
- `Withdraw`
- `Cancel`

---

### MetaplexTokenMetadataDiscriminator

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum MetaplexTokenMetadataDiscriminator { /// Create metadata account for NFT CreateMetadataAccount = 0, /// Update existing metadata account UpdateMetadataAccount = 1, /// Deprecated create master edition instruction DeprecatedCreateMasterEdition = 2, /// Deprecated mint new edition from master edition via printing token DeprecatedMintNewEditionFromMasterEditionViaPrintingToken = 3, /// Update primary sale happened via token UpdatePrimarySaleHappenedViaToken = 4, /// Deprecated set reservation list instruction DeprecatedSetReservationList = 5, /// Deprecated create reservation list instruction DeprecatedCreateReservationList = 6, /// Sign metadata for creator verification SignMetadata = 7, /// Deprecated mint printing tokens via token DeprecatedMintPrintingTokensViaToken = 8, /// Deprecated mint printing tokens instruction DeprecatedMintPrintingTokens = 9, /// Create master edition for limited editions CreateMasterEdition = 10, /// Mint new edition from master edition via token MintNewEditionFromMasterEditionViaToken = 11, /// Convert master edition V1 to V2 ConvertMasterEditionV1ToV2 = 12, /// Mint new edition from master edition via vault proxy MintNewEditionFromMasterEditionViaVaultProxy = 13, /// Puff metadata to increase size PuffMetadata = 14, /// Update metadata account version 2 UpdateMetadataAccountV2 = 15, /// Create metadata account version 2 CreateMetadataAccountV2 = 16, /// Create master edition version 3 CreateMasterEditionV3 = 17, /// Verify collection membership VerifyCollection = 18, /// Utilize NFT for specific use case Utilize = 19, /// Approve use authority for NFT ApproveUseAuthority = 20, /// Revoke use authority for NFT RevokeUseAuthority = 21, /// Unverify collection membership UnverifyCollection = 22, /// Approve collection authority ApproveCollectionAuthority = 23, /// Revoke collection authority RevokeCollectionAuthority = 24, /// Set and verify collection in one transaction SetAndVerifyCollection = 25, /// Freeze delegated account FreezeDelegatedAccount = 26, /// Thaw delegated account ThawDelegatedAccount = 27, /// Remove creator verification RemoveCreatorVerification = 28, /// Burn NFT permanently BurnNft = 29, /// Verify creator signature VerifyCreator = 30, /// Unverify creator signature UnverifyCreator = 31, /// Bubblegum set collection size BubblegumSetCollectionSize = 32, /// Burn edition NFT BurnEditionNft = 33, /// Create metadata account version 3 CreateMetadataAccountV3 = 34, /// Set collection size SetCollectionSize = 35, /// Set token standard SetTokenStandard = 36, /// Bubblegum verify creator BubblegumVerifyCreator = 37, /// Bubblegum unverify creator BubblegumUnverifyCreator = 38, /// Bubblegum verify collection BubblegumVerifyCollection = 39, /// Bubblegum unverify collection BubblegumUnverifyCollection = 40, /// Bubblegum set and verify collection BubblegumSetAndVerifyCollection = 41, /// Transfer NFT ownership Transfer = 42, }
```

Metaplex instruction discriminators for Token Metadata program

**Variants**:

- `CreateMetadataAccount`
- `UpdateMetadataAccount`
- `DeprecatedCreateMasterEdition`
- `DeprecatedMintNewEditionFromMasterEditionViaPrintingToken`
- `UpdatePrimarySaleHappenedViaToken`
- `DeprecatedSetReservationList`
- `DeprecatedCreateReservationList`
- `SignMetadata`
- `DeprecatedMintPrintingTokensViaToken`
- `DeprecatedMintPrintingTokens`
- `CreateMasterEdition`
- `MintNewEditionFromMasterEditionViaToken`
- `ConvertMasterEditionV1ToV2`
- `MintNewEditionFromMasterEditionViaVaultProxy`
- `PuffMetadata`
- `UpdateMetadataAccountV2`
- `CreateMetadataAccountV2`
- `CreateMasterEditionV3`
- `VerifyCollection`
- `Utilize`
- `ApproveUseAuthority`
- `RevokeUseAuthority`
- `UnverifyCollection`
- `ApproveCollectionAuthority`
- `RevokeCollectionAuthority`
- `SetAndVerifyCollection`
- `FreezeDelegatedAccount`
- `ThawDelegatedAccount`
- `RemoveCreatorVerification`
- `BurnNft`
- `VerifyCreator`
- `UnverifyCreator`
- `BubblegumSetCollectionSize`
- `BurnEditionNft`
- `CreateMetadataAccountV3`
- `SetCollectionSize`
- `SetTokenStandard`
- `BubblegumVerifyCreator`
- `BubblegumUnverifyCreator`
- `BubblegumVerifyCollection`
- `BubblegumUnverifyCollection`
- `BubblegumSetAndVerifyCollection`
- `Transfer`

---

### ParseError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ParseError { /// Not enough bytes available for the requested operation #[error("Not enough bytes: expected {expected}, got {found} at offset {offset}")] NotEnoughBytes { /// Number of bytes expected expected: usize, /// Number of bytes actually found found: usize, /// Byte offset where the error occurred offset: usize, }, /// Invalid discriminator for the instruction #[error("Invalid discriminator: expected {expected:?}, got {found:?}")] InvalidDiscriminator { /// Expected discriminator bytes expected: Vec<u8>, /// Actual discriminator bytes found found: Vec<u8>, }, /// Invalid account index #[error("Account index {index} out of bounds (max: {max})")] InvalidAccountIndex { /// The invalid account index that was accessed index: usize, /// Maximum valid account index max: usize, }, /// Invalid public key format #[error("Invalid public key: {0}")] InvalidPubkey(String), /// UTF-8 decoding error #[error("UTF-8 decoding error: {0}")] Utf8Error(#[from] std::str::Utf8Error), /// Borsh deserialization error #[error("Borsh deserialization error: {0}")] BorshError(String), /// Invalid enum variant #[error("Invalid enum variant {variant} for type {type_name}")] InvalidEnumVariant { /// The invalid variant value variant: u8, /// Name of the enum type type_name: String, }, /// Invalid instruction type #[error("Invalid instruction type: {0}")] InvalidInstructionType(String), /// Missing required field #[error("Missing required field: {0}")] MissingField(String), /// Invalid data format #[error("Invalid data format: {0}")] InvalidDataFormat(String), /// Overflow error #[error("Arithmetic overflow: {0}")] Overflow(String), /// Generic parsing error #[error("Parse error: {0}")] Generic(String), /// Network error (for streaming) #[error("Network error: {0}")] Network(String), /// Timeout error #[error("Operation timed out: {0}")] Timeout(String), }
```

Custom error type for event parsing operations

NOTE: This will be gradually replaced with EventError from riglr-events-core

**Variants**:

- `NotEnoughBytes`
- `expected`
- `found`
- `offset`
- `InvalidDiscriminator`
- `expected`
- `found`
- `InvalidAccountIndex`
- `index`
- `max`
- `InvalidPubkey(String)`
- `Utf8Error(#[from] std::str::Utf8Error)`
- `BorshError(String)`
- `InvalidEnumVariant`
- `variant`
- `type_name`
- `InvalidInstructionType(String)`
- `MissingField(String)`
- `InvalidDataFormat(String)`
- `Overflow(String)`
- `Generic(String)`
- `Network(String)`
- `Timeout(String)`

---

### ParseError

**Source**: `zero_copy/parsers.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum ParseError { /// Invalid instruction data encountered during parsing #[error("Invalid instruction data: {0}")] InvalidInstructionData(String), /// Insufficient data available for parsing operation #[error("Insufficient data length: expected {expected}, got {actual}")] InsufficientData { /// Expected number of bytes expected: usize, /// Actual number of bytes available actual: usize }, /// Unknown discriminator value encountered #[error("Unknown discriminator: {discriminator:?}")] UnknownDiscriminator { /// The unrecognized discriminator bytes discriminator: Vec<u8> }, /// Borsh deserialization error #[error("Deserialization error: {0}")] DeserializationError(#[from] borsh::io::Error), /// Memory mapping operation error #[error("Memory map error: {0}")] MemoryMapError(String), }
```

Error type for parsing operations

**Variants**:

- `InvalidInstructionData(String)`
- `InsufficientData`
- `expected`
- `actual`
- `UnknownDiscriminator`
- `discriminator`
- `DeserializationError(#[from] borsh::io::Error)`
- `MemoryMapError(String)`

---

### PipelineError

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum PipelineError { /// Error acquiring semaphore permit #[error("Semaphore error")] SemaphoreError(()), /// Error sending data through channel #[error("Channel error")] ChannelError, /// Error during event parsing #[error("Parse error: {0}")] ParseError(#[from] ParseError), /// Invalid pipeline configuration #[error("Configuration error: {0}")] ConfigError(String), }
```

Error type for parsing pipeline operations

**Variants**:

- `SemaphoreError(()`
- `ChannelError`
- `ParseError(#[from] ParseError)`
- `ConfigError(String)`

---

### PoolStatus

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub enum PoolStatus { /// Initial funding phase where liquidity is being raised #[default] Fund, /// Migration phase where pool is transitioning to DEX Migrate, /// Active trading phase where tokens can be traded Trade, }
```

Current status of a liquidity pool in the BONK protocol

**Variants**:

- `Fund`
- `Migrate`
- `Trade`

---

### Protocol

**Source**: `events/factory.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
```

```rust
pub enum Protocol { /// Orca Whirlpool concentrated liquidity protocol OrcaWhirlpool, /// Meteora Dynamic Liquidity Market Maker protocol MeteoraDlmm, /// MarginFi lending and borrowing protocol MarginFi, /// Jupiter swap aggregator protocol Jupiter, /// Raydium Automated Market Maker V4 protocol RaydiumAmmV4, /// Raydium Concentrated Liquidity Market Maker protocol RaydiumClmm, /// Raydium Constant Product Market Maker protocol RaydiumCpmm, /// PumpFun meme token creation protocol PumpFun, /// PumpSwap trading protocol PumpSwap, /// Bonk token protocol Bonk, /// Custom protocol with arbitrary name Custom(String), }
```

Protocol enum for supported protocols

**Variants**:

- `OrcaWhirlpool`
- `MeteoraDlmm`
- `MarginFi`
- `Jupiter`
- `RaydiumAmmV4`
- `RaydiumClmm`
- `RaydiumCpmm`
- `PumpFun`
- `PumpSwap`
- `Bonk`
- `Custom(String)`

---

### ProtocolType

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
```

```rust
pub enum ProtocolType { /// Orca Whirlpool concentrated liquidity pools OrcaWhirlpool, /// Meteora Dynamic Liquidity Market Maker MeteoraDlmm, /// MarginFi lending protocol MarginFi, /// Bonk decentralized exchange Bonk, /// PumpSwap automated market maker PumpSwap, /// Raydium automated market maker RaydiumAmm, /// Raydium AMM V4 implementation RaydiumAmmV4, /// Raydium concentrated liquidity market maker RaydiumClmm, /// Raydium constant product market maker RaydiumCpmm, /// Jupiter aggregator protocol Jupiter, /// Other protocol not explicitly supported Other(String), }
```

Enumeration of supported Solana DeFi protocols

**Variants**:

- `OrcaWhirlpool`
- `MeteoraDlmm`
- `MarginFi`
- `Bonk`
- `PumpSwap`
- `RaydiumAmm`
- `RaydiumAmmV4`
- `RaydiumClmm`
- `RaydiumCpmm`
- `Jupiter`
- `Other(String)`

---

### PumpFunDiscriminator

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum PumpFunDiscriminator { /// Buy tokens instruction Buy, /// Sell tokens instruction Sell, /// Create new pool instruction CreatePool, /// Deposit liquidity instruction Deposit, /// Withdraw liquidity instruction Withdraw, /// Set pool parameters instruction SetParams, }
```

PumpFun instruction discriminators

**Variants**:

- `Buy`
- `Sell`
- `CreatePool`
- `Deposit`
- `Withdraw`
- `SetParams`

---

### RaydiumV4Discriminator

**Source**: `parsers/raydium_v4.rs`

**Attributes**:
```rust
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
```

```rust
pub enum RaydiumV4Discriminator { /// Swap with base input amount (discriminator 0x09) SwapBaseIn = 0x09, /// Swap with base output amount (discriminator 0x0a) SwapBaseOut = 0x0a, /// Deposit liquidity to pool (discriminator 0x03) Deposit = 0x03, /// Withdraw liquidity from pool (discriminator 0x04) Withdraw = 0x04, /// Initialize pool version 2 (discriminator 0x00) Initialize2 = 0x00, /// Withdraw profit and loss (discriminator 0x05) WithdrawPnl = 0x05, }
```

Raydium AMM V4 instruction discriminators

**Variants**:

- `SwapBaseIn`
- `SwapBaseOut`
- `Deposit`
- `Withdraw`
- `Initialize2`
- `WithdrawPnl`

---

### SwapDirection

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
```

```rust
pub enum SwapDirection { /// Swap from token A to token B AtoB, /// Swap from token B to token A BtoA, }
```

Orca swap direction

**Variants**:

- `AtoB`
- `BtoA`

---

### SwapDirection

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
```

```rust
pub enum SwapDirection { /// Swapping base token in for quote token out #[default] BaseIn, /// Swapping quote token in for base token out BaseOut, }
```

Direction of the swap operation

**Variants**:

- `BaseIn`
- `BaseOut`

---

### TradeDirection

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub enum TradeDirection { /// Buy operation - purchasing tokens #[default] Buy, /// Sell operation - selling tokens Sell, }
```

Direction of a trade operation in the BONK protocol

**Variants**:

- `Buy`
- `Sell`

---

### ValidationError

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum ValidationError { /// Missing required field MissingField { /// The name of the missing field field: String }, /// Invalid field value InvalidValue { /// The name of the invalid field field: String, /// The reason why the value is invalid reason: String }, /// Data inconsistency Inconsistency { /// Description of the inconsistency description: String }, /// Business logic violation BusinessLogicError { /// The business rule that was violated rule: String, /// Description of the violation description: String }, /// Event too old StaleEvent { /// How old the event is age: Duration }, /// Duplicate event detected Duplicate { /// ID of the original event original_id: String }, }
```

Validation error types

**Variants**:

- `MissingField`
- `field`
- `InvalidValue`
- `field`
- `reason`
- `Inconsistency`
- `description`
- `BusinessLogicError`
- `rule`
- `description`
- `StaleEvent`
- `age`
- `Duplicate`
- `original_id`

---

### ValidationWarning

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum ValidationWarning { /// Unusual but potentially valid value UnusualValue { /// The name of the field with unusual value field: String, /// The unusual value value: String }, /// Deprecated field usage DeprecatedField { /// The name of the deprecated field field: String }, /// Performance concern PerformanceWarning { /// Description of the performance concern description: String }, }
```

Validation warning types

**Variants**:

- `UnusualValue`
- `field`
- `value`
- `DeprecatedField`
- `field`
- `PerformanceWarning`
- `description`

---

## Functions (error)

### invalid_account_index

**Source**: `src/error.rs`

```rust
pub fn invalid_account_index(index: usize, max: usize) -> Self
```

Create an InvalidAccountIndex error

---

### invalid_discriminator

**Source**: `src/error.rs`

```rust
pub fn invalid_discriminator(expected: Vec<u8>, found: Vec<u8>) -> Self
```

Create an InvalidDiscriminator error

---

### invalid_enum_variant

**Source**: `src/error.rs`

```rust
pub fn invalid_enum_variant(variant: u8, type_name: &str) -> Self
```

Create an InvalidEnumVariant error

---

### not_enough_bytes

**Source**: `src/error.rs`

```rust
pub fn not_enough_bytes(expected: usize, found: usize, offset: usize) -> Self
```

Create a NotEnoughBytes error

---

## Functions (utils)

### calculate_price_impact

**Source**: `common/utils.rs`

```rust
pub fn calculate_price_impact( amount_in: u64, amount_out: u64, reserve_in: u64, reserve_out: u64, ) -> f64
```

Calculate price impact for a swap

---

### decode_base58

**Source**: `common/utils.rs`

```rust
pub fn decode_base58(data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
```

Decode base58 string to bytes

---

### discriminator_matches

**Source**: `src/utils.rs`

```rust
pub fn discriminator_matches(data: &str, discriminator: &str) -> bool
```

Checks if a hex data string starts with a given hex discriminator

Both `data` and `discriminator` must be valid hex strings starting with "0x".
Returns `true` if the data string begins with the discriminator string.

# Arguments

* `data` - The hex data string to check
* `discriminator` - The hex discriminator to match against

# Returns

`true` if the data starts with the discriminator, `false` otherwise

---

### encode_base58

**Source**: `common/utils.rs`

```rust
pub fn encode_base58(data: &[u8]) -> String
```

Encode bytes to base58 string

---

### extract_account_keys

**Source**: `common/utils.rs`

```rust
pub fn extract_account_keys( accounts: &[Pubkey], indices: &[usize], ) -> Result<Vec<Pubkey>, Box<dyn std::error::Error + Send + Sync>>
```

Extract account keys from accounts array

---

### extract_discriminator

**Source**: `common/utils.rs`

```rust
pub fn extract_discriminator(data: &[u8], size: usize) -> Option<Vec<u8>>
```

Extract discriminator from instruction data

---

### format_token_amount

**Source**: `common/utils.rs`

```rust
pub fn format_token_amount(amount: u64, decimals: u8) -> f64
```

Convert amount with decimals to human-readable format

---

### has_discriminator

**Source**: `common/utils.rs`

```rust
pub fn has_discriminator(data: &[u8], discriminator: &[u8]) -> bool
```

Check if instruction data starts with a specific discriminator

---

### parse_pubkey_from_bytes

**Source**: `common/utils.rs`

```rust
pub fn parse_pubkey_from_bytes(bytes: &[u8]) -> ParseResult<Pubkey>
```

Parse a pubkey from bytes

---

### parse_u128_le

**Source**: `common/utils.rs`

```rust
pub fn parse_u128_le(bytes: &[u8]) -> ParseResult<u128>
```

Parse a u128 from little-endian bytes

---

### parse_u16_le

**Source**: `common/utils.rs`

```rust
pub fn parse_u16_le(bytes: &[u8]) -> ParseResult<u16>
```

Parse a u16 from little-endian bytes

---

### parse_u32_le

**Source**: `common/utils.rs`

```rust
pub fn parse_u32_le(bytes: &[u8]) -> ParseResult<u32>
```

Parse a u32 from little-endian bytes

---

### parse_u64_le

**Source**: `common/utils.rs`

```rust
pub fn parse_u64_le(bytes: &[u8]) -> ParseResult<u64>
```

Parse a u64 from little-endian bytes

---

### read_i32_le

**Source**: `common/utils.rs`

```rust
pub fn read_i32_le(data: &[u8], offset: usize) -> ParseResult<i32>
```

Read i32 from little-endian bytes at offset

---

### read_option_bool

**Source**: `common/utils.rs`

```rust
pub fn read_option_bool(data: &[u8], offset: &mut usize) -> ParseResult<Option<bool>>
```

Read optional bool from bytes at offset

---

### read_u128_le

**Source**: `common/utils.rs`

```rust
pub fn read_u128_le(data: &[u8], offset: usize) -> ParseResult<u128>
```

Read u128 from little-endian bytes at offset

---

### read_u32_le

**Source**: `common/utils.rs`

```rust
pub fn read_u32_le(data: &[u8], offset: usize) -> ParseResult<u32>
```

Read u32 from little-endian bytes at offset

---

### read_u64_le

**Source**: `common/utils.rs`

```rust
pub fn read_u64_le(data: &[u8], offset: usize) -> ParseResult<u64>
```

Common utility functions for event parsing

Read u64 from little-endian bytes at offset

---

### read_u8_le

**Source**: `common/utils.rs`

```rust
pub fn read_u8_le(data: &[u8], offset: usize) -> ParseResult<u8>
```

Read u8 from bytes at offset

---

### system_time_to_millis

**Source**: `common/utils.rs`

```rust
pub fn system_time_to_millis(time: std::time::SystemTime) -> u64
```

Convert SystemTime to milliseconds since epoch

---

### to_token_amount

**Source**: `common/utils.rs`

```rust
pub fn to_token_amount(amount: f64, decimals: u8) -> u64
```

Convert human-readable amount to token amount with decimals

---

## Structs

### AccountMeta

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct AccountMeta { /// Public key of the account pub pubkey: Pubkey, /// Whether the account is a signer pub is_signer: bool, /// Whether the account is writable pub is_writable: bool, }
```

Account metadata for swap

---

### BatchEventParser

**Source**: `zero_copy/parsers.rs`

```rust
pub struct BatchEventParser { /// Protocol-specific parsers indexed by protocol type parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>, /// SIMD pattern matcher for fast protocol detection pattern_matcher: SIMDPatternMatcher, /// Maximum number of transactions to process in a single batch max_batch_size: usize, }
```

Batch processor for efficient parsing of multiple transactions

---

### BatchParserStats

**Source**: `zero_copy/parsers.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct BatchParserStats { /// Number of registered protocol parsers pub registered_parsers: usize, /// Maximum batch size configured pub max_batch_size: usize, /// Number of discriminator patterns registered pub pattern_count: usize, }
```

Statistics for batch parser performance monitoring

---

### BonkEventParser

**Source**: `bonk/parser.rs`

```rust
pub struct BonkEventParser { inner: GenericEventParser, }
```

Bonk event parser

---

### BonkPoolCreateEvent

**Source**: `bonk/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct BonkPoolCreateEvent { /// Event metadata #[serde(skip)]
```

Create pool event

---

### BonkTradeEvent

**Source**: `bonk/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct BonkTradeEvent { /// Event metadata #[serde(skip)]
```

Trade event

---

### BurnNftInstruction

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct BurnNftInstruction { /// Discriminator pub discriminator: u8, }
```

Metaplex BurnNft instruction data

---

### CacheStats

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct CacheStats { /// Number of entries in token metadata cache pub token_cache_size: usize, /// Number of entries in price data cache pub price_cache_size: usize, /// Token cache hit rate (0.0 to 1.0)
```

Cache statistics

---

### ConstantCurve

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct ConstantCurve { /// Total token supply for the curve pub supply: u64, /// Total base tokens available for selling pub total_base_sell: u64, /// Total quote tokens raised during funding pub total_quote_fund_raising: u64, /// Type of migration when curve completes pub migrate_type: u8, }
```

Constant product bonding curve parameters

---

### CreateMetadataAccountInstruction

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct CreateMetadataAccountInstruction { /// Discriminator pub discriminator: u8, /// Metadata account bump pub metadata_account_bump: u8, /// NFT name pub name: String, /// NFT symbol pub symbol: String, /// NFT URI (metadata JSON)
```

Metaplex CreateMetadataAccount instruction data

---

### CurveParams

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct CurveParams { /// Constant product curve configuration, if used pub constant_curve: Option<ConstantCurve>, /// Fixed price curve configuration, if used pub fixed_curve: Option<FixedCurve>, /// Linear price curve configuration, if used pub linear_curve: Option<LinearCurve>, }
```

Container for different bonding curve configurations

---

### CustomDeserializer

**Source**: `zero_copy/parsers.rs`

```rust
pub struct CustomDeserializer<'a> { /// Byte data being deserialized data: &'a [u8], /// Current read position in the data pos: usize, }
```

Custom deserializer for hot path parsing

---

### DepositInstruction

**Source**: `parsers/raydium_v4.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct DepositInstruction { /// Discriminator (should be 0x03)
```

Raydium V4 Deposit instruction data

---

### DlmmBin

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct DlmmBin { /// Unique identifier for this price bin pub bin_id: u32, /// Amount of token X reserves in this bin pub reserve_x: u64, /// Amount of token Y reserves in this bin pub reserve_y: u64, /// Current price for this bin pub price: f64, /// Total liquidity token supply for this bin pub liquidity_supply: u128, }
```

Meteora DLMM bin information

---

### DlmmPairConfig

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct DlmmPairConfig { /// Public key of the DLMM pair account pub pair: Pubkey, /// Mint address of token X pub token_mint_x: Pubkey, /// Mint address of token Y pub token_mint_y: Pubkey, /// Step size between bins in basis points pub bin_step: u16, /// Base fee percentage charged for swaps pub base_fee_percentage: u64, /// Maximum fee percentage that can be charged pub max_fee_percentage: u64, /// Protocol fee percentage taken from trades pub protocol_fee_percentage: u64, /// Liquidity provider fee percentage pub liquidity_fee_percentage: u64, /// Current volatility accumulator value pub volatility_accumulator: u32, /// Volatility reference point pub volatility_reference: u32, /// ID reference for bin tracking pub id_reference: u32, /// Timestamp of last pair update pub time_of_last_update: u64, /// Currently active bin ID pub active_id: u32, /// Base key for the pair pub base_key: Pubkey, }
```

Meteora DLMM pair configuration

---

### EnrichmentConfig

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EnrichmentConfig { /// Enable token metadata enrichment pub enable_token_metadata: bool, /// Enable price data enrichment pub enable_price_data: bool, /// Enable transaction context enrichment pub enable_transaction_context: bool, /// Cache TTL for metadata pub metadata_cache_ttl: Duration, /// Maximum cache size pub max_cache_size: usize, /// Timeout for external API calls pub api_timeout: Duration, }
```

Configuration for event enrichment

---

### EventEnricher

**Source**: `pipelines/enrichment.rs`

```rust
pub struct EventEnricher { /// Enrichment configuration config: EnrichmentConfig, /// Token metadata cache token_cache: Arc<DashMap<Pubkey, CacheEntry<TokenMetadata>>>, /// Price data cache price_cache: Arc<DashMap<Pubkey, CacheEntry<PriceData>>>, /// HTTP client for external API calls http_client: reqwest::Client, }
```

Event enricher with caching and external API integration

---

### EventParameters

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct EventParameters { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Event index within the transaction pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct EventParameters { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Index of the instruction within the transaction pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct EventParameters { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Event index within the transaction pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct EventParameters { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Event index within the transaction pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParserRegistry

**Source**: `events/factory.rs`

**Attributes**:
```rust
#[derive(Default)]
```

```rust
pub struct EventParserRegistry { /// Map of protocols to their respective event parsers parsers: HashMap<Protocol, Arc<dyn EventParser>>, /// Map of program IDs to their respective event parsers for fast lookup program_id_to_parser: HashMap<solana_sdk::pubkey::Pubkey, Arc<dyn EventParser>>, }
```

EventParserRegistry - the new async event parser registry

---

### FixedCurve

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct FixedCurve { /// Total token supply for the curve pub supply: u64, /// Total quote tokens to be raised during funding pub total_quote_fund_raising: u64, /// Type of migration when curve completes pub migrate_type: u8, }
```

Fixed price bonding curve parameters

---

### GenericEventParseConfig

**Source**: `core/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GenericEventParseConfig { /// Program ID this configuration applies to pub program_id: solana_sdk::pubkey::Pubkey, /// Protocol type for events generated from this configuration pub protocol_type: ProtocolType, /// Discriminator string for inner instructions pub inner_instruction_discriminator: &'static str, /// Discriminator bytes for instructions pub instruction_discriminator: &'static [u8], /// Type of events this configuration generates pub event_type: EventType, /// Parser function for inner instructions pub inner_instruction_parser: InnerInstructionEventParser, /// Parser function for instructions pub instruction_parser: InstructionEventParser, }
```

Generic event parser configuration

---

### GenericEventParser

**Source**: `core/traits.rs`

```rust
pub struct GenericEventParser { /// List of program IDs this parser handles pub program_ids: Vec<solana_sdk::pubkey::Pubkey>, /// Configuration mapping for inner instruction parsing by discriminator pub inner_instruction_configs: std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>, /// Configuration mapping for instruction parsing by discriminator bytes pub instruction_configs: std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
```

Generic event parser base class

---

### InnerInstructionParseParams

**Source**: `events/factory.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct InnerInstructionParseParams<'a> { /// Inner instruction data from transaction metadata pub inner_instruction: &'a solana_transaction_status::UiCompiledInstruction, /// Transaction signature pub signature: &'a str, /// Solana slot number pub slot: u64, /// Block time (optional)
```

Parameters for parsing events from inner instructions, reducing function parameter count

---

### InstructionParseParams

**Source**: `events/factory.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct InstructionParseParams<'a> { /// Compiled instruction data pub instruction: &'a solana_sdk::instruction::CompiledInstruction, /// Account keys from the transaction pub accounts: &'a [solana_sdk::pubkey::Pubkey], /// Transaction signature pub signature: &'a str, /// Solana slot number pub slot: u64, /// Block time (optional)
```

Parameters for parsing events from instructions, reducing function parameter count

---

### JupiterAccountLayout

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct JupiterAccountLayout { /// User's transfer authority pub user_transfer_authority: Pubkey, /// User's source token account pub user_source_token_account: Pubkey, /// User's destination token account pub user_destination_token_account: Pubkey, /// Destination token account for the swap pub destination_token_account: Pubkey, /// Source token mint address pub source_mint: Pubkey, /// Destination token mint address pub destination_mint: Pubkey, /// Optional platform fee account pub platform_fee_account: Option<Pubkey>, }
```

Jupiter program account layout

---

### JupiterEventParser

**Source**: `jupiter/parser.rs`

```rust
pub struct JupiterEventParser { program_ids: Vec<Pubkey>, inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>, instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
```

Jupiter event parser

---

### JupiterExactOutRouteInstruction

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct JupiterExactOutRouteInstruction { /// Maximum amount in pub max_amount_in: u64, /// Exact amount out desired pub amount_out: u64, /// Platform fee basis points pub platform_fee_bps: u16, }
```

Jupiter ExactOutRoute instruction data

---

### JupiterLiquidityEvent

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct JupiterLiquidityEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time spent handling the event in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// User account providing/removing liquidity pub user: solana_sdk::pubkey::Pubkey, /// First token mint address pub mint_a: solana_sdk::pubkey::Pubkey, /// Second token mint address pub mint_b: solana_sdk::pubkey::Pubkey, /// Amount of token A pub amount_a: u64, /// Amount of token B pub amount_b: u64, /// Amount of liquidity tokens pub liquidity_amount: u64, /// Whether this is a liquidity removal operation pub is_remove: bool, /// Associated token transfer data pub transfer_data: Vec<TransferData>, }
```

Jupiter liquidity provision event

---

### JupiterParser

**Source**: `parsers/jupiter.rs`

```rust
pub struct JupiterParser { /// Program ID for validation #[allow(dead_code)]
```

High-performance Jupiter parser

---

### JupiterParserFactory

**Source**: `parsers/jupiter.rs`

```rust
pub struct JupiterParserFactory;
```

Factory for creating Jupiter parsers

---

### JupiterRouteAnalysis

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct JupiterRouteAnalysis { /// Instruction type pub instruction_type: String, /// Total amount in pub total_amount_in: u64, /// Total amount out (minimum or exact)
```

Jupiter route analysis result

---

### JupiterRouteData

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct JupiterRouteData { /// The parsed instruction data pub instruction: JupiterRouteInstruction, /// The route analysis result pub analysis: JupiterRouteAnalysis, }
```

Combined Jupiter Route Data (instruction + analysis)

---

### JupiterRouteInstruction

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct JupiterRouteInstruction { /// Amount to swap in pub amount_in: u64, /// Minimum amount out expected pub minimum_amount_out: u64, /// Platform fee basis points pub platform_fee_bps: u16, }
```

Jupiter Route instruction data

---

### JupiterSwapBorshEvent

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct JupiterSwapBorshEvent { /// Event metadata (skipped during serialization)
```

Jupiter swap event with borsh (for simple events)

---

### JupiterSwapData

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct JupiterSwapData { /// User who initiated the swap pub user: Pubkey, /// Input token mint address pub input_mint: Pubkey, /// Output token mint address pub output_mint: Pubkey, /// Input amount in base units pub input_amount: u64, /// Output amount in base units pub output_amount: u64, /// Price impact percentage as string pub price_impact_pct: Option<String>, /// Platform fee in basis points pub platform_fee_bps: Option<u32>, /// Route plan for the swap pub route_plan: Vec<RoutePlan>, }
```

Jupiter swap event data (for UnifiedEvent)

---

### JupiterSwapEvent

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct JupiterSwapEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time spent handling the event in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Jupiter-specific swap data pub swap_data: JupiterSwapData, /// Associated token transfer data pub transfer_data: Vec<TransferData>, }
```

Jupiter swap event

---

### LinearCurve

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct LinearCurve { /// Total token supply for the curve pub supply: u64, /// Total quote tokens to be raised during funding pub total_quote_fund_raising: u64, /// Type of migration when curve completes pub migrate_type: u8, }
```

Linear bonding curve parameters

---

### LiquidityEventParams

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct LiquidityEventParams { /// Event identifier pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp pub block_time: i64, /// Protocol type (e.g., Orca, Raydium)
```

Parameters for creating liquidity events, reducing function parameter count

---

### MarginFiAccount

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiAccount { /// MarginFi group this account belongs to pub group: Pubkey, /// Authority that controls this account pub authority: Pubkey, /// Lending account with balance information pub lending_account: MarginFiLendingAccount, /// Account configuration flags pub account_flags: u64, /// Padding for future use pub padding: [u128; 8], }
```

MarginFi user account

---

### MarginFiBalance

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiBalance { /// Whether this balance is active pub active: bool, /// Bank public key for this balance pub bank_pk: Pubkey, /// Shares representing deposited assets pub asset_shares: u128, /// Shares representing borrowed liabilities pub liability_shares: u128, /// Outstanding emission rewards pub emissions_outstanding: u64, /// Timestamp of last balance update pub last_update: u64, /// Padding for future use pub padding: [u64; 1], }
```

MarginFi account balance

---

### MarginFiBankConfig

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiBankConfig { /// Bank account public key pub bank: Pubkey, /// Token mint for this bank pub mint: Pubkey, /// Liquidity vault holding deposited tokens pub vault: Pubkey, /// Price oracle for this token pub oracle: Pubkey, /// Authority that can modify bank settings pub bank_authority: Pubkey, /// Outstanding insurance fees collected pub collected_insurance_fees_outstanding: u64, /// Fee rate charged on operations pub fee_rate: u64, /// Insurance fee rate for risk coverage pub insurance_fee_rate: u64, /// Vault holding insurance funds pub insurance_vault: Pubkey, /// Maximum amount that can be deposited pub deposit_limit: u64, /// Maximum amount that can be borrowed pub borrow_limit: u64, /// Current operational state of the bank pub operational_state: u8, /// Oracle configuration setup pub oracle_setup: u8, /// Array of oracle public keys pub oracle_keys: [Pubkey; 5], }
```

MarginFi bank configuration

---

### MarginFiBankState

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiBankState { /// Total shares representing assets in the bank pub total_asset_shares: u128, /// Total shares representing liabilities in the bank pub total_liability_shares: u128, /// Timestamp of last state update pub last_update: u64, /// Current lending interest rate pub lending_rate: u64, /// Current borrowing interest rate pub borrowing_rate: u64, /// Value per asset share pub asset_share_value: u128, /// Value per liability share pub liability_share_value: u128, /// Authority for the liquidity vault pub liquidity_vault_authority: Pubkey, /// Bump seed for liquidity vault authority pub liquidity_vault_authority_bump: u8, /// Authority for the insurance vault pub insurance_vault_authority: Pubkey, /// Bump seed for insurance vault authority pub insurance_vault_authority_bump: u8, /// Outstanding group fees collected pub collected_group_fees_outstanding: u64, /// Authority for the fee vault pub fee_vault_authority: Pubkey, /// Bump seed for fee vault authority pub fee_vault_authority_bump: u8, /// Fee collection vault pub fee_vault: Pubkey, }
```

MarginFi bank state

---

### MarginFiBorrowData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiBorrowData { /// MarginFi group for the borrow pub marginfi_group: Pubkey, /// User's MarginFi account pub marginfi_account: Pubkey, /// Transaction signer pub signer: Pubkey, /// Bank from which tokens are borrowed pub bank: Pubkey, /// User's token account being credited pub token_account: Pubkey, /// Bank's liquidity vault being debited pub bank_liquidity_vault: Pubkey, /// Authority for the bank's liquidity vault pub bank_liquidity_vault_authority: Pubkey, /// Token program ID pub token_program: Pubkey, /// Amount being borrowed pub amount: u64, }
```

MarginFi borrow data

---

### MarginFiBorrowEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiBorrowEvent { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Time spent handling the transaction in milliseconds pub program_handle_time_consuming_ms: i64, /// Index of the instruction within the transaction pub index: String, /// MarginFi-specific borrow operation data pub borrow_data: MarginFiBorrowData, /// Associated token transfer data for this borrow pub transfer_data: Vec<TransferData>, /// Event metadata for unified event handling #[serde(skip)]
```

MarginFi borrow event

---

### MarginFiDepositData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiDepositData { /// MarginFi group for the deposit pub marginfi_group: Pubkey, /// User's MarginFi account pub marginfi_account: Pubkey, /// Transaction signer pub signer: Pubkey, /// Bank receiving the deposit pub bank: Pubkey, /// User's token account being debited pub token_account: Pubkey, /// Bank's liquidity vault being credited pub bank_liquidity_vault: Pubkey, /// Token program ID pub token_program: Pubkey, /// Amount being deposited pub amount: u64, }
```

MarginFi deposit data

---

### MarginFiDepositEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiDepositEvent { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Time spent handling the transaction in milliseconds pub program_handle_time_consuming_ms: i64, /// Index of the instruction within the transaction pub index: String, /// MarginFi-specific deposit operation data pub deposit_data: MarginFiDepositData, /// Associated token transfer data for this deposit pub transfer_data: Vec<TransferData>, /// Event metadata for unified event handling #[serde(skip)]
```

MarginFi deposit event

---

### MarginFiEventParser

**Source**: `marginfi/parser.rs`

```rust
pub struct MarginFiEventParser { program_ids: Vec<Pubkey>, inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>, instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
```

MarginFi event parser

---

### MarginFiLendingAccount

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiLendingAccount { /// Array of balances for different tokens pub balances: [MarginFiBalance; 16], /// Padding for future use pub padding: [u64; 8], }
```

MarginFi lending account

---

### MarginFiLiquidationData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiLiquidationData { /// MarginFi group for the liquidation pub marginfi_group: Pubkey, /// Bank holding the asset being seized pub asset_bank: Pubkey, /// Bank holding the liability being repaid pub liab_bank: Pubkey, /// Account being liquidated pub liquidatee_marginfi_account: Pubkey, /// Liquidator's MarginFi account pub liquidator_marginfi_account: Pubkey, /// Liquidator's wallet address pub liquidator: Pubkey, /// Asset bank's liquidity vault pub asset_bank_liquidity_vault: Pubkey, /// Liability bank's liquidity vault pub liab_bank_liquidity_vault: Pubkey, /// Liquidator's token account pub liquidator_token_account: Pubkey, /// Token program ID pub token_program: Pubkey, /// Amount of asset being seized pub asset_amount: u64, /// Amount of liability being repaid pub liab_amount: u64, }
```

MarginFi liquidation data

---

### MarginFiLiquidationEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiLiquidationEvent { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Time spent handling the transaction in milliseconds pub program_handle_time_consuming_ms: i64, /// Index of the instruction within the transaction pub index: String, /// MarginFi-specific liquidation operation data pub liquidation_data: MarginFiLiquidationData, /// Associated token transfer data for this liquidation pub transfer_data: Vec<TransferData>, /// Event metadata for unified event handling #[serde(skip)]
```

MarginFi liquidation event

---

### MarginFiRepayData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiRepayData { /// MarginFi group for the repayment pub marginfi_group: Pubkey, /// User's MarginFi account pub marginfi_account: Pubkey, /// Transaction signer pub signer: Pubkey, /// Bank to which tokens are repaid pub bank: Pubkey, /// User's token account being debited pub token_account: Pubkey, /// Bank's liquidity vault being credited pub bank_liquidity_vault: Pubkey, /// Token program ID pub token_program: Pubkey, /// Amount being repaid pub amount: u64, /// Whether to repay all outstanding debt pub repay_all: bool, }
```

MarginFi repay data

---

### MarginFiRepayEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiRepayEvent { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Time spent handling the transaction in milliseconds pub program_handle_time_consuming_ms: i64, /// Index of the instruction within the transaction pub index: String, /// MarginFi-specific repay operation data pub repay_data: MarginFiRepayData, /// Associated token transfer data for this repayment pub transfer_data: Vec<TransferData>, /// Event metadata for unified event handling #[serde(skip)]
```

MarginFi repay event

---

### MarginFiWithdrawData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiWithdrawData { /// MarginFi group for the withdrawal pub marginfi_group: Pubkey, /// User's MarginFi account pub marginfi_account: Pubkey, /// Transaction signer pub signer: Pubkey, /// Bank from which tokens are withdrawn pub bank: Pubkey, /// User's token account being credited pub token_account: Pubkey, /// Bank's liquidity vault being debited pub bank_liquidity_vault: Pubkey, /// Authority for the bank's liquidity vault pub bank_liquidity_vault_authority: Pubkey, /// Token program ID pub token_program: Pubkey, /// Amount being withdrawn pub amount: u64, /// Whether to withdraw all available tokens pub withdraw_all: bool, }
```

MarginFi withdraw data

---

### MarginFiWithdrawEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MarginFiWithdrawEvent { /// Unique identifier for the event pub id: String, /// Transaction signature hash pub signature: String, /// Solana slot number when the transaction was processed pub slot: u64, /// Block timestamp in seconds since Unix epoch pub block_time: i64, /// Block timestamp in milliseconds since Unix epoch pub block_time_ms: i64, /// Timestamp when the program received the transaction in milliseconds pub program_received_time_ms: i64, /// Time spent handling the transaction in milliseconds pub program_handle_time_consuming_ms: i64, /// Index of the instruction within the transaction pub index: String, /// MarginFi-specific withdraw operation data pub withdraw_data: MarginFiWithdrawData, /// Associated token transfer data for this withdrawal pub transfer_data: Vec<TransferData>, /// Event metadata for unified event handling #[serde(skip)]
```

MarginFi withdraw event

---

### MemoryMappedParser

**Source**: `zero_copy/parsers.rs`

```rust
pub struct MemoryMappedParser { /// Memory-mapped file handle mmap: memmap2::Mmap, /// Protocol-specific parsers indexed by protocol type parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>, }
```

High-performance parser using memory-mapped files for large transaction logs

---

### MetaplexEventAnalysis

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct MetaplexEventAnalysis { /// Event category (mint, transfer, burn, etc.)
```

Metaplex marketplace event analysis

---

### MetaplexParser

**Source**: `parsers/metaplex.rs`

```rust
pub struct MetaplexParser { /// Token Metadata program ID #[allow(dead_code)]
```

High-performance Metaplex parser

---

### MetaplexParserFactory

**Source**: `parsers/metaplex.rs`

```rust
pub struct MetaplexParserFactory;
```

Factory for creating Metaplex parsers

---

### MeteoraDynamicLiquidityData

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraDynamicLiquidityData { /// Public key of the Dynamic AMM pool pub pool: Pubkey, /// Public key of the user adding/removing liquidity pub user: Pubkey, /// Mint address of token A pub token_mint_a: Pubkey, /// Mint address of token B pub token_mint_b: Pubkey, /// Vault account holding token A reserves pub vault_a: Pubkey, /// Vault account holding token B reserves pub vault_b: Pubkey, /// Mint address of the LP tokens pub lp_mint: Pubkey, /// Amount of pool tokens being minted or burned pub pool_token_amount: u64, /// Amount of token A being deposited or withdrawn pub token_a_amount: u64, /// Amount of token B being deposited or withdrawn pub token_b_amount: u64, /// Minimum acceptable pool token amount for slippage protection pub minimum_pool_token_amount: u64, /// Maximum token A amount willing to deposit pub maximum_token_a_amount: u64, /// Maximum token B amount willing to deposit pub maximum_token_b_amount: u64, /// Whether this is a deposit (true) or withdrawal (false) operation
```

Meteora Dynamic liquidity data

---

### MeteoraDynamicLiquidityEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraDynamicLiquidityEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time consumed by program handling in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Meteora dynamic liquidity-specific data pub liquidity_data: MeteoraDynamicLiquidityData, /// Token transfer data associated with the liquidity operation pub transfer_data: Vec<TransferData>, /// Event metadata for cross-chain compatibility #[serde(skip)]
```

Meteora Dynamic AMM liquidity event

---

### MeteoraDynamicPoolData

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraDynamicPoolData { /// Public key of the Dynamic AMM pool pub pool: Pubkey, /// Mint address of token A pub token_mint_a: Pubkey, /// Mint address of token B pub token_mint_b: Pubkey, /// Vault account holding token A reserves pub vault_a: Pubkey, /// Vault account holding token B reserves pub vault_b: Pubkey, /// Mint address of the LP tokens pub lp_mint: Pubkey, /// Base fee rate for the pool pub fee_rate: u64, /// Administrative fee rate pub admin_fee_rate: u64, /// Numerator for trade fee calculation pub trade_fee_numerator: u64, /// Denominator for trade fee calculation pub trade_fee_denominator: u64, /// Numerator for owner trade fee calculation pub owner_trade_fee_numerator: u64, /// Denominator for owner trade fee calculation pub owner_trade_fee_denominator: u64, /// Numerator for owner withdraw fee calculation pub owner_withdraw_fee_numerator: u64, /// Denominator for owner withdraw fee calculation pub owner_withdraw_fee_denominator: u64, /// Numerator for host fee calculation pub host_fee_numerator: u64, /// Denominator for host fee calculation pub host_fee_denominator: u64, }
```

Meteora Dynamic AMM pool data

---

### MeteoraEventParser

**Source**: `meteora/parser.rs`

```rust
pub struct MeteoraEventParser { program_ids: Vec<Pubkey>, inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>, instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
```

Meteora event parser

---

### MeteoraLiquidityData

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraLiquidityData { /// Public key of the DLMM pair pub pair: Pubkey, /// Public key of the user adding/removing liquidity pub user: Pubkey, /// Public key of the liquidity position pub position: Pubkey, /// Mint address of token X pub token_mint_x: Pubkey, /// Mint address of token Y pub token_mint_y: Pubkey, /// Reserve account for token X pub reserve_x: Pubkey, /// Reserve account for token Y pub reserve_y: Pubkey, /// Starting bin ID for liquidity range pub bin_id_from: u32, /// Ending bin ID for liquidity range pub bin_id_to: u32, /// Amount of token X added or removed pub amount_x: u64, /// Amount of token Y added or removed pub amount_y: u64, /// Amount of liquidity tokens minted or burned pub liquidity_minted: u128, /// Currently active bin ID pub active_id: u32, /// Whether this is an add (true) or remove (false) operation
```

Meteora DLMM liquidity data

---

### MeteoraLiquidityEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraLiquidityEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time consumed by program handling in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Meteora liquidity-specific data pub liquidity_data: MeteoraLiquidityData, /// Token transfer data associated with the liquidity operation pub transfer_data: Vec<TransferData>, /// Event metadata for cross-chain compatibility #[serde(skip)]
```

Meteora DLMM liquidity event

---

### MeteoraSwapData

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraSwapData { /// Public key of the DLMM pair being swapped on pub pair: Pubkey, /// Public key of the user performing the swap pub user: Pubkey, /// Mint address of token X pub token_mint_x: Pubkey, /// Mint address of token Y pub token_mint_y: Pubkey, /// Reserve account for token X pub reserve_x: Pubkey, /// Reserve account for token Y pub reserve_y: Pubkey, /// Amount of tokens being swapped in pub amount_in: u64, /// Minimum expected amount of tokens out pub min_amount_out: u64, /// Actual amount of tokens received pub actual_amount_out: u64, /// Whether swapping X for Y (true) or Y for X (false)
```

Meteora DLMM swap data

---

### MeteoraSwapEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct MeteoraSwapEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time consumed by program handling in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Meteora swap-specific data pub swap_data: MeteoraSwapData, /// Token transfer data associated with the swap pub transfer_data: Vec<TransferData>, /// Event metadata for cross-chain compatibility #[serde(skip)]
```

Meteora DLMM swap event

---

### MintParams

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct MintParams { /// Number of decimal places for the token pub decimals: u8, /// Human-readable name of the token pub name: String, /// Trading symbol for the token pub symbol: String, /// URI pointing to token metadata pub uri: String, }
```

Parameters for minting new tokens in the BONK protocol

---

### OrcaEventParser

**Source**: `orca/parser.rs`

```rust
pub struct OrcaEventParser { program_ids: Vec<Pubkey>, inner_instruction_configs: HashMap<&'static str, Vec<GenericEventParseConfig>>, instruction_configs: HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
```

Orca Whirlpool event parser

---

### OrcaLiquidityData

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaLiquidityData { /// The whirlpool account where liquidity is being modified pub whirlpool: Pubkey, /// The position account being modified pub position: Pubkey, /// Authority account that can modify the position pub position_authority: Pubkey, /// Mint address of the first token in the trading pair pub token_mint_a: Pubkey, /// Mint address of the second token in the trading pair pub token_mint_b: Pubkey, /// Vault account holding token A reserves for the pool pub token_vault_a: Pubkey, /// Vault account holding token B reserves for the pool pub token_vault_b: Pubkey, /// Lower tick boundary of the position's price range pub tick_lower_index: i32, /// Upper tick boundary of the position's price range pub tick_upper_index: i32, /// Amount of liquidity being added or removed pub liquidity_amount: u128, /// Maximum amount of token A willing to deposit/withdraw pub token_max_a: u64, /// Maximum amount of token B willing to deposit/withdraw pub token_max_b: u64, /// Actual amount of token A deposited/withdrawn pub token_actual_a: u64, /// Actual amount of token B deposited/withdrawn pub token_actual_b: u64, /// Whether this is a liquidity increase (true) or decrease (false)
```

Orca liquidity change data

---

### OrcaLiquidityEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaLiquidityEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time spent handling the event in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Orca-specific liquidity data pub liquidity_data: OrcaLiquidityData, /// Associated token transfer data pub transfer_data: Vec<TransferData>, /// Event metadata (excluded from serialization)
```

Orca liquidity event (increase/decrease)

---

### OrcaPositionData

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaPositionData { /// The whirlpool account this position provides liquidity to pub whirlpool: Pubkey, /// Mint address for the NFT representing this liquidity position pub position_mint: Pubkey, /// Account address storing the position's state and parameters pub position: Pubkey, /// Token account holding the position NFT pub position_token_account: Pubkey, /// Authority account that can modify this position pub position_authority: Pubkey, /// Lower tick boundary of the position's price range pub tick_lower_index: i32, /// Upper tick boundary of the position's price range pub tick_upper_index: i32, /// Amount of liquidity provided by this position pub liquidity: u128, /// Fee growth checkpoint for token A when position was last updated pub fee_growth_checkpoint_a: u128, /// Fee growth checkpoint for token B when position was last updated pub fee_growth_checkpoint_b: u128, /// Accumulated fees owed to this position in token A pub fee_owed_a: u64, /// Accumulated fees owed to this position in token B pub fee_owed_b: u64, /// Array of reward information for up to 3 reward tokens pub reward_infos: [PositionRewardInfo; 3], }
```

Orca position data

---

### OrcaPositionEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaPositionEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time spent handling the event in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Orca-specific position data pub position_data: OrcaPositionData, /// Whether the position is being opened (true) or closed (false)
```

Orca position event (open/close)

---

### OrcaSwapData

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaSwapData { /// The whirlpool account where the swap occurred pub whirlpool: Pubkey, /// The user account that initiated the swap transaction pub user: Pubkey, /// Mint address of the first token in the trading pair pub token_mint_a: Pubkey, /// Mint address of the second token in the trading pair pub token_mint_b: Pubkey, /// Vault account holding token A reserves for the pool pub token_vault_a: Pubkey, /// Vault account holding token B reserves for the pool pub token_vault_b: Pubkey, /// The amount specified by the user for the swap operation pub amount: u64, /// Whether the specified amount represents input (true) or output (false)
```

Orca swap event data

---

### OrcaSwapEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
```

```rust
pub struct OrcaSwapEvent { /// Unique identifier for the event pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp in seconds pub block_time: i64, /// Block timestamp in milliseconds pub block_time_ms: i64, /// Time when the program received the event in milliseconds pub program_received_time_ms: i64, /// Time spent handling the event in milliseconds pub program_handle_time_consuming_ms: i64, /// Event index within the transaction pub index: String, /// Orca-specific swap data pub swap_data: OrcaSwapData, /// Associated token transfer data pub transfer_data: Vec<TransferData>, /// Event metadata (excluded from serialization)
```

Orca Whirlpool swap event

---

### ParserConfig

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ParserConfig { /// Enable zero-copy parsing for this protocol pub zero_copy: bool, /// Enable detailed analysis pub detailed_analysis: bool, /// Parser priority (higher numbers = higher priority)
```

Parser-specific configuration

---

### ParserMetrics

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ParserMetrics { /// Time spent parsing pub parse_time: Duration, /// Number of events parsed by this parser pub events_count: usize, /// Success rate (0.0 to 1.0)
```

Parser-specific metrics

---

### ParsingInput

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ParsingInput { /// Raw instruction or transaction data pub data: Vec<u8>, /// Event metadata pub metadata: EventMetadata, /// Optional program ID hint for faster parsing pub program_id_hint: Option<solana_sdk::pubkey::Pubkey>, }
```

Input for the parsing pipeline

---

### ParsingMetrics

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ParsingMetrics { /// Total processing time pub processing_time: Duration, /// Number of events parsed pub events_parsed: usize, /// Number of bytes processed pub bytes_processed: usize, /// Number of parsing errors pub error_count: usize, /// Parser-specific metrics pub parser_metrics: HashMap<ProtocolType, ParserMetrics>, }
```

Metrics for parsing performance

---

### ParsingOutput

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct ParsingOutput { /// Parsed events pub events: Vec<ZeroCopyEvent<'static>>, /// Parsing metrics pub metrics: ParsingMetrics, /// Any errors that occurred during parsing pub errors: Vec<ParseError>, }
```

Output from the parsing pipeline

---

### ParsingPipeline

**Source**: `pipelines/parsing.rs`

```rust
pub struct ParsingPipeline { /// Pipeline configuration config: ParsingPipelineConfig, /// Batch event parser batch_parser: BatchEventParser, /// Semaphore for controlling concurrency semaphore: Arc<Semaphore>, /// Output channel sender output_sender: mpsc::UnboundedSender<ParsingOutput>, /// Output channel receiver output_receiver: mpsc::UnboundedReceiver<ParsingOutput>, }
```

High-performance parsing pipeline

---

### ParsingPipelineBuilder

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Default)]
```

```rust
pub struct ParsingPipelineBuilder { config: ParsingPipelineConfig, parsers: Vec<Arc<dyn ByteSliceEventParser>>, }
```

Builder for creating parsing pipelines

---

### ParsingPipelineConfig

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ParsingPipelineConfig { /// Maximum batch size for processing pub max_batch_size: usize, /// Timeout for batch collection pub batch_timeout: Duration, /// Maximum number of concurrent parsing tasks pub max_concurrent_tasks: usize, /// Buffer size for the output channel pub output_buffer_size: usize, /// Enable performance metrics collection pub enable_metrics: bool, /// Parser-specific configurations pub parser_configs: HashMap<ProtocolType, ParserConfig>, }
```

Configuration for the parsing pipeline

---

### PipelineStats

**Source**: `pipelines/parsing.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct PipelineStats { /// Maximum batch size for processing pub max_batch_size: usize, /// Maximum number of concurrent parsing tasks pub max_concurrent_tasks: usize, /// Number of registered parsers pub registered_parsers: usize, /// Number of available semaphore permits pub available_permits: usize, }
```

Statistics for the parsing pipeline

---

### PositionRewardInfo

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
```

```rust
pub struct PositionRewardInfo { /// Reward growth checkpoint inside the position's tick range pub growth_inside_checkpoint: u128, /// Amount of reward tokens owed to this position pub amount_owed: u64, }
```

Position reward information

---

### PriceData

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct PriceData { /// Token mint address pub mint: Pubkey, /// Price in USD pub usd_price: Option<f64>, /// Price in SOL pub sol_price: Option<f64>, /// 24h volume in USD pub volume_24h: Option<f64>, /// Market cap in USD pub market_cap: Option<f64>, /// When this price data was fetched pub fetched_at: Instant, }
```

Price information for a token

---

### ProtocolEventParams

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ProtocolEventParams { /// Event identifier pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp pub block_time: i64, /// Protocol type (e.g., MarginFi, Meteora)
```

Parameters for creating protocol events, reducing function parameter count

---

### PumpBuyInstruction

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct PumpBuyInstruction { /// 8-byte discriminator pub discriminator: u64, /// Amount to buy (in lamports or token base units)
```

PumpFun Buy instruction data

---

### PumpCreatePoolInstruction

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct PumpCreatePoolInstruction { /// 8-byte discriminator pub discriminator: u64, /// Token name (32 bytes max)
```

PumpFun CreatePool instruction data

---

### PumpDepositInstruction

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct PumpDepositInstruction { /// 8-byte discriminator pub discriminator: u64, /// Amount to deposit pub amount: u64, /// Minimum LP tokens expected pub min_lp_tokens: u64, }
```

PumpFun Deposit instruction data

---

### PumpFunParser

**Source**: `parsers/pump_fun.rs`

```rust
pub struct PumpFunParser { /// Program ID for validation #[allow(dead_code)]
```

High-performance PumpFun parser

---

### PumpFunParserFactory

**Source**: `parsers/pump_fun.rs`

```rust
pub struct PumpFunParserFactory;
```

Factory for creating PumpFun parsers

---

### PumpSellInstruction

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct PumpSellInstruction { /// 8-byte discriminator pub discriminator: u64, /// Amount to sell (in token units)
```

PumpFun Sell instruction data

---

### PumpSwapBuyEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct PumpSwapBuyEvent { /// Event metadata (excluded from serialization)
```

Buy event

---

### PumpSwapCreatePoolEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct PumpSwapCreatePoolEvent { /// Event metadata (excluded from serialization)
```

Create pool event

---

### PumpSwapDepositEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct PumpSwapDepositEvent { /// Event metadata (excluded from serialization)
```

Deposit event

---

### PumpSwapEventParser

**Source**: `pumpswap/parser.rs`

```rust
pub struct PumpSwapEventParser { inner: GenericEventParser, }
```

PumpSwap event parser

---

### PumpSwapSellEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct PumpSwapSellEvent { /// Event metadata (excluded from serialization)
```

Sell event

---

### PumpSwapWithdrawEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct PumpSwapWithdrawEvent { /// Event metadata (excluded from serialization)
```

Withdraw event

---

### PumpWithdrawInstruction

**Source**: `parsers/pump_fun.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct PumpWithdrawInstruction { /// 8-byte discriminator pub discriminator: u64, /// LP tokens to burn pub lp_tokens: u64, /// Minimum amount out pub min_amount_out: u64, }
```

PumpFun Withdraw instruction data

---

### RaydiumAmmV4DepositEvent

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumAmmV4DepositEvent { /// Event metadata pub metadata: EventMetadata, /// Maximum amount of coin tokens to deposit pub max_coin_amount: u64, /// Maximum amount of PC tokens to deposit pub max_pc_amount: u64, /// Base side identifier pub base_side: u64, // Account keys /// Token program account pub token_program: Pubkey, /// Automated Market Maker account pub amm: Pubkey, /// AMM authority account pub amm_authority: Pubkey, /// AMM open orders account pub amm_open_orders: Pubkey, /// AMM target orders account pub amm_target_orders: Pubkey, /// Liquidity provider mint address pub lp_mint_address: Pubkey, /// Pool coin token account pub pool_coin_token_account: Pubkey, /// Pool PC token account pub pool_pc_token_account: Pubkey, /// Serum market account pub serum_market: Pubkey, /// User coin token account pub user_coin_token_account: Pubkey, /// User PC token account pub user_pc_token_account: Pubkey, /// User liquidity provider token account pub user_lp_token_account: Pubkey, /// User owner account pub user_owner: Pubkey, }
```

Raydium AMM V4 deposit event

---

### RaydiumAmmV4EventParser

**Source**: `raydium_amm_v4/parser.rs`

```rust
pub struct RaydiumAmmV4EventParser { inner: GenericEventParser, }
```

Raydium AMM V4 event parser

---

### RaydiumAmmV4Initialize2Event

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumAmmV4Initialize2Event { /// Event metadata pub metadata: EventMetadata, /// Nonce value for initialization pub nonce: u8, /// Time when the pool opens pub open_time: u64, /// Initial PC token amount pub init_pc_amount: u64, /// Initial coin token amount pub init_coin_amount: u64, // Account keys /// Automated Market Maker account pub amm: Pubkey, /// AMM authority account pub amm_authority: Pubkey, /// AMM open orders account pub amm_open_orders: Pubkey, /// Liquidity provider mint address pub lp_mint_address: Pubkey, /// Coin mint address pub coin_mint_address: Pubkey, /// PC mint address pub pc_mint_address: Pubkey, /// Pool coin token account pub pool_coin_token_account: Pubkey, /// Pool PC token account pub pool_pc_token_account: Pubkey, /// Pool withdrawal queue account pub pool_withdraw_queue: Pubkey, /// AMM target orders account pub amm_target_orders: Pubkey, /// Pool liquidity provider token account pub pool_lp_token_account: Pubkey, /// Pool temporary LP token account pub pool_temp_lp_token_account: Pubkey, /// Serum program account pub serum_program: Pubkey, /// Serum market account pub serum_market: Pubkey, /// User wallet account pub user_wallet: Pubkey, }
```

Raydium AMM V4 initialize2 event

---

### RaydiumAmmV4SwapEvent

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumAmmV4SwapEvent { /// Event metadata pub metadata: EventMetadata, /// Amount of tokens going into the swap pub amount_in: u64, /// Amount of tokens coming out of the swap pub amount_out: u64, /// Direction of the swap (BaseIn or BaseOut)
```

Raydium AMM V4 swap event

---

### RaydiumAmmV4WithdrawEvent

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumAmmV4WithdrawEvent { /// Event metadata pub metadata: EventMetadata, /// Amount of tokens being withdrawn pub amount: u64, // Account keys /// Token program account pub token_program: Pubkey, /// Automated Market Maker account pub amm: Pubkey, /// AMM authority account pub amm_authority: Pubkey, /// AMM open orders account pub amm_open_orders: Pubkey, /// AMM target orders account pub amm_target_orders: Pubkey, /// Liquidity provider mint address pub lp_mint_address: Pubkey, /// Pool coin token account pub pool_coin_token_account: Pubkey, /// Pool PC token account pub pool_pc_token_account: Pubkey, /// Pool withdrawal queue account pub pool_withdraw_queue: Pubkey, /// Pool temporary LP token account pub pool_temp_lp_token_account: Pubkey, /// Serum program account pub serum_program: Pubkey, /// Serum market account pub serum_market: Pubkey, /// Serum coin vault account pub serum_coin_vault_account: Pubkey, /// Serum PC vault account pub serum_pc_vault_account: Pubkey, /// Serum vault signer account pub serum_vault_signer: Pubkey, /// User liquidity provider token account pub user_lp_token_account: Pubkey, /// User coin token account pub user_coin_token_account: Pubkey, /// User PC token account pub user_pc_token_account: Pubkey, /// User owner account pub user_owner: Pubkey, /// Serum event queue account pub serum_event_queue: Pubkey, /// Serum bids account pub serum_bids: Pubkey, /// Serum asks account pub serum_asks: Pubkey, }
```

Raydium AMM V4 withdraw event

---

### RaydiumAmmV4WithdrawPnlEvent

**Source**: `raydium_amm_v4/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumAmmV4WithdrawPnlEvent { /// Event metadata pub metadata: EventMetadata, // Account keys /// Token program account pub token_program: Pubkey, /// Automated Market Maker account pub amm: Pubkey, /// AMM configuration account pub amm_config: Pubkey, /// AMM authority account pub amm_authority: Pubkey, /// AMM open orders account pub amm_open_orders: Pubkey, /// Pool coin token account pub pool_coin_token_account: Pubkey, /// Pool PC token account pub pool_pc_token_account: Pubkey, /// Coin PNL (Profit and Loss) token account
```

Raydium AMM V4 withdraw PNL event

---

### RaydiumClmmClosePositionEvent

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmClosePositionEvent { /// Event metadata pub metadata: EventMetadata, // Account keys /// Owner of the NFT pub nft_owner: Pubkey, /// Mint account for the position NFT pub position_nft_mint: Pubkey, /// Token account for the position NFT pub position_nft_account: Pubkey, /// Personal position account pub personal_position: Pubkey, }
```

Raydium CLMM close position event

---

### RaydiumClmmCreatePoolEvent

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmCreatePoolEvent { /// Event metadata pub metadata: EventMetadata, /// Square root of price multiplied by 2^64 pub sqrt_price_x64: u128, /// Current tick of the pool pub tick_current: i32, /// Observation index for the pool pub observation_index: u16, // Account keys /// Account that creates the pool pub pool_creator: Pubkey, /// Pool state account pub pool_state: Pubkey, /// Token mint for token0 pub token_mint0: Pubkey, /// Token mint for token1 pub token_mint1: Pubkey, /// Vault account for token0 pub token_vault0: Pubkey, /// Vault account for token1 pub token_vault1: Pubkey, }
```

Raydium CLMM create pool event

---

### RaydiumClmmDecreaseLiquidityV2Event

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmDecreaseLiquidityV2Event { /// Event metadata pub metadata: EventMetadata, /// Amount of liquidity to remove pub liquidity: u128, /// Minimum amount of token0 to receive pub amount0_min: u64, /// Minimum amount of token1 to receive pub amount1_min: u64, // Account keys /// Owner of the NFT pub nft_owner: Pubkey, /// Token account for the position NFT pub position_nft_account: Pubkey, /// Pool state account pub pool_state: Pubkey, }
```

Raydium CLMM decrease liquidity V2 event

---

### RaydiumClmmEventParser

**Source**: `raydium_clmm/parser.rs`

```rust
pub struct RaydiumClmmEventParser { inner: GenericEventParser, }
```

Raydium CLMM event parser

---

### RaydiumClmmIncreaseLiquidityV2Event

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmIncreaseLiquidityV2Event { /// Event metadata pub metadata: EventMetadata, /// Amount of liquidity to add pub liquidity: u128, /// Maximum amount of token0 to use pub amount0_max: u64, /// Maximum amount of token1 to use pub amount1_max: u64, /// Optional base flag pub base_flag: Option<bool>, // Account keys /// Owner of the NFT pub nft_owner: Pubkey, /// Token account for the position NFT pub position_nft_account: Pubkey, /// Pool state account pub pool_state: Pubkey, }
```

Raydium CLMM increase liquidity V2 event

---

### RaydiumClmmOpenPositionV2Event

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmOpenPositionV2Event { /// Event metadata pub metadata: EventMetadata, /// Lower tick index for the position pub tick_lower_index: i32, /// Upper tick index for the position pub tick_upper_index: i32, /// Start index for lower tick array pub tick_array_lower_start_index: i32, /// Start index for upper tick array pub tick_array_upper_start_index: i32, /// Liquidity amount for the position pub liquidity: u128, /// Maximum amount of token0 to use pub amount0_max: u64, /// Maximum amount of token1 to use pub amount1_max: u64, /// Whether to include metadata pub with_metadata: bool, /// Optional base flag pub base_flag: Option<bool>, // Account keys /// Account that pays for the transaction pub payer: Pubkey, /// Owner of the position NFT pub position_nft_owner: Pubkey, /// Mint account for the position NFT pub position_nft_mint: Pubkey, /// Token account for the position NFT pub position_nft_account: Pubkey, /// Metadata account for the NFT pub metadata_account: Pubkey, /// Pool state account pub pool_state: Pubkey, }
```

Raydium CLMM open position V2 event

---

### RaydiumClmmOpenPositionWithToken22NftEvent

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmOpenPositionWithToken22NftEvent { /// Event metadata pub metadata: EventMetadata, /// Lower tick index for the position pub tick_lower_index: i32, /// Upper tick index for the position pub tick_upper_index: i32, /// Start index for lower tick array pub tick_array_lower_start_index: i32, /// Start index for upper tick array pub tick_array_upper_start_index: i32, /// Liquidity amount for the position pub liquidity: u128, /// Maximum amount of token0 to use pub amount0_max: u64, /// Maximum amount of token1 to use pub amount1_max: u64, /// Whether to include metadata pub with_metadata: bool, /// Optional base flag pub base_flag: Option<bool>, // Account keys /// Account that pays for the transaction pub payer: Pubkey, /// Owner of the position NFT pub position_nft_owner: Pubkey, /// Mint account for the position NFT pub position_nft_mint: Pubkey, /// Token account for the position NFT pub position_nft_account: Pubkey, /// Pool state account pub pool_state: Pubkey, }
```

Raydium CLMM open position with Token-22 NFT event

---

### RaydiumClmmSwapEvent

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmSwapEvent { /// Event metadata pub metadata: EventMetadata, /// Amount of token0 in the swap pub amount0: u64, /// Amount of token1 in the swap pub amount1: u64, /// Square root of price multiplied by 2^64 pub sqrt_price_x64: u128, /// Liquidity amount for the pool pub liquidity: u128, /// Current tick of the pool pub tick_current: i32, // Account keys /// Account that pays for the transaction pub payer: Pubkey, /// Pool state account pub pool_state: Pubkey, /// Input token account pub input_token_account: Pubkey, /// Output token account pub output_token_account: Pubkey, /// Input vault account pub input_vault: Pubkey, /// Output vault account pub output_vault: Pubkey, /// Token mint for token0 pub token_mint0: Pubkey, /// Token mint for token1 pub token_mint1: Pubkey, }
```

Raydium CLMM swap event

---

### RaydiumClmmSwapV2Event

**Source**: `raydium_clmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub struct RaydiumClmmSwapV2Event { /// Event metadata pub metadata: EventMetadata, /// Amount of token0 in the swap pub amount0: u64, /// Amount of token1 in the swap pub amount1: u64, /// Square root of price multiplied by 2^64 pub sqrt_price_x64: u128, /// Liquidity amount for the pool pub liquidity: u128, /// Current tick of the pool pub tick_current: i32, /// Whether base token is the input pub is_base_input: bool, // Account keys /// Account that pays for the transaction pub payer: Pubkey, /// Pool state account pub pool_state: Pubkey, /// Input token account pub input_token_account: Pubkey, /// Output token account pub output_token_account: Pubkey, /// Input vault account pub input_vault: Pubkey, /// Output vault account pub output_vault: Pubkey, /// Token mint for token0 pub token_mint0: Pubkey, /// Token mint for token1 pub token_mint1: Pubkey, }
```

Raydium CLMM swap V2 event

---

### RaydiumCpmmDepositEvent

**Source**: `raydium_cpmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct RaydiumCpmmDepositEvent { /// Event metadata (excluded from serialization)
```

Raydium CPMM Deposit event

---

### RaydiumCpmmEventParser

**Source**: `raydium_cpmm/parser.rs`

```rust
pub struct RaydiumCpmmEventParser { inner: GenericEventParser, }
```

Raydium CPMM event parser

---

### RaydiumCpmmSwapEvent

**Source**: `raydium_cpmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, Default)]
```

```rust
pub struct RaydiumCpmmSwapEvent { /// Event metadata (excluded from serialization)
```

Raydium CPMM Swap event

---

### RaydiumV4Parser

**Source**: `parsers/raydium_v4.rs`

```rust
pub struct RaydiumV4Parser { /// Program ID for validation #[allow(dead_code)]
```

High-performance Raydium V4 parser

---

### RaydiumV4ParserFactory

**Source**: `parsers/raydium_v4.rs`

```rust
pub struct RaydiumV4ParserFactory;
```

Factory for creating Raydium V4 parsers

---

### RouteHop

**Source**: `parsers/jupiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RouteHop { /// DEX program ID pub program_id: Pubkey, /// Input mint pub input_mint: Option<Pubkey>, /// Output mint pub output_mint: Option<Pubkey>, /// Amount in for this hop pub amount_in: Option<u64>, /// Amount out for this hop pub amount_out: Option<u64>, }
```

Simplified route hop representation

---

### RoutePlan

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RoutePlan { /// Input token mint address pub input_mint: Pubkey, /// Output token mint address pub output_mint: Pubkey, /// Amount going into this step pub amount_in: u64, /// Amount coming out of this step pub amount_out: u64, /// Label identifying the DEX used pub dex_label: String, }
```

Route plan information (simplified for event data)

---

### RoutePlanStep

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct RoutePlanStep { /// Swap information for this step pub swap: SwapInfo, /// Percentage of the input amount for this step pub percent: u8, }
```

Route plan step for Jupiter swaps

---

### RpcConnectionPool

**Source**: `zero_copy/parsers.rs`

```rust
pub struct RpcConnectionPool { /// Pool of shared RPC client instances clients: Vec<Arc<solana_client::rpc_client::RpcClient>>, /// Current index for round-robin client selection current: std::sync::atomic::AtomicUsize, }
```

Connection pool for RPC calls during parsing

---

### RuleResult

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Default)]
```

```rust
pub struct RuleResult { /// Validation errors found by the rule pub errors: Vec<ValidationError>, /// Validation warnings found by the rule pub warnings: Vec<ValidationWarning>, /// Number of consistency checks performed pub consistency_checks: usize, }
```

Result from a validation rule

---

### SIMDPatternMatcher

**Source**: `zero_copy/parsers.rs`

**Attributes**:
```rust
#[derive(Default)]
```

```rust
pub struct SIMDPatternMatcher { /// Discriminator byte patterns to match patterns: Vec<Vec<u8>>, /// Protocol types corresponding to each pattern protocols: Vec<ProtocolType>, }
```

SIMD-optimized pattern matcher for instruction discriminators

---

### SharedAccountsExactOutRouteData

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct SharedAccountsExactOutRouteData { /// Route plan steps for the swap pub route_plan: Vec<RoutePlanStep>, /// Output amount in base units pub out_amount: u64, /// Quoted input amount in base units pub quoted_in_amount: u64, /// Slippage tolerance in basis points pub slippage_bps: u16, /// Platform fee in basis points pub platform_fee_bps: u8, }
```

Jupiter exact out route instruction data (after discriminator)

---

### SharedAccountsRouteData

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct SharedAccountsRouteData { /// Route plan steps for the swap pub route_plan: Vec<RoutePlanStep>, /// Input amount in base units pub in_amount: u64, /// Quoted output amount in base units pub quoted_out_amount: u64, /// Slippage tolerance in basis points pub slippage_bps: u16, /// Platform fee in basis points pub platform_fee_bps: u8, }
```

Jupiter shared accounts route instruction data (after discriminator)

---

### SolanaEvent

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct SolanaEvent { /// Event metadata pub metadata: EventMetadata, /// Event data payload pub data: serde_json::Value, /// Transfer data for token movements pub transfer_data: Vec<TransferData>, }
```

A wrapper that implements the Event trait for Solana events

---

### SolanaEventMetadata

**Source**: `src/solana_metadata.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct SolanaEventMetadata { /// Transaction signature pub signature: String, /// Slot number pub slot: u64, /// Event type pub event_type: EventType, /// Protocol type pub protocol_type: ProtocolType, /// Instruction index pub index: String, /// Program received time in milliseconds pub program_received_time_ms: i64, /// The underlying core metadata (excluded from borsh serialization)
```

Solana-specific event metadata that wraps core EventMetadata
and provides additional fields and trait implementations

---

### SolanaEventMetadata

**Source**: `src/metadata_helpers.rs`

```rust
pub struct SolanaEventMetadata { /// The wrapped EventMetadata instance pub inner: EventMetadata, }
```

Helper struct that wraps EventMetadata for Solana-specific operations
This is primarily used in legacy code that expects direct field access

---

### SolanaEventParser

**Source**: `src/solana_parser.rs`

```rust
pub struct SolanaEventParser { /// Legacy multi-parser for actual parsing logic legacy_parser: EventParserRegistry, /// Parser information info: ParserInfo, /// Supported program IDs supported_programs: Vec<Pubkey>, }
```

Solana event parser that bridges between legacy and new parsers

---

### SolanaInnerInstructionInput

**Source**: `src/solana_parser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SolanaInnerInstructionInput { /// Inner instruction data pub inner_instruction: UiCompiledInstruction, /// Transaction signature pub signature: String, /// Slot number pub slot: u64, /// Block time (optional)
```

Input type for Solana inner instruction parsing

---

### SolanaInnerInstructionParser

**Source**: `src/solana_parser.rs`

```rust
pub struct SolanaInnerInstructionParser { /// Inner Solana parser solana_parser: Arc<SolanaEventParser>, }
```

Inner instruction parser that implements the riglr-events-core EventParser trait

---

### SolanaTransactionInput

**Source**: `src/solana_parser.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SolanaTransactionInput { /// Solana instruction data pub instruction: solana_sdk::instruction::CompiledInstruction, /// Account keys from the transaction pub accounts: Vec<Pubkey>, /// Transaction signature pub signature: String, /// Slot number pub slot: u64, /// Block time (optional)
```

Input type for Solana transaction parsing

---

### StreamMetadata

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct StreamMetadata { /// Source of the stream (e.g., "geyser", "binance", "evm-ws")
```

Metadata for streaming context

---

### SwapBaseInInstruction

**Source**: `parsers/raydium_v4.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct SwapBaseInInstruction { /// Discriminator (should be 0x09)
```

Raydium V4 SwapBaseIn instruction data

---

### SwapBaseOutInstruction

**Source**: `parsers/raydium_v4.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct SwapBaseOutInstruction { /// Discriminator (should be 0x0a)
```

Raydium V4 SwapBaseOut instruction data

---

### SwapData

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct SwapData { /// Input token mint public key pub input_mint: Pubkey, /// Output token mint public key pub output_mint: Pubkey, /// Amount of input tokens in smallest unit pub amount_in: u64, /// Amount of output tokens in smallest unit pub amount_out: u64, }
```

Data structure for token swap events

---

### SwapEventParams

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SwapEventParams { /// Event identifier pub id: String, /// Transaction signature pub signature: String, /// Solana slot number pub slot: u64, /// Block timestamp pub block_time: i64, /// Protocol type (e.g., Jupiter, Raydium)
```

Parameters for creating swap events, reducing function parameter count

---

### SwapInfo

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct SwapInfo { /// Source token mint address pub source_token: Pubkey, /// Destination token mint address pub destination_token: Pubkey, /// Source token account address pub source_token_account: Pubkey, /// Destination token account address pub destination_token_account: Pubkey, /// Program ID for the swap pub swap_program_id: Pubkey, /// Account metadata required for the swap pub swap_accounts: Vec<AccountMeta>, /// Instruction data for the swap pub swap_data: Vec<u8>, }
```

Swap information within a route step

---

### TokenMetadata

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TokenMetadata { /// Token mint address pub mint: Pubkey, /// Token name pub name: Option<String>, /// Token symbol pub symbol: Option<String>, /// Token decimals pub decimals: u8, /// Token logo URI pub logo_uri: Option<String>, /// Token description pub description: Option<String>, /// When this metadata was fetched pub fetched_at: Instant, }
```

Token metadata information

---

### TransactionContext

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct TransactionContext { /// Transaction signature pub signature: String, /// Block height pub block_height: Option<u64>, /// Slot number pub slot: u64, /// Block time pub block_time: Option<i64>, /// Fee paid for the transaction pub fee: Option<u64>, /// Compute units consumed pub compute_units_consumed: Option<u64>, }
```

Transaction context information

---

### TransferData

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct TransferData { /// Source account public key pub source: Pubkey, /// Destination account public key pub destination: Pubkey, /// Token mint public key (None for SOL transfers)
```

Data structure for token transfer events

---

### TransferInstruction

**Source**: `parsers/metaplex.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct TransferInstruction { /// Discriminator pub discriminator: u8, /// Authorization data pub authorization_data: Option<Vec<u8>>, }
```

Metaplex Transfer instruction data

---

### ValidationConfig

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ValidationConfig { /// Enable strict validation (fails on any error)
```

Configuration for validation pipeline

---

### ValidationMetrics

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ValidationMetrics { /// Time spent validating pub validation_time: Duration, /// Number of rules checked pub rules_checked: usize, /// Number of consistency checks performed pub consistency_checks: usize, }
```

Validation metrics

---

### ValidationPipeline

**Source**: `pipelines/validation.rs`

```rust
pub struct ValidationPipeline { /// Validation configuration config: ValidationConfig, /// Duplicate detection cache seen_events: Arc<DashMap<String, Instant>>, /// Validation rule registry rules: Vec<Arc<dyn ValidationRule>>, }
```

Data integrity validation pipeline

---

### ValidationResult

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ValidationResult { /// Whether the event is valid pub is_valid: bool, /// Validation errors found pub errors: Vec<ValidationError>, /// Validation warnings (non-critical issues)
```

Validation result for a single event

---

### ValidationStats

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ValidationStats { /// Total number of validation rules registered pub total_rules: usize, /// Number of events currently cached for duplicate detection pub cached_events: usize, /// Whether strict mode is enabled pub strict_mode: bool, }
```

Validation pipeline statistics

---

### VestingParams

**Source**: `bonk/types.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct VestingParams { /// Total amount of tokens locked in vesting pub total_locked_amount: u64, /// Duration before any tokens can be unlocked pub cliff_period: u64, /// Period over which tokens are gradually unlocked pub unlock_period: u64, }
```

Parameters for token vesting schedules

---

### WhirlpoolAccount

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct WhirlpoolAccount { /// Configuration account that governs this whirlpool's parameters pub whirlpools_config: Pubkey, /// Program-derived address bump seed for this whirlpool account pub whirlpool_bump: [u8; 1], /// The tick spacing for this whirlpool, determining price granularity pub tick_spacing: u16, /// Seed bytes used to derive the whirlpool account from tick spacing pub tick_spacing_seed: [u8; 2], /// Fee rate charged for swaps in basis points (e.g., 300 = 0.3%)
```

Orca Whirlpool account layout

---

### WhirlpoolRewardInfo

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct WhirlpoolRewardInfo { /// Mint address of the reward token being distributed pub mint: Pubkey, /// Vault account holding the reward tokens for distribution pub vault: Pubkey, /// Authority account that can control reward distribution parameters pub authority: Pubkey, /// Rate of reward token emissions per second as Q64.64 fixed-point pub emissions_per_second_x64: u128, /// Global growth accumulator for this reward token as Q64.64 fixed-point pub growth_global_x64: u128, }
```

Whirlpool reward information

---

### WithdrawInstruction

**Source**: `parsers/raydium_v4.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
```

```rust
pub struct WithdrawInstruction { /// Discriminator (should be 0x04)
```

Raydium V4 Withdraw instruction data

---

### ZeroCopyEvent

**Source**: `zero_copy/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ZeroCopyEvent<'a> { /// Event metadata (owned)
```

Zero-copy base event that holds references to source data

---

### ZeroCopyLiquidityEvent

**Source**: `zero_copy/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ZeroCopyLiquidityEvent<'a> { /// Base event data pub base: ZeroCopyEvent<'a>, /// Pool address (parsed lazily)
```

Zero-copy liquidity event

---

### ZeroCopySwapEvent

**Source**: `zero_copy/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ZeroCopySwapEvent<'a> { /// Base event data pub base: ZeroCopyEvent<'a>, /// Input mint (parsed lazily)
```

Specialized zero-copy swap event

---

## Functions (solana_metadata)

### core

**Source**: `src/solana_metadata.rs`

```rust
pub fn core(&self) -> &EventMetadata
```

Get a reference to the core EventMetadata

---

### core_mut

**Source**: `src/solana_metadata.rs`

```rust
pub fn core_mut(&mut self) -> &mut EventMetadata
```

Get a mutable reference to the core EventMetadata

---

### create_metadata

**Source**: `src/solana_metadata.rs`

```rust
pub fn create_metadata( id: String, signature: String, slot: u64, block_time: Option<i64>, received_time: i64, index: String, event_type: EventType, protocol_type: ProtocolType, ) -> SolanaEventMetadata
```

Create a SolanaEventMetadata from components

---

### event_kind

**Source**: `src/solana_metadata.rs`

```rust
pub fn event_kind(&self) -> EventKind
```

Convert event type to event kind

---

### id

**Source**: `src/solana_metadata.rs`

```rust
pub fn id(&self) -> &str
```

Get the event ID

---

### kind

**Source**: `src/solana_metadata.rs`

```rust
pub fn kind(&self) -> &EventKind
```

Get the event kind from core metadata

---

### new

**Source**: `src/solana_metadata.rs`

```rust
pub fn new( signature: String, slot: u64, event_type: EventType, protocol_type: ProtocolType, index: String, program_received_time_ms: i64, core: EventMetadata, ) -> Self
```

Create new Solana event metadata

---

### set_id

**Source**: `src/solana_metadata.rs`

```rust
pub fn set_id(&mut self, id: String)
```

Set the event ID

---

## Functions (solana_parser)

### add_protocol_parser

**Source**: `src/solana_parser.rs`

```rust
pub fn add_protocol_parser<P>( &mut self, _protocol: Protocol, _parser: Arc<dyn LegacyEventParser>, ) where P: LegacyEventParser + 'static,
```

Add a protocol parser

---

### new

**Source**: `src/solana_parser.rs`

```rust
pub fn new() -> Self
```

Create a new Solana event parser

---

### new

**Source**: `src/solana_parser.rs`

```rust
pub fn new(solana_parser: Arc<SolanaEventParser>) -> Self
```

Create new inner instruction parser

---

### new

**Source**: `src/solana_parser.rs`

```rust
pub fn new( instruction: solana_sdk::instruction::CompiledInstruction, accounts: Vec<Pubkey>, signature: String, slot: u64, block_time: Option<i64>, instruction_index: usize, ) -> Self
```

Create a new Solana transaction input

---

### new

**Source**: `src/solana_parser.rs`

```rust
pub fn new( inner_instruction: UiCompiledInstruction, signature: String, slot: u64, block_time: Option<i64>, instruction_index: String, ) -> Self
```

Create a new Solana inner instruction input

---

### parse_inner_instruction

**Source**: `src/solana_parser.rs`

```rust
pub async fn parse_inner_instruction( &self, input: SolanaInnerInstructionInput, ) -> EventResult<Vec<SolanaEvent>>
```

Parse events from a Solana inner instruction

---

### parse_instruction

**Source**: `src/solana_parser.rs`

```rust
pub async fn parse_instruction( &self, input: SolanaTransactionInput, ) -> EventResult<Vec<SolanaEvent>>
```

Parse events from a Solana instruction

---

### supports_program

**Source**: `src/solana_parser.rs`

```rust
pub fn supports_program(&self, program_id: &Pubkey) -> bool
```

Check if a program ID is supported

---

### with_legacy_parser

**Source**: `src/solana_parser.rs`

```rust
pub fn with_legacy_parser(legacy_parser: EventParserRegistry) -> Self
```

Create with specific legacy parser

---

### with_received_time

**Source**: `src/solana_parser.rs`

```rust
pub fn with_received_time(mut self, received_time: chrono::DateTime<chrono::Utc>) -> Self
```

Set the received time

---

### with_received_time

**Source**: `src/solana_parser.rs`

```rust
pub fn with_received_time(mut self, received_time: chrono::DateTime<chrono::Utc>) -> Self
```

Set the received time

---

## Functions (solana_events)

### extract_data

**Source**: `src/solana_events.rs`

```rust
pub fn extract_data<T>(&self) -> Result<T, serde_json::Error> where T: for<'de> Deserialize<'de>,
```

Extract typed data from the event

---

### liquidity

**Source**: `src/solana_events.rs`

```rust
pub fn liquidity(params: LiquidityEventParams) -> Self
```

Create a liquidity event

---

### new

**Source**: `src/solana_events.rs`

```rust
pub fn new(metadata: EventMetadata, data: serde_json::Value) -> Self
```

Create a new Solana event

---

### protocol_event

**Source**: `src/solana_events.rs`

```rust
pub fn protocol_event(params: ProtocolEventParams) -> Self
```

Create a generic protocol event

---

### swap

**Source**: `src/solana_events.rs`

```rust
pub fn swap(params: SwapEventParams) -> Self
```

Create a swap event

---

### with_data

**Source**: `src/solana_events.rs`

```rust
pub fn with_data<T: Serialize>(mut self, data: T) -> Result<Self, serde_json::Error>
```

Update the data payload

---

### with_transfer_data

**Source**: `src/solana_events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Set transfer data

---

## Traits

### ByteSliceEventParser

**Source**: `zero_copy/parsers.rs`

```rust
pub trait ByteSliceEventParser: Send + Sync { ... }
```

Trait for zero-copy byte slice parsing

**Methods**:

#### `parse_from_slice`

```rust
fn parse_from_slice<'a>( &self, data: &'a [u8], metadata: EventMetadata, ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError>;
```

#### `can_parse`

```rust
fn can_parse(&self, data: &[u8]) -> bool;
```

#### `protocol_type`

```rust
fn protocol_type(&self) -> ProtocolType;
```

---

### EventParser

**Source**: `core/traits.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait EventParser: Send + Sync { ... }
```

Event Parser trait - defines the core methods for event parsing
This now returns Event trait objects instead of UnifiedEvent

**Methods**:

#### `inner_instruction_configs`

```rust
fn inner_instruction_configs( &self, ) -> std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>;
```

#### `instruction_configs`

```rust
fn instruction_configs( &self, ) -> std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>;
```

#### `parse_events_from_inner_instruction`

```rust
fn parse_events_from_inner_instruction( &self, inner_instruction: &solana_transaction_status::UiCompiledInstruction, signature: &str, slot: u64, block_time: Option<i64>, program_received_time_ms: i64, index: String, ) -> Vec<Box<dyn Event>>;
```

#### `parse_events_from_instruction`

```rust
fn parse_events_from_instruction( &self, instruction: &solana_sdk::instruction::CompiledInstruction, accounts: &[solana_sdk::pubkey::Pubkey], signature: &str, slot: u64, block_time: Option<i64>, program_received_time_ms: i64, index: String, ) -> Vec<Box<dyn Event>>;
```

#### `should_handle`

```rust
fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool;
```

#### `supported_program_ids`

```rust
fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey>;
```

---

### ToSolanaEvent

**Source**: `src/solana_events.rs`

```rust
pub trait ToSolanaEvent { ... }
```

Helper trait for converting legacy events to Solana events

**Methods**:

#### `to_solana_event`

```rust
fn to_solana_event(&self) -> SolanaEvent;
```

---

### ValidationRule

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait ValidationRule: Send + Sync { ... }
```

Trait for validation rules

**Methods**:

#### `validate`

```rust
async fn validate(&self, event: &ZeroCopyEvent<'_>, config: &ValidationConfig) -> RuleResult;
```

#### `name`

```rust
fn name(&self) -> &'static str;
```

---

## Functions (types)

### amount_to_shares

**Source**: `marginfi/types.rs`

```rust
pub fn amount_to_shares(amount: u64, share_value: u128) -> u128
```

Convert amount to shares using share value

---

### bin_id_to_price

**Source**: `meteora/types.rs`

```rust
pub fn bin_id_to_price(bin_id: u32, bin_step: u16) -> f64
```

Convert bin ID to price for DLMM

---

### calculate_active_bin_price

**Source**: `meteora/types.rs`

```rust
pub fn calculate_active_bin_price(active_id: u32, bin_step: u16) -> f64
```

Calculate active bin price

---

### calculate_health_ratio

**Source**: `marginfi/types.rs`

```rust
pub fn calculate_health_ratio( total_asset_value: u128, total_liability_value: u128, maintenance_margin: u64, ) -> f64
```

Calculate health ratio for a MarginFi account

---

### calculate_interest_rate

**Source**: `marginfi/types.rs`

```rust
pub fn calculate_interest_rate( utilization_rate: u64, base_rate: u64, slope1: u64, slope2: u64, optimal_utilization: u64, ) -> u64
```

Calculate interest rate based on utilization

---

### calculate_liquidation_threshold

**Source**: `marginfi/types.rs`

```rust
pub fn calculate_liquidation_threshold(total_asset_value: u128, liquidation_ltv: u64) -> u128
```

Calculate liquidation threshold

---

### calculate_liquidity_distribution

**Source**: `meteora/types.rs`

```rust
pub fn calculate_liquidity_distribution( amount_x: u64, amount_y: u64, bin_id_from: u32, bin_id_to: u32, active_id: u32, ) -> Vec<(u32, u64, u64)>
```

Calculate liquidity distribution across bins

---

### create_solana_metadata

**Source**: `src/types.rs`

```rust
pub fn create_solana_metadata( id: String, signature: String, slot: u64, block_time: i64, protocol_type: ProtocolType, event_type: EventType, program_id: Pubkey, index: String, program_received_time_ms: i64, ) -> EventMetadata
```

Create a core EventMetadata with Solana chain data

---

### from_event_kind

**Source**: `src/types.rs`

```rust
pub fn from_event_kind(kind: &riglr_events_core::types::EventKind) -> Self
```

Convert from riglr-events-core EventKind to local EventType

---

### is_jupiter_v6_program

**Source**: `jupiter/types.rs`

```rust
pub fn is_jupiter_v6_program(program_id: &Pubkey) -> bool
```

Check if the given pubkey is Jupiter V6 program

---

### is_marginfi_program

**Source**: `marginfi/types.rs`

```rust
pub fn is_marginfi_program(program_id: &Pubkey) -> bool
```

Check if the given pubkey is MarginFi program

---

### is_meteora_dlmm_program

**Source**: `meteora/types.rs`

```rust
pub fn is_meteora_dlmm_program(program_id: &Pubkey) -> bool
```

Check if the given pubkey is Meteora DLMM program

---

### is_meteora_dynamic_program

**Source**: `meteora/types.rs`

```rust
pub fn is_meteora_dynamic_program(program_id: &Pubkey) -> bool
```

Check if the given pubkey is Meteora Dynamic program

---

### is_orca_whirlpool_program

**Source**: `orca/types.rs`

```rust
pub fn is_orca_whirlpool_program(program_id: &Pubkey) -> bool
```

Check if the given pubkey is Orca Whirlpool program

---

### jupiter_v6_program_id

**Source**: `jupiter/types.rs`

```rust
pub fn jupiter_v6_program_id() -> Pubkey
```

Extract Jupiter program ID as Pubkey

---

### marginfi_bank_program_id

**Source**: `marginfi/types.rs`

```rust
pub fn marginfi_bank_program_id() -> Pubkey
```

Extract MarginFi bank program ID as Pubkey

---

### marginfi_program_id

**Source**: `marginfi/types.rs`

```rust
pub fn marginfi_program_id() -> Pubkey
```

Extract MarginFi program ID as Pubkey

---

### meteora_dlmm_program_id

**Source**: `meteora/types.rs`

```rust
pub fn meteora_dlmm_program_id() -> Pubkey
```

Extract Meteora DLMM program ID as Pubkey

---

### meteora_dynamic_program_id

**Source**: `meteora/types.rs`

```rust
pub fn meteora_dynamic_program_id() -> Pubkey
```

Extract Meteora Dynamic program ID as Pubkey

---

### orca_whirlpool_program_id

**Source**: `orca/types.rs`

```rust
pub fn orca_whirlpool_program_id() -> Pubkey
```

Extract Orca Whirlpool program ID as Pubkey

---

### shares_to_amount

**Source**: `marginfi/types.rs`

```rust
pub fn shares_to_amount(shares: u128, share_value: u128) -> u64
```

Convert shares to amount using share value

---

### sqrt_price_to_price

**Source**: `orca/types.rs`

```rust
pub fn sqrt_price_to_price(sqrt_price: u128, decimals_a: u8, decimals_b: u8) -> f64
```

Convert sqrt price to price

---

### tick_index_to_price

**Source**: `orca/types.rs`

```rust
pub fn tick_index_to_price(tick_index: i32) -> f64
```

Convert tick index to price

---

### to_event_kind

**Source**: `src/types.rs`

```rust
pub fn to_event_kind(&self) -> riglr_events_core::types::EventKind
```

Convert local EventType to riglr-events-core EventKind

---

## Functions (metadata_helpers)

### create_solana_metadata

**Source**: `src/metadata_helpers.rs`

```rust
pub fn create_solana_metadata( id: String, kind: EventKind, source: String, slot: u64, signature: Option<String>, program_id: Option<Pubkey>, instruction_index: Option<usize>, block_time: Option<i64>, protocol_type: ProtocolType, event_type: EventType, ) -> EventMetadata
```

Create a new EventMetadata for Solana events with all required fields

---

### event_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get the event type

---

### get_block_time

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_block_time(metadata: &EventMetadata) -> Option<i64>
```

Get block_time from Solana EventMetadata

---

### get_event_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_event_type(metadata: &EventMetadata) -> Option<EventType>
```

Get event_type from Solana EventMetadata

---

### get_instruction_index

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_instruction_index(metadata: &EventMetadata) -> Option<usize>
```

Get instruction_index from Solana EventMetadata

---

### get_program_id

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_program_id(metadata: &EventMetadata) -> Option<&Pubkey>
```

Get program_id from Solana EventMetadata

---

### get_protocol_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_protocol_type(metadata: &EventMetadata) -> Option<ProtocolType>
```

Get protocol_type from Solana EventMetadata

---

### get_signature

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_signature(metadata: &EventMetadata) -> Option<&str>
```

Get signature from Solana EventMetadata

---

### get_slot

**Source**: `src/metadata_helpers.rs`

```rust
pub fn get_slot(metadata: &EventMetadata) -> Option<u64>
```

Get slot from Solana EventMetadata

---

### index

**Source**: `src/metadata_helpers.rs`

```rust
pub fn index(&self) -> Option<usize>
```

Get the instruction index

---

### into_inner

**Source**: `src/metadata_helpers.rs`

```rust
pub fn into_inner(self) -> EventMetadata
```

Convert to the inner EventMetadata

---

### new

**Source**: `src/metadata_helpers.rs`

```rust
pub fn new( id: String, kind: EventKind, source: String, slot: u64, signature: Option<String>, program_id: Option<Pubkey>, instruction_index: Option<usize>, block_time: Option<i64>, protocol_type: ProtocolType, event_type: EventType, ) -> Self
```

Create a new SolanaEventMetadata

---

### protocol_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn protocol_type(&self) -> ProtocolType
```

Get the protocol type

---

### set_event_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn set_event_type(metadata: &mut EventMetadata, event_type: EventType)
```

Set event_type in Solana EventMetadata

---

### set_event_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn set_event_type(&mut self, event_type: EventType)
```

Set the event type

---

### set_protocol_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn set_protocol_type(metadata: &mut EventMetadata, protocol_type: ProtocolType)
```

Set protocol_type in Solana EventMetadata

---

### set_protocol_type

**Source**: `src/metadata_helpers.rs`

```rust
pub fn set_protocol_type(&mut self, protocol_type: ProtocolType)
```

Set the protocol type

---

### signature

**Source**: `src/metadata_helpers.rs`

```rust
pub fn signature(&self) -> Option<String>
```

Get the signature

---

### slot

**Source**: `src/metadata_helpers.rs`

```rust
pub fn slot(&self) -> u64
```

Get the slot

---

## Functions (factory)

### add_parser

**Source**: `events/factory.rs`

```rust
pub fn add_parser(&mut self, protocol: Protocol, parser: Arc<dyn EventParser>)
```

Add a parser for a specific protocol

---

### get_parser

**Source**: `events/factory.rs`

```rust
pub fn get_parser(&self, protocol: &Protocol) -> Option<&Arc<dyn EventParser>>
```

Get parser for a specific protocol

---

### get_parser_for_program

**Source**: `events/factory.rs`

```rust
pub fn get_parser_for_program( &self, program_id: &solana_sdk::pubkey::Pubkey, ) -> Option<&Arc<dyn EventParser>>
```

Get parser for a specific program ID

---

### new

**Source**: `events/factory.rs`

```rust
pub fn new() -> Self
```

Create a new event parser registry

---

### parse_events_from_inner_instruction

**Source**: `events/factory.rs`

```rust
pub fn parse_events_from_inner_instruction( &self, params: InnerInstructionParseParams, ) -> Vec<Box<dyn Event>>
```

Parse events from inner instruction using the appropriate parser

---

### parse_events_from_instruction

**Source**: `events/factory.rs`

```rust
pub fn parse_events_from_instruction( &self, params: InstructionParseParams, ) -> Vec<Box<dyn Event>>
```

Parse events from instruction using the appropriate parser

---

### should_handle

**Source**: `events/factory.rs`

```rust
pub fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool
```

Check if a program ID is supported

---

### supported_program_ids

**Source**: `events/factory.rs`

```rust
pub fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey>
```

Get all supported program IDs

---

### with_all_parsers

**Source**: `events/factory.rs`

```rust
pub fn with_all_parsers() -> Self
```

Create a registry with all available parsers

---

## Functions (metaplex)

### create_fast

**Source**: `parsers/metaplex.rs`

```rust
pub fn create_fast() -> Arc<dyn ByteSliceEventParser>
```

Create a fast parser with minimal metadata

---

### create_zero_copy

**Source**: `parsers/metaplex.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser with full metadata

---

### event_type

**Source**: `parsers/metaplex.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### from_byte

**Source**: `parsers/metaplex.rs`

```rust
pub fn from_byte(byte: u8) -> Option<Self>
```

Parse discriminator from byte

---

### new

**Source**: `parsers/metaplex.rs`

```rust
pub fn new() -> Self
```

Create new Metaplex parser

---

### new_fast

**Source**: `parsers/metaplex.rs`

```rust
pub fn new_fast() -> Self
```

Create parser with minimal metadata parsing (faster)

---

## Functions (jupiter)

### bytes

**Source**: `parsers/jupiter.rs`

```rust
pub fn bytes(&self) -> &'static [u8; 8]
```

Get discriminator bytes

---

### create_fast

**Source**: `parsers/jupiter.rs`

```rust
pub fn create_fast() -> Arc<dyn ByteSliceEventParser>
```

Create a fast parser with simplified analysis

---

### create_standard

**Source**: `parsers/jupiter.rs`

```rust
pub fn create_standard() -> Arc<dyn ByteSliceEventParser>
```

Create a standard parser

---

### create_zero_copy

**Source**: `parsers/jupiter.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser with full analysis

---

### event_type

**Source**: `parsers/jupiter.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### from_bytes

**Source**: `parsers/jupiter.rs`

```rust
pub fn from_bytes(bytes: &[u8]) -> Option<Self>
```

Parse discriminator from first 8 bytes

---

### new

**Source**: `parsers/jupiter.rs`

```rust
pub fn new() -> Self
```

Create new Jupiter parser with full analysis

---

### new_fast

**Source**: `parsers/jupiter.rs`

```rust
pub fn new_fast() -> Self
```

Create parser with simplified analysis (faster)

---

### new_standard

**Source**: `parsers/jupiter.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

## Functions (pump_fun)

### create_standard

**Source**: `parsers/pump_fun.rs`

```rust
pub fn create_standard() -> Arc<dyn ByteSliceEventParser>
```

Create a standard parser

---

### create_zero_copy

**Source**: `parsers/pump_fun.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser

---

### event_type

**Source**: `parsers/pump_fun.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### from_u64

**Source**: `parsers/pump_fun.rs`

```rust
pub fn from_u64(value: u64) -> Option<Self>
```

Parse discriminator from first 8 bytes (u64 little-endian)

---

### new

**Source**: `parsers/pump_fun.rs`

```rust
pub fn new() -> Self
```

Create new PumpFun parser

---

### new_standard

**Source**: `parsers/pump_fun.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

## Functions (raydium_v4)

### create_standard

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn create_standard() -> Arc<dyn ByteSliceEventParser>
```

Create a standard parser

---

### create_zero_copy

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser

---

### event_type

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### from_byte

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn from_byte(byte: u8) -> Option<Self>
```

Parse discriminator from byte

---

### new_standard

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

## Functions (enrichment)

### enrich_event

**Source**: `pipelines/enrichment.rs`

```rust
pub async fn enrich_event( &self, mut event: ZeroCopyEvent<'static>, ) -> Result<ZeroCopyEvent<'static>, EnrichmentError>
```

Enrich a single event with additional metadata

---

### enrich_events

**Source**: `pipelines/enrichment.rs`

```rust
pub async fn enrich_events( &self, events: Vec<ZeroCopyEvent<'static>>, ) -> Result<Vec<ZeroCopyEvent<'static>>, EnrichmentError>
```

Enrich multiple events in parallel

---

### get_cache_stats

**Source**: `pipelines/enrichment.rs`

```rust
pub fn get_cache_stats(&self) -> CacheStats
```

Get cache statistics

---

### new

**Source**: `pipelines/enrichment.rs`

```rust
pub fn new(config: EnrichmentConfig) -> Self
```

Create a new event enricher

---

## Functions (parsing)

### add_parser

**Source**: `pipelines/parsing.rs`

```rust
pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>)
```

Add a parser to the pipeline

---

### add_parser

**Source**: `pipelines/parsing.rs`

```rust
pub fn add_parser(mut self, parser: Arc<dyn ByteSliceEventParser>) -> Self
```

Add a parser

---

### build

**Source**: `pipelines/parsing.rs`

```rust
pub fn build(self) -> ParsingPipeline
```

Build the parsing pipeline

---

### get_stats

**Source**: `pipelines/parsing.rs`

```rust
pub fn get_stats(&self) -> PipelineStats
```

Get pipeline statistics

---

### new

**Source**: `pipelines/parsing.rs`

```rust
pub fn new(config: ParsingPipelineConfig) -> Self
```

Create a new parsing pipeline

---

### new

**Source**: `pipelines/parsing.rs`

```rust
pub fn new() -> Self
```

Create a new builder

---

### process_stream

**Source**: `pipelines/parsing.rs`

```rust
pub async fn process_stream<S>(&mut self, mut input_stream: S) -> Result<(), PipelineError> where S: Stream<Item = ParsingInput> + Unpin,
```

Process a stream of parsing inputs

---

### take_output_receiver

**Source**: `pipelines/parsing.rs`

```rust
pub fn take_output_receiver(&mut self) -> mpsc::UnboundedReceiver<ParsingOutput>
```

Get the output receiver for processed events

---

### with_batch_size

**Source**: `pipelines/parsing.rs`

```rust
pub fn with_batch_size(mut self, size: usize) -> Self
```

Set batch size

---

### with_batch_timeout

**Source**: `pipelines/parsing.rs`

```rust
pub fn with_batch_timeout(mut self, timeout: Duration) -> Self
```

Set batch timeout

---

### with_concurrency_limit

**Source**: `pipelines/parsing.rs`

```rust
pub fn with_concurrency_limit(mut self, limit: usize) -> Self
```

Set concurrency limit

---

### with_metrics

**Source**: `pipelines/parsing.rs`

```rust
pub fn with_metrics(mut self, enable: bool) -> Self
```

Enable or disable metrics collection

---

### with_parser_config

**Source**: `pipelines/parsing.rs`

```rust
pub fn with_parser_config(mut self, protocol: ProtocolType, config: ParserConfig) -> Self
```

Add parser configuration

---

## Functions (validation)

### add_rule

**Source**: `pipelines/validation.rs`

```rust
pub fn add_rule(&mut self, rule: Arc<dyn ValidationRule>)
```

Add a custom validation rule

---

### get_stats

**Source**: `pipelines/validation.rs`

```rust
pub async fn get_stats(&self) -> ValidationStats
```

Get pipeline statistics

---

### new

**Source**: `pipelines/validation.rs`

```rust
pub fn new(config: ValidationConfig) -> Self
```

Create a new validation pipeline

---

### validate_event

**Source**: `pipelines/validation.rs`

```rust
pub async fn validate_event(&self, event: &ZeroCopyEvent<'static>) -> ValidationResult
```

Validate a single event

---

### validate_events

**Source**: `pipelines/validation.rs`

```rust
pub async fn validate_events( &self, events: &[ZeroCopyEvent<'static>], ) -> Vec<ValidationResult>
```

Validate multiple events

---

## Functions (events)

### amount_in

**Source**: `zero_copy/events.rs`

```rust
pub fn amount_in(&mut self) -> Option<u64>
```

Get amount in, parsing if necessary

---

### amount_out

**Source**: `zero_copy/events.rs`

```rust
pub fn amount_out(&mut self) -> Option<u64>
```

Get amount out, parsing if necessary

---

### block_number

**Source**: `zero_copy/events.rs`

```rust
pub fn block_number(&self) -> Option<u64>
```

Get block number for any lifetime

---

### event_type

**Source**: `zero_copy/events.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get event type for any lifetime

---

### get_json_data

**Source**: `zero_copy/events.rs`

```rust
pub fn get_json_data(&self) -> Option<&serde_json::Value>
```

Get JSON representation, computing if necessary

---

### get_parsed_data

**Source**: `zero_copy/events.rs`

```rust
pub fn get_parsed_data<T: std::any::Any + Send + Sync>(&self) -> Option<&T>
```

Get parsed data (strongly typed)

---

### id

**Source**: `zero_copy/events.rs`

```rust
pub fn id(&self) -> &str
```

Get id for any lifetime

---

### index

**Source**: `zero_copy/events.rs`

```rust
pub fn index(&self) -> String
```

Get index for any lifetime

---

### input_mint

**Source**: `zero_copy/events.rs`

```rust
pub fn input_mint(&mut self) -> Option<Pubkey>
```

Get input mint, parsing if necessary

---

### lp_amount

**Source**: `zero_copy/events.rs`

```rust
pub fn lp_amount(&mut self) -> Option<u64>
```

Get LP token amount, parsing if necessary

---

### new

**Source**: `zero_copy/events.rs`

```rust
pub fn new(base: ZeroCopyEvent<'a>) -> Self
```

Create new zero-copy swap event

---

### new

**Source**: `zero_copy/events.rs`

```rust
pub fn new(base: ZeroCopyEvent<'a>) -> Self
```

Create new zero-copy liquidity event

---

### new

**Source**: `jupiter/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

Creates a new EventParameters instance with the provided values

---

### new

**Source**: `jupiter/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: JupiterSwapData) -> Self
```

Creates a new JupiterSwapEvent with the provided parameters and swap data

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

Creates a new EventParameters instance with the provided values

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self
```

Creates a new MarginFi deposit event with the provided parameters and deposit data

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, withdraw_data: MarginFiWithdrawData) -> Self
```

Creates a new MarginFi withdraw event with the provided parameters and withdraw data

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, borrow_data: MarginFiBorrowData) -> Self
```

Creates a new MarginFi borrow event with the provided parameters and borrow data

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, repay_data: MarginFiRepayData) -> Self
```

Creates a new MarginFi repay event with the provided parameters and repay data

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, liquidation_data: MarginFiLiquidationData) -> Self
```

Creates a new MarginFi liquidation event with the provided parameters and liquidation data

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

Creates a new EventParameters instance with the provided values

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: MeteoraSwapData) -> Self
```

Creates a new MeteoraSwapEvent with the provided parameters and swap data

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new(params: EventParameters, liquidity_data: MeteoraLiquidityData) -> Self
```

Creates a new MeteoraLiquidityEvent with the provided parameters and liquidity data

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new(params: EventParameters, liquidity_data: MeteoraDynamicLiquidityData) -> Self
```

Creates a new MeteoraDynamicLiquidityEvent with the provided parameters and liquidity data

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

Creates a new EventParameters instance with the provided values

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: OrcaSwapData) -> Self
```

Creates a new OrcaSwapEvent with the provided parameters and swap data

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new(params: EventParameters, position_data: OrcaPositionData, is_open: bool) -> Self
```

Creates a new OrcaPositionEvent with the provided parameters and position data

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new(params: EventParameters, liquidity_data: OrcaLiquidityData) -> Self
```

Creates a new OrcaLiquidityEvent with the provided parameters and liquidity data

---

### new_borrowed

**Source**: `zero_copy/events.rs`

```rust
pub fn new_borrowed(metadata: EventMetadata, raw_data: &'a [u8]) -> Self
```

Create a new zero-copy event with borrowed data

---

### new_owned

**Source**: `zero_copy/events.rs`

```rust
pub fn new_owned(metadata: EventMetadata, raw_data: Vec<u8>) -> Self
```

Create a new zero-copy event with owned data

---

### output_mint

**Source**: `zero_copy/events.rs`

```rust
pub fn output_mint(&mut self) -> Option<Pubkey>
```

Get output mint, parsing if necessary

---

### pool_address

**Source**: `zero_copy/events.rs`

```rust
pub fn pool_address(&mut self) -> Option<Pubkey>
```

Get pool address, parsing if necessary

---

### protocol_type

**Source**: `zero_copy/events.rs`

```rust
pub fn protocol_type(&self) -> ProtocolType
```

Get protocol type for any lifetime

---

### raw_data

**Source**: `zero_copy/events.rs`

```rust
pub fn raw_data(&self) -> &[u8]
```

Get the raw data as a slice

---

### set_json_data

**Source**: `zero_copy/events.rs`

```rust
pub fn set_json_data(&mut self, json: serde_json::Value)
```

Set JSON representation (cached)

---

### set_parsed_data

**Source**: `zero_copy/events.rs`

```rust
pub fn set_parsed_data<T: std::any::Any + Send + Sync>(&mut self, data: T)
```

Set parsed data (strongly typed)

---

### signature

**Source**: `zero_copy/events.rs`

```rust
pub fn signature(&self) -> &str
```

Get signature for any lifetime

---

### slot

**Source**: `zero_copy/events.rs`

```rust
pub fn slot(&self) -> u64
```

Get slot for any lifetime

---

### timestamp

**Source**: `zero_copy/events.rs`

```rust
pub fn timestamp(&self) -> std::time::SystemTime
```

Get timestamp for any lifetime

---

### to_owned

**Source**: `zero_copy/events.rs`

```rust
pub fn to_owned(&self) -> ZeroCopyEvent<'static>
```

Convert to owned event (clones all data)

---

### token_a_amount

**Source**: `zero_copy/events.rs`

```rust
pub fn token_a_amount(&mut self) -> Option<u64>
```

Get token A amount, parsing if necessary

---

### token_b_amount

**Source**: `zero_copy/events.rs`

```rust
pub fn token_b_amount(&mut self) -> Option<u64>
```

Get token B amount, parsing if necessary

---

### with_transfer_data

**Source**: `jupiter/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Adds transfer data to the swap event

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Adds transfer data to the deposit event and returns the modified event

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this withdraw event

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this borrow event

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this repay event

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this liquidation event

---

### with_transfer_data

**Source**: `meteora/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this swap event

---

### with_transfer_data

**Source**: `meteora/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this liquidity event

---

### with_transfer_data

**Source**: `meteora/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this dynamic liquidity event

---

### with_transfer_data

**Source**: `orca/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this swap event

---

### with_transfer_data

**Source**: `orca/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this position event

---

### with_transfer_data

**Source**: `orca/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Sets the transfer data for this liquidity event

---

## Functions (parsers)

### add_parser

**Source**: `zero_copy/parsers.rs`

```rust
pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>)
```

Add a protocol-specific parser

---

### add_parser

**Source**: `zero_copy/parsers.rs`

```rust
pub fn add_parser(&mut self, parser: Arc<dyn ByteSliceEventParser>)
```

Add a parser for a specific protocol

---

### add_pattern

**Source**: `zero_copy/parsers.rs`

```rust
pub fn add_pattern(&mut self, pattern: Vec<u8>, protocol: ProtocolType)
```

Add a pattern to match

---

### data_slice

**Source**: `zero_copy/parsers.rs`

```rust
pub fn data_slice(&self, offset: usize, len: usize) -> Option<&[u8]>
```

Get a slice of the memory-mapped data

---

### find_matches

**Source**: `zero_copy/parsers.rs`

```rust
pub fn find_matches(&self, data: &[u8]) -> Vec<(usize, ProtocolType)>
```

Find matching patterns in data

In a real implementation, this would use SIMD instructions for parallel matching
For now, we provide a basic implementation

---

### from_file

**Source**: `zero_copy/parsers.rs`

```rust
pub fn from_file(file_path: &str) -> Result<Self, ParseError>
```

Create a new memory-mapped parser from a file

---

### get_client

**Source**: `zero_copy/parsers.rs`

```rust
pub fn get_client(&self) -> Arc<solana_client::rpc_client::RpcClient>
```

Get the next client in round-robin fashion

---

### get_stats

**Source**: `zero_copy/parsers.rs`

```rust
pub fn get_stats(&self) -> BatchParserStats
```

Get statistics about the batch parser

---

### match_discriminator

**Source**: `zero_copy/parsers.rs`

```rust
pub fn match_discriminator(&self, data: &[u8]) -> Option<ProtocolType>
```

Fast prefix matching for instruction discriminators

---

### new

**Source**: `zero_copy/parsers.rs`

```rust
pub fn new(data: &'a [u8]) -> Self
```

Create a new deserializer

---

### new

**Source**: `zero_copy/parsers.rs`

```rust
pub fn new(max_batch_size: usize) -> Self
```

Create a new batch parser

---

### new

**Source**: `zero_copy/parsers.rs`

```rust
pub fn new(urls: Vec<String>) -> Self
```

Create a new connection pool

---

### parse_all

**Source**: `zero_copy/parsers.rs`

```rust
pub fn parse_all(&self) -> Result<Vec<ZeroCopyEvent<'_>>, ParseError>
```

Parse events from the memory-mapped data

---

### parse_batch

**Source**: `zero_copy/parsers.rs`

```rust
pub fn parse_batch<'a>( &self, batch: &'a [&'a [u8]], metadatas: Vec<EventMetadata>, ) -> Result<Vec<ZeroCopyEvent<'a>>, ParseError>
```

Parse a batch of transaction data

---

### position

**Source**: `zero_copy/parsers.rs`

```rust
pub fn position(&self) -> usize
```

Get current position

---

### read_bytes

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParseError>
```

Read a byte array of fixed size

---

### read_pubkey

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_pubkey(&mut self) -> Result<solana_sdk::pubkey::Pubkey, ParseError>
```

Read a Pubkey (32 bytes)

---

### read_u32_le

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u32_le(&mut self) -> Result<u32, ParseError>
```

Read a u32 value (little endian)

---

### read_u64_le

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u64_le(&mut self) -> Result<u64, ParseError>
```

Read a u64 value (little endian)

---

### read_u8

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u8(&mut self) -> Result<u8, ParseError>
```

Read a u8 value

---

### remaining

**Source**: `zero_copy/parsers.rs`

```rust
pub fn remaining(&self) -> usize
```

Get remaining bytes

---

### remaining_data

**Source**: `zero_copy/parsers.rs`

```rust
pub fn remaining_data(&self) -> &'a [u8]
```

Get remaining data as slice

---

### size

**Source**: `zero_copy/parsers.rs`

```rust
pub fn size(&self) -> usize
```

Get the total size of the mapped data

---

### size

**Source**: `zero_copy/parsers.rs`

```rust
pub fn size(&self) -> usize
```

Get pool size

---

### skip

**Source**: `zero_copy/parsers.rs`

```rust
pub fn skip(&mut self, len: usize) -> Result<(), ParseError>
```

Skip bytes

---

## Functions (traits)

### new

**Source**: `core/traits.rs`

```rust
pub fn new( program_ids: Vec<solana_sdk::pubkey::Pubkey>, configs: Vec<GenericEventParseConfig>, ) -> Self
```

Create new generic event parser

---

## Functions (parser)

### new

**Source**: `bonk/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Bonk event parser with configured discriminators and parsers

---

### new

**Source**: `jupiter/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Jupiter event parser with default configurations for routing and exact-out routing

---

### new

**Source**: `meteora/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Meteora event parser with default configurations

---

### new

**Source**: `pumpswap/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new PumpSwap event parser with default configuration

---

### new

**Source**: `raydium_amm_v4/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Raydium AMM V4 event parser with configured discriminators and instruction parsers

---

### new

**Source**: `raydium_clmm/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Raydium CLMM event parser

Initializes the parser with all supported Raydium CLMM instruction types
including swaps, position management, and pool creation operations.

---

### new

**Source**: `raydium_cpmm/parser.rs`

```rust
pub fn new() -> Self
```

Creates a new Raydium CPMM event parser with default configuration

---


---

*This documentation was automatically generated from the source code.*