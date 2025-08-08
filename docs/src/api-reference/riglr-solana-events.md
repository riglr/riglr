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

### Functions

- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`add_parser`](#add_parser)
- [`add_pattern`](#add_pattern)
- [`add_protocol_parser`](#add_protocol_parser)
- [`add_rule`](#add_rule)
- [`amount_in`](#amount_in)
- [`amount_out`](#amount_out)
- [`amount_to_shares`](#amount_to_shares)
- [`bin_id_to_price`](#bin_id_to_price)
- [`block_number`](#block_number)
- [`build`](#build)
- [`bytes`](#bytes)
- [`calculate_active_bin_price`](#calculate_active_bin_price)
- [`calculate_health_ratio`](#calculate_health_ratio)
- [`calculate_interest_rate`](#calculate_interest_rate)
- [`calculate_liquidation_threshold`](#calculate_liquidation_threshold)
- [`calculate_liquidity_distribution`](#calculate_liquidity_distribution)
- [`calculate_price_impact`](#calculate_price_impact)
- [`create_fast`](#create_fast)
- [`create_fast`](#create_fast)
- [`create_standard`](#create_standard)
- [`create_standard`](#create_standard)
- [`create_standard`](#create_standard)
- [`create_zero_copy`](#create_zero_copy)
- [`create_zero_copy`](#create_zero_copy)
- [`create_zero_copy`](#create_zero_copy)
- [`create_zero_copy`](#create_zero_copy)
- [`data_slice`](#data_slice)
- [`decode_base58`](#decode_base58)
- [`discriminator_matches`](#discriminator_matches)
- [`encode_base58`](#encode_base58)
- [`enrich_event`](#enrich_event)
- [`enrich_events`](#enrich_events)
- [`event_type`](#event_type)
- [`event_type`](#event_type)
- [`event_type`](#event_type)
- [`event_type`](#event_type)
- [`event_type`](#event_type)
- [`extract_account_keys`](#extract_account_keys)
- [`extract_data`](#extract_data)
- [`extract_discriminator`](#extract_discriminator)
- [`find_matches`](#find_matches)
- [`format_token_amount`](#format_token_amount)
- [`from_byte`](#from_byte)
- [`from_byte`](#from_byte)
- [`from_bytes`](#from_bytes)
- [`from_core_metadata`](#from_core_metadata)
- [`from_event_kind`](#from_event_kind)
- [`from_file`](#from_file)
- [`from_u64`](#from_u64)
- [`get_cache_stats`](#get_cache_stats)
- [`get_client`](#get_client)
- [`get_json_data`](#get_json_data)
- [`get_parsed_data`](#get_parsed_data)
- [`get_parser`](#get_parser)
- [`get_parser_for_program`](#get_parser_for_program)
- [`get_stats`](#get_stats)
- [`get_stats`](#get_stats)
- [`get_stats`](#get_stats)
- [`has_discriminator`](#has_discriminator)
- [`id`](#id)
- [`index`](#index)
- [`input_mint`](#input_mint)
- [`invalid_account_index`](#invalid_account_index)
- [`invalid_discriminator`](#invalid_discriminator)
- [`invalid_enum_variant`](#invalid_enum_variant)
- [`is_jupiter_v6_program`](#is_jupiter_v6_program)
- [`is_marginfi_program`](#is_marginfi_program)
- [`is_meteora_dlmm_program`](#is_meteora_dlmm_program)
- [`is_meteora_dynamic_program`](#is_meteora_dynamic_program)
- [`is_orca_whirlpool_program`](#is_orca_whirlpool_program)
- [`jupiter_v6_program_id`](#jupiter_v6_program_id)
- [`liquidity`](#liquidity)
- [`lp_amount`](#lp_amount)
- [`marginfi_bank_program_id`](#marginfi_bank_program_id)
- [`marginfi_program_id`](#marginfi_program_id)
- [`match_discriminator`](#match_discriminator)
- [`meteora_dlmm_program_id`](#meteora_dlmm_program_id)
- [`meteora_dynamic_program_id`](#meteora_dynamic_program_id)
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
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new_borrowed`](#new_borrowed)
- [`new_fast`](#new_fast)
- [`new_fast`](#new_fast)
- [`new_owned`](#new_owned)
- [`new_standard`](#new_standard)
- [`new_standard`](#new_standard)
- [`new_standard`](#new_standard)
- [`not_enough_bytes`](#not_enough_bytes)
- [`orca_whirlpool_program_id`](#orca_whirlpool_program_id)
- [`output_mint`](#output_mint)
- [`parse_all`](#parse_all)
- [`parse_batch`](#parse_batch)
- [`parse_events_from_inner_instruction`](#parse_events_from_inner_instruction)
- [`parse_events_from_instruction`](#parse_events_from_instruction)
- [`parse_inner_instruction`](#parse_inner_instruction)
- [`parse_instruction`](#parse_instruction)
- [`parse_pubkey_from_bytes`](#parse_pubkey_from_bytes)
- [`parse_u128_le`](#parse_u128_le)
- [`parse_u16_le`](#parse_u16_le)
- [`parse_u32_le`](#parse_u32_le)
- [`parse_u64_le`](#parse_u64_le)
- [`pool_address`](#pool_address)
- [`position`](#position)
- [`process_stream`](#process_stream)
- [`protocol_event`](#protocol_event)
- [`protocol_type`](#protocol_type)
- [`raw_data`](#raw_data)
- [`read_bytes`](#read_bytes)
- [`read_i32_le`](#read_i32_le)
- [`read_option_bool`](#read_option_bool)
- [`read_pubkey`](#read_pubkey)
- [`read_u128_le`](#read_u128_le)
- [`read_u32_le`](#read_u32_le)
- [`read_u32_le`](#read_u32_le)
- [`read_u64_le`](#read_u64_le)
- [`read_u64_le`](#read_u64_le)
- [`read_u8`](#read_u8)
- [`read_u8_le`](#read_u8_le)
- [`remaining`](#remaining)
- [`remaining_data`](#remaining_data)
- [`set_id`](#set_id)
- [`set_json_data`](#set_json_data)
- [`set_parsed_data`](#set_parsed_data)
- [`shares_to_amount`](#shares_to_amount)
- [`should_handle`](#should_handle)
- [`signature`](#signature)
- [`size`](#size)
- [`size`](#size)
- [`skip`](#skip)
- [`slot`](#slot)
- [`sqrt_price_to_price`](#sqrt_price_to_price)
- [`supported_program_ids`](#supported_program_ids)
- [`supports_program`](#supports_program)
- [`swap`](#swap)
- [`system_time_to_millis`](#system_time_to_millis)
- [`take_output_receiver`](#take_output_receiver)
- [`tick_index_to_price`](#tick_index_to_price)
- [`timestamp`](#timestamp)
- [`to_core_metadata`](#to_core_metadata)
- [`to_event_kind`](#to_event_kind)
- [`to_owned`](#to_owned)
- [`to_token_amount`](#to_token_amount)
- [`token_a_amount`](#token_a_amount)
- [`token_b_amount`](#token_b_amount)
- [`validate_event`](#validate_event)
- [`validate_events`](#validate_events)
- [`with_all_parsers`](#with_all_parsers)
- [`with_batch_size`](#with_batch_size)
- [`with_batch_timeout`](#with_batch_timeout)
- [`with_concurrency_limit`](#with_concurrency_limit)
- [`with_data`](#with_data)
- [`with_legacy_parser`](#with_legacy_parser)
- [`with_metadata`](#with_metadata)
- [`with_metrics`](#with_metrics)
- [`with_parser_config`](#with_parser_config)
- [`with_received_time`](#with_received_time)
- [`with_received_time`](#with_received_time)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)
- [`with_transfer_data`](#with_transfer_data)

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
- [`EventMetadata`](#eventmetadata)
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

### Traits

- [`ByteSliceEventParser`](#bytesliceeventparser)
- [`EventParser`](#eventparser)
- [`ToSolanaEvent`](#tosolanaevent)
- [`ValidationRule`](#validationrule)

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

---

### BUY_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const BUY_EVENT_BYTES: &[u8]
```

---

### BUY_EXACT_IN_IX

**Source**: `bonk/events.rs`

```rust
const BUY_EXACT_IN_IX: &[u8]
```

---

### BUY_EXACT_OUT_IX

**Source**: `bonk/events.rs`

```rust
const BUY_EXACT_OUT_IX: &[u8]
```

---

### BUY_IX

**Source**: `pumpswap/events.rs`

```rust
const BUY_IX: &[u8]
```

---

### CLOSE_POSITION

**Source**: `raydium_clmm/discriminators.rs`

```rust
const CLOSE_POSITION: &[u8]
```

---

### CLOSE_POSITION_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const CLOSE_POSITION_DISCRIMINATOR: [u8; 8]
```

---

### CREATE_POOL

**Source**: `raydium_clmm/discriminators.rs`

```rust
const CREATE_POOL: &[u8]
```

---

### CREATE_POOL_EVENT

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_EVENT: &str
```

---

### CREATE_POOL_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_EVENT_BYTES: &[u8]
```

---

### CREATE_POOL_IX

**Source**: `pumpswap/events.rs`

```rust
const CREATE_POOL_IX: &[u8]
```

---

### DECREASE_LIQUIDITY_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const DECREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### DECREASE_LIQUIDITY_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const DECREASE_LIQUIDITY_V2: &[u8]
```

---

### DEPOSIT

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const DEPOSIT: &[u8]
```

---

### DEPOSIT_EVENT

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_EVENT: &str
```

---

### DEPOSIT_EVENT

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_EVENT: &str
```

---

### DEPOSIT_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_EVENT_BYTES: &[u8]
```

---

### DEPOSIT_EVENT_BYTES

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_EVENT_BYTES: &[u8]
```

---

### DEPOSIT_IX

**Source**: `pumpswap/events.rs`

```rust
const DEPOSIT_IX: &[u8]
```

---

### DEPOSIT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const DEPOSIT_IX: &[u8]
```

---

### DLMM_ADD_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### DLMM_SWAP_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DLMM_SWAP_DISCRIMINATOR: [u8; 8]
```

Meteora instruction discriminators

---

### DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DYNAMIC_ADD_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR

**Source**: `meteora/types.rs`

```rust
const DYNAMIC_REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### EXACT_OUT_ROUTE_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8]
```

---

### INCREASE_LIQUIDITY_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const INCREASE_LIQUIDITY_DISCRIMINATOR: [u8; 8]
```

---

### INCREASE_LIQUIDITY_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const INCREASE_LIQUIDITY_V2: &[u8]
```

---

### INITIALIZE2

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const INITIALIZE2: &[u8]
```

---

### INITIALIZE_IX

**Source**: `bonk/events.rs`

```rust
const INITIALIZE_IX: &[u8]
```

---

### INITIALIZE_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const INITIALIZE_IX: &[u8]
```

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

---

### LEGACY_ROUTE_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const LEGACY_ROUTE_DISCRIMINATOR: [u8; 8]
```

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

---

### MARGINFI_DEPOSIT_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_DEPOSIT_DISCRIMINATOR: [u8; 8]
```

MarginFi instruction discriminators

---

### MARGINFI_LIQUIDATE_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_LIQUIDATE_DISCRIMINATOR: [u8; 8]
```

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

---

### MARGINFI_WITHDRAW_DISCRIMINATOR

**Source**: `marginfi/types.rs`

```rust
const MARGINFI_WITHDRAW_DISCRIMINATOR: [u8; 8]
```

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

---

### MIGRATE_TO_CPSWAP_IX

**Source**: `bonk/events.rs`

```rust
const MIGRATE_TO_CPSWAP_IX: &[u8]
```

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

---

### OPEN_POSITION_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const OPEN_POSITION_V2: &[u8]
```

---

### OPEN_POSITION_WITH_TOKEN_22_NFT

**Source**: `raydium_clmm/discriminators.rs`

```rust
const OPEN_POSITION_WITH_TOKEN_22_NFT: &[u8]
```

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

---

### POOL_CREATE_EVENT_BYTES

**Source**: `bonk/events.rs`

```rust
const POOL_CREATE_EVENT_BYTES: &[u8]
```

---

### PUBKEY_RANGE

**Source**: `src/constants.rs`

```rust
const PUBKEY_RANGE: std::ops::Range<usize>
```

---

### PUBKEY_SIZE

**Source**: `src/constants.rs`

```rust
const PUBKEY_SIZE: usize
```

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

---

### SELL_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const SELL_EVENT_BYTES: &[u8]
```

---

### SELL_EXACT_IN_IX

**Source**: `bonk/events.rs`

```rust
const SELL_EXACT_IN_IX: &[u8]
```

---

### SELL_EXACT_OUT_IX

**Source**: `bonk/events.rs`

```rust
const SELL_EXACT_OUT_IX: &[u8]
```

---

### SELL_IX

**Source**: `pumpswap/events.rs`

```rust
const SELL_IX: &[u8]
```

---

### SWAP

**Source**: `raydium_clmm/discriminators.rs`

```rust
const SWAP: &[u8]
```

Raydium CLMM instruction discriminators

---

### SWAP_BASE_IN

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const SWAP_BASE_IN: &[u8]
```

Raydium AMM V4 instruction discriminators

---

### SWAP_BASE_INPUT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_BASE_INPUT_IX: &[u8]
```

---

### SWAP_BASE_OUT

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const SWAP_BASE_OUT: &[u8]
```

---

### SWAP_BASE_OUTPUT_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_BASE_OUTPUT_IX: &[u8]
```

---

### SWAP_DISCRIMINATOR

**Source**: `jupiter/types.rs`

```rust
const SWAP_DISCRIMINATOR: [u8; 8]
```

---

### SWAP_DISCRIMINATOR

**Source**: `orca/types.rs`

```rust
const SWAP_DISCRIMINATOR: [u8; 8]
```

Orca Whirlpool instruction discriminators (calculated from Anchor's "global:<instruction_name>")

---

### SWAP_EVENT

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_EVENT: &str
```

---

### SWAP_EVENT_BYTES

**Source**: `raydium_cpmm/events.rs`

```rust
const SWAP_EVENT_BYTES: &[u8]
```

---

### SWAP_V2

**Source**: `raydium_clmm/discriminators.rs`

```rust
const SWAP_V2: &[u8]
```

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

---

### TRADE_EVENT_BYTES

**Source**: `bonk/events.rs`

```rust
const TRADE_EVENT_BYTES: &[u8]
```

---

### U128_RANGE

**Source**: `src/constants.rs`

```rust
const U128_RANGE: std::ops::Range<usize>
```

---

### U128_SIZE

**Source**: `src/constants.rs`

```rust
const U128_SIZE: usize
```

---

### U16_RANGE

**Source**: `src/constants.rs`

```rust
const U16_RANGE: std::ops::Range<usize>
```

---

### U16_SIZE

**Source**: `src/constants.rs`

```rust
const U16_SIZE: usize
```

---

### U32_RANGE

**Source**: `src/constants.rs`

```rust
const U32_RANGE: std::ops::Range<usize>
```

---

### U32_SIZE

**Source**: `src/constants.rs`

```rust
const U32_SIZE: usize
```

---

### U64_RANGE

**Source**: `src/constants.rs`

```rust
const U64_RANGE: std::ops::Range<usize>
```

---

### U64_SIZE

**Source**: `src/constants.rs`

```rust
const U64_SIZE: usize
```

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

---

### WITHDRAW_EVENT

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_EVENT: &str
```

---

### WITHDRAW_EVENT_BYTES

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_EVENT_BYTES: &[u8]
```

---

### WITHDRAW_IX

**Source**: `pumpswap/events.rs`

```rust
const WITHDRAW_IX: &[u8]
```

---

### WITHDRAW_IX

**Source**: `raydium_cpmm/events.rs`

```rust
const WITHDRAW_IX: &[u8]
```

---

### WITHDRAW_PNL

**Source**: `raydium_amm_v4/discriminators.rs`

```rust
const WITHDRAW_PNL: &[u8]
```

---

## Enums

### EnrichmentError

**Source**: `pipelines/enrichment.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum EnrichmentError { #[error("HTTP request failed: {0}")] HttpError(#[from] reqwest::Error), #[error("Serialization error: {0}")] SerializationError(#[from] serde_json::Error), #[error("Task execution error: {0}")] TaskError(String), #[error("Cache error: {0}")] CacheError(String), #[error("API rate limit exceeded")] RateLimitExceeded, }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshDeserialize, Default)]
```

```rust
pub enum EventType { Swap, AddLiquidity, RemoveLiquidity, Borrow, Repay, Liquidate, Transfer, Mint, Burn, CreatePool, UpdatePool, Transaction, Block, ContractEvent, PriceUpdate, OrderBook, Trade, FeeUpdate, BonkBuyExactIn, BonkBuyExactOut, BonkSellExactIn, BonkSellExactOut, BonkInitialize, BonkMigrateToAmm, BonkMigrateToCpswap, PumpSwapBuy, PumpSwapSell, PumpSwapCreatePool, PumpSwapDeposit, PumpSwapWithdraw, PumpSwapSetParams, RaydiumSwap, RaydiumDeposit, RaydiumWithdraw, RaydiumAmmV4SwapBaseIn, RaydiumAmmV4SwapBaseOut, RaydiumAmmV4Deposit, RaydiumAmmV4Initialize2, RaydiumAmmV4Withdraw, RaydiumAmmV4WithdrawPnl, RaydiumClmmSwap, RaydiumClmmSwapV2, RaydiumClmmCreatePool, RaydiumClmmOpenPositionV2, RaydiumClmmIncreaseLiquidityV2, RaydiumClmmDecreaseLiquidityV2, RaydiumClmmClosePosition, RaydiumClmmOpenPositionWithToken22Nft, RaydiumCpmmSwap, RaydiumCpmmSwapBaseInput, RaydiumCpmmSwapBaseOutput, RaydiumCpmmDeposit, RaydiumCpmmWithdraw, RaydiumCpmmCreatePool, OpenPosition, ClosePosition, IncreaseLiquidity, DecreaseLiquidity, Deposit, Withdraw, #[default] Unknown, }
```

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
pub enum MarginFiAccountType { MarginfiGroup, MarginfiAccount, Bank, Unknown, }
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
pub enum MetaplexAuctionHouseDiscriminator { Buy, Sell, ExecuteSale, Deposit, Withdraw, Cancel, }
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
pub enum MetaplexTokenMetadataDiscriminator { CreateMetadataAccount = 0, UpdateMetadataAccount = 1, DeprecatedCreateMasterEdition = 2, DeprecatedMintNewEditionFromMasterEditionViaPrintingToken = 3, UpdatePrimarySaleHappenedViaToken = 4, DeprecatedSetReservationList = 5, DeprecatedCreateReservationList = 6, SignMetadata = 7, DeprecatedMintPrintingTokensViaToken = 8, DeprecatedMintPrintingTokens = 9, CreateMasterEdition = 10, MintNewEditionFromMasterEditionViaToken = 11, ConvertMasterEditionV1ToV2 = 12, MintNewEditionFromMasterEditionViaVaultProxy = 13, PuffMetadata = 14, UpdateMetadataAccountV2 = 15, CreateMetadataAccountV2 = 16, CreateMasterEditionV3 = 17, VerifyCollection = 18, Utilize = 19, ApproveUseAuthority = 20, RevokeUseAuthority = 21, UnverifyCollection = 22, ApproveCollectionAuthority = 23, RevokeCollectionAuthority = 24, SetAndVerifyCollection = 25, FreezeDelegatedAccount = 26, ThawDelegatedAccount = 27, RemoveCreatorVerification = 28, BurnNft = 29, VerifyCreator = 30, UnverifyCreator = 31, BubblegumSetCollectionSize = 32, BurnEditionNft = 33, CreateMetadataAccountV3 = 34, SetCollectionSize = 35, SetTokenStandard = 36, BubblegumVerifyCreator = 37, BubblegumUnverifyCreator = 38, BubblegumVerifyCollection = 39, BubblegumUnverifyCollection = 40, BubblegumSetAndVerifyCollection = 41, Transfer = 42, }
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
pub enum ParseError { /// Not enough bytes available for the requested operation #[error("Not enough bytes: expected {expected}, got {found} at offset {offset}")] NotEnoughBytes { expected: usize, found: usize, offset: usize, }, /// Invalid discriminator for the instruction #[error("Invalid discriminator: expected {expected:?}, got {found:?}")] InvalidDiscriminator { expected: Vec<u8>, found: Vec<u8> }, /// Invalid account index #[error("Account index {index} out of bounds (max: {max})")] InvalidAccountIndex { index: usize, max: usize }, /// Invalid public key format #[error("Invalid public key: {0}")] InvalidPubkey(String), /// UTF-8 decoding error #[error("UTF-8 decoding error: {0}")] Utf8Error(#[from] std::str::Utf8Error), /// Borsh deserialization error #[error("Borsh deserialization error: {0}")] BorshError(String), /// Invalid enum variant #[error("Invalid enum variant {variant} for type {type_name}")] InvalidEnumVariant { variant: u8, type_name: String }, /// Invalid instruction type #[error("Invalid instruction type: {0}")] InvalidInstructionType(String), /// Missing required field #[error("Missing required field: {0}")] MissingField(String), /// Invalid data format #[error("Invalid data format: {0}")] InvalidDataFormat(String), /// Overflow error #[error("Arithmetic overflow: {0}")] Overflow(String), /// Generic parsing error #[error("Parse error: {0}")] Generic(String), /// Network error (for streaming) #[error("Network error: {0}")] Network(String), /// Timeout error #[error("Operation timed out: {0}")] Timeout(String), }
```

Custom error type for event parsing operations

NOTE: This will be gradually replaced with EventError from riglr-events-core

**Variants**:

- `NotEnoughBytes`
- `expected`
- `found`
- `offset`
- `InvalidDiscriminator`
- `InvalidAccountIndex`
- `InvalidPubkey(String)`
- `Utf8Error(#[from] std::str::Utf8Error)`
- `BorshError(String)`
- `InvalidEnumVariant`
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
pub enum ParseError { #[error("Invalid instruction data: {0}")] InvalidInstructionData(String), #[error("Insufficient data length: expected {expected}, got {actual}")] InsufficientData { expected: usize, actual: usize }, #[error("Unknown discriminator: {discriminator:?}")] UnknownDiscriminator { discriminator: Vec<u8> }, #[error("Deserialization error: {0}")] DeserializationError(#[from] borsh::io::Error), #[error("Memory map error: {0}")] MemoryMapError(String), }
```

Error type for parsing operations

**Variants**:

- `InvalidInstructionData(String)`
- `InsufficientData`
- `UnknownDiscriminator`
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
pub enum PipelineError { #[error("Semaphore error")] SemaphoreError(()), #[error("Channel error")] ChannelError, #[error("Parse error: {0}")] ParseError(#[from] ParseError), #[error("Configuration error: {0}")] ConfigError(String), }
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
pub enum PoolStatus { #[default] Fund, Migrate, Trade, }
```

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
pub enum Protocol { OrcaWhirlpool, MeteoraDlmm, MarginFi, Jupiter, RaydiumAmmV4, RaydiumClmm, RaydiumCpmm, PumpFun, PumpSwap, Bonk, Custom(String), }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, BorshDeserialize)]
```

```rust
pub enum ProtocolType { OrcaWhirlpool, MeteoraDlmm, MarginFi, Bonk, PumpSwap, RaydiumAmm, RaydiumAmmV4, RaydiumClmm, RaydiumCpmm, Jupiter, Other(String), }
```

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
pub enum PumpFunDiscriminator { Buy, Sell, CreatePool, Deposit, Withdraw, SetParams, }
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
pub enum RaydiumV4Discriminator { SwapBaseIn = 0x09, SwapBaseOut = 0x0a, Deposit = 0x03, Withdraw = 0x04, Initialize2 = 0x00, WithdrawPnl = 0x05, }
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
pub enum SwapDirection { AtoB, BtoA, }
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
```

```rust
pub enum SwapDirection { #[default] BaseIn, BaseOut, }
```

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
pub enum TradeDirection { #[default] Buy, Sell, }
```

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
pub enum ValidationError { /// Missing required field MissingField { field: String }, /// Invalid field value InvalidValue { field: String, reason: String }, /// Data inconsistency Inconsistency { description: String }, /// Business logic violation BusinessLogicError { rule: String, description: String }, /// Event too old StaleEvent { age: Duration }, /// Duplicate event detected Duplicate { original_id: String }, }
```

Validation error types

**Variants**:

- `MissingField`
- `InvalidValue`
- `Inconsistency`
- `BusinessLogicError`
- `StaleEvent`
- `Duplicate`

---

### ValidationWarning

**Source**: `pipelines/validation.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub enum ValidationWarning { /// Unusual but potentially valid value UnusualValue { field: String, value: String }, /// Deprecated field usage DeprecatedField { field: String }, /// Performance concern PerformanceWarning { description: String }, }
```

Validation warning types

**Variants**:

- `UnusualValue`
- `DeprecatedField`
- `PerformanceWarning`

---

## Functions

### add_parser

**Source**: `events/factory.rs`

```rust
pub fn add_parser(&mut self, protocol: Protocol, parser: Arc<dyn EventParser>)
```

Add a parser for a specific protocol

---

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

### add_protocol_parser

**Source**: `src/solana_parser.rs`

```rust
pub fn add_protocol_parser<P>( &mut self, _protocol: Protocol, _parser: Arc<dyn LegacyEventParser>, ) where P: LegacyEventParser + 'static,
```

Add a protocol parser

---

### add_rule

**Source**: `pipelines/validation.rs`

```rust
pub fn add_rule(&mut self, rule: Arc<dyn ValidationRule>)
```

Add a custom validation rule

---

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

### block_number

**Source**: `zero_copy/events.rs`

```rust
pub fn block_number(&self) -> Option<u64>
```

Get block number for any lifetime

---

### build

**Source**: `pipelines/parsing.rs`

```rust
pub fn build(self) -> ParsingPipeline
```

Build the parsing pipeline

---

### bytes

**Source**: `parsers/jupiter.rs`

```rust
pub fn bytes(&self) -> &'static [u8; 8]
```

Get discriminator bytes

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

### calculate_price_impact

**Source**: `common/utils.rs`

```rust
pub fn calculate_price_impact( amount_in: u64, amount_out: u64, reserve_in: u64, reserve_out: u64, ) -> f64
```

Calculate price impact for a swap

---

### create_fast

**Source**: `parsers/jupiter.rs`

```rust
pub fn create_fast() -> Arc<dyn ByteSliceEventParser>
```

Create a fast parser with simplified analysis

---

### create_fast

**Source**: `parsers/metaplex.rs`

```rust
pub fn create_fast() -> Arc<dyn ByteSliceEventParser>
```

Create a fast parser with minimal metadata

---

### create_standard

**Source**: `parsers/pump_fun.rs`

```rust
pub fn create_standard() -> Arc<dyn ByteSliceEventParser>
```

Create a standard parser

---

### create_standard

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn create_standard() -> Arc<dyn ByteSliceEventParser>
```

Create a standard parser

---

### create_standard

**Source**: `parsers/jupiter.rs`

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

### create_zero_copy

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser

---

### create_zero_copy

**Source**: `parsers/jupiter.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser with full analysis

---

### create_zero_copy

**Source**: `parsers/metaplex.rs`

```rust
pub fn create_zero_copy() -> Arc<dyn ByteSliceEventParser>
```

Create a new high-performance zero-copy parser with full metadata

---

### data_slice

**Source**: `zero_copy/parsers.rs`

```rust
pub fn data_slice(&self, offset: usize, len: usize) -> Option<&[u8]>
```

Get a slice of the memory-mapped data

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

---

### encode_base58

**Source**: `common/utils.rs`

```rust
pub fn encode_base58(data: &[u8]) -> String
```

Encode bytes to base58 string

---

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

### event_type

**Source**: `parsers/pump_fun.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### event_type

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### event_type

**Source**: `parsers/jupiter.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### event_type

**Source**: `parsers/metaplex.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get corresponding event type

---

### event_type

**Source**: `zero_copy/events.rs`

```rust
pub fn event_type(&self) -> EventType
```

Get event type for any lifetime

---

### extract_account_keys

**Source**: `common/utils.rs`

```rust
pub fn extract_account_keys( accounts: &[Pubkey], indices: &[usize], ) -> Result<Vec<Pubkey>, Box<dyn std::error::Error + Send + Sync>>
```

Extract account keys from accounts array

---

### extract_data

**Source**: `src/solana_events.rs`

```rust
pub fn extract_data<T>(&self) -> Result<T, serde_json::Error> where T: for<'de> Deserialize<'de>,
```

Extract typed data from the event

---

### extract_discriminator

**Source**: `common/utils.rs`

```rust
pub fn extract_discriminator(data: &[u8], size: usize) -> Option<Vec<u8>>
```

Extract discriminator from instruction data

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

### format_token_amount

**Source**: `common/utils.rs`

```rust
pub fn format_token_amount(amount: u64, decimals: u8) -> f64
```

Convert amount with decimals to human-readable format

---

### from_byte

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn from_byte(byte: u8) -> Option<Self>
```

Parse discriminator from byte

---

### from_byte

**Source**: `parsers/metaplex.rs`

```rust
pub fn from_byte(byte: u8) -> Option<Self>
```

Parse discriminator from byte

---

### from_bytes

**Source**: `parsers/jupiter.rs`

```rust
pub fn from_bytes(bytes: &[u8]) -> Option<Self>
```

Parse discriminator from first 8 bytes

---

### from_core_metadata

**Source**: `src/types.rs`

```rust
pub fn from_core_metadata( core_metadata: &riglr_events_core::types::EventMetadata, protocol_type: ProtocolType, event_type: EventType, program_id: Pubkey, index: String, ) -> Self
```

Convert from riglr-events-core EventMetadata to local EventMetadata

---

### from_event_kind

**Source**: `src/types.rs`

```rust
pub fn from_event_kind(kind: &riglr_events_core::types::EventKind) -> Self
```

Convert from riglr-events-core EventKind to local EventType

---

### from_file

**Source**: `zero_copy/parsers.rs`

```rust
pub fn from_file(file_path: &str) -> Result<Self, ParseError>
```

Create a new memory-mapped parser from a file

---

### from_u64

**Source**: `parsers/pump_fun.rs`

```rust
pub fn from_u64(value: u64) -> Option<Self>
```

Parse discriminator from first 8 bytes (u64 little-endian)

---

### get_cache_stats

**Source**: `pipelines/enrichment.rs`

```rust
pub async fn get_cache_stats(&self) -> CacheStats
```

Get cache statistics

---

### get_client

**Source**: `zero_copy/parsers.rs`

```rust
pub fn get_client(&self) -> Arc<solana_client::rpc_client::RpcClient>
```

Get the next client in round-robin fashion

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

### get_stats

**Source**: `pipelines/parsing.rs`

```rust
pub fn get_stats(&self) -> PipelineStats
```

Get pipeline statistics

---

### get_stats

**Source**: `pipelines/validation.rs`

```rust
pub async fn get_stats(&self) -> ValidationStats
```

Get pipeline statistics

---

### get_stats

**Source**: `zero_copy/parsers.rs`

```rust
pub fn get_stats(&self) -> BatchParserStats
```

Get statistics about the batch parser

---

### has_discriminator

**Source**: `common/utils.rs`

```rust
pub fn has_discriminator(data: &[u8], discriminator: &[u8]) -> bool
```

Check if instruction data starts with a specific discriminator

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

### liquidity

**Source**: `src/solana_events.rs`

```rust
pub fn liquidity(params: LiquidityEventParams) -> Self
```

Create a liquidity event

---

### lp_amount

**Source**: `zero_copy/events.rs`

```rust
pub fn lp_amount(&mut self) -> Option<u64>
```

Get LP token amount, parsing if necessary

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

### match_discriminator

**Source**: `zero_copy/parsers.rs`

```rust
pub fn match_discriminator(&self, data: &[u8]) -> Option<ProtocolType>
```

Fast prefix matching for instruction discriminators

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

### new

**Source**: `src/solana_events.rs`

```rust
pub fn new(legacy_metadata: EventMetadata, data: serde_json::Value) -> Self
```

Create a new Solana event from legacy metadata

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

### new

**Source**: `src/types.rs`

**Attributes**:
```rust
#[allow(clippy::too_many_arguments)]
```

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, protocol_type: ProtocolType, event_type: EventType, program_id: Pubkey, index: String, program_received_time_ms: i64, ) -> Self
```

---

### new

**Source**: `events/factory.rs`

```rust
pub fn new() -> Self
```

Create a new event parser registry

---

### new

**Source**: `parsers/pump_fun.rs`

```rust
pub fn new() -> Self
```

Create new PumpFun parser

---

### new

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn new() -> Self
```

Create new Raydium V4 parser

---

### new

**Source**: `parsers/jupiter.rs`

```rust
pub fn new() -> Self
```

Create new Jupiter parser with full analysis

---

### new

**Source**: `parsers/metaplex.rs`

```rust
pub fn new() -> Self
```

Create new Metaplex parser

---

### new

**Source**: `pipelines/enrichment.rs`

```rust
pub fn new(config: EnrichmentConfig) -> Self
```

Create a new event enricher

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

### new

**Source**: `pipelines/validation.rs`

```rust
pub fn new(config: ValidationConfig) -> Self
```

Create a new validation pipeline

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

**Source**: `zero_copy/parsers.rs`

```rust
pub fn new() -> Self
```

Create a new SIMD pattern matcher

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

### new

**Source**: `core/traits.rs`

```rust
pub fn new( program_ids: Vec<solana_sdk::pubkey::Pubkey>, configs: Vec<GenericEventParseConfig>, ) -> Self
```

Create new generic event parser

---

### new

**Source**: `bonk/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `jupiter/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

---

### new

**Source**: `jupiter/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: JupiterSwapData) -> Self
```

---

### new

**Source**: `jupiter/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

---

### new

**Source**: `marginfi/events.rs`

```rust
pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self
```

---

### new

**Source**: `marginfi/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

---

### new

**Source**: `meteora/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: MeteoraSwapData) -> Self
```

---

### new

**Source**: `meteora/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new( id: String, signature: String, slot: u64, block_time: i64, block_time_ms: i64, program_received_time_ms: i64, index: String, ) -> Self
```

---

### new

**Source**: `orca/events.rs`

```rust
pub fn new(params: EventParameters, swap_data: OrcaSwapData) -> Self
```

---

### new

**Source**: `orca/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `pumpswap/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `raydium_amm_v4/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `raydium_clmm/parser.rs`

```rust
pub fn new() -> Self
```

---

### new

**Source**: `raydium_cpmm/parser.rs`

```rust
pub fn new() -> Self
```

---

### new_borrowed

**Source**: `zero_copy/events.rs`

```rust
pub fn new_borrowed(metadata: EventMetadata, raw_data: &'a [u8]) -> Self
```

Create a new zero-copy event with borrowed data

---

### new_fast

**Source**: `parsers/jupiter.rs`

```rust
pub fn new_fast() -> Self
```

Create parser with simplified analysis (faster)

---

### new_fast

**Source**: `parsers/metaplex.rs`

```rust
pub fn new_fast() -> Self
```

Create parser with minimal metadata parsing (faster)

---

### new_owned

**Source**: `zero_copy/events.rs`

```rust
pub fn new_owned(metadata: EventMetadata, raw_data: Vec<u8>) -> Self
```

Create a new zero-copy event with owned data

---

### new_standard

**Source**: `parsers/pump_fun.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

### new_standard

**Source**: `parsers/raydium_v4.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

### new_standard

**Source**: `parsers/jupiter.rs`

```rust
pub fn new_standard() -> Self
```

Create parser with zero-copy disabled

---

### not_enough_bytes

**Source**: `src/error.rs`

```rust
pub fn not_enough_bytes(expected: usize, found: usize, offset: usize) -> Self
```

Create a NotEnoughBytes error

---

### orca_whirlpool_program_id

**Source**: `orca/types.rs`

```rust
pub fn orca_whirlpool_program_id() -> Pubkey
```

Extract Orca Whirlpool program ID as Pubkey

---

### output_mint

**Source**: `zero_copy/events.rs`

```rust
pub fn output_mint(&mut self) -> Option<Pubkey>
```

Get output mint, parsing if necessary

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

### pool_address

**Source**: `zero_copy/events.rs`

```rust
pub fn pool_address(&mut self) -> Option<Pubkey>
```

Get pool address, parsing if necessary

---

### position

**Source**: `zero_copy/parsers.rs`

```rust
pub fn position(&self) -> usize
```

Get current position

---

### process_stream

**Source**: `pipelines/parsing.rs`

```rust
pub async fn process_stream<S>(&mut self, mut input_stream: S) -> Result<(), PipelineError> where S: Stream<Item = ParsingInput> + Unpin,
```

Process a stream of parsing inputs

---

### protocol_event

**Source**: `src/solana_events.rs`

```rust
pub fn protocol_event(params: ProtocolEventParams) -> Self
```

Create a generic protocol event

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

### read_bytes

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParseError>
```

Read a byte array of fixed size

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

### read_pubkey

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_pubkey(&mut self) -> Result<solana_sdk::pubkey::Pubkey, ParseError>
```

Read a Pubkey (32 bytes)

---

### read_u128_le

**Source**: `common/utils.rs`

```rust
pub fn read_u128_le(data: &[u8], offset: usize) -> ParseResult<u128>
```

Read u128 from little-endian bytes at offset

---

### read_u32_le

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u32_le(&mut self) -> Result<u32, ParseError>
```

Read a u32 value (little endian)

---

### read_u32_le

**Source**: `common/utils.rs`

```rust
pub fn read_u32_le(data: &[u8], offset: usize) -> ParseResult<u32>
```

Read u32 from little-endian bytes at offset

---

### read_u64_le

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u64_le(&mut self) -> Result<u64, ParseError>
```

Read a u64 value (little endian)

---

### read_u64_le

**Source**: `common/utils.rs`

```rust
pub fn read_u64_le(data: &[u8], offset: usize) -> ParseResult<u64>
```

Common utility functions for event parsing

Read u64 from little-endian bytes at offset

---

### read_u8

**Source**: `zero_copy/parsers.rs`

```rust
pub fn read_u8(&mut self) -> Result<u8, ParseError>
```

Read a u8 value

---

### read_u8_le

**Source**: `common/utils.rs`

```rust
pub fn read_u8_le(data: &[u8], offset: usize) -> ParseResult<u8>
```

Read u8 from bytes at offset

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

### set_id

**Source**: `src/types.rs`

```rust
pub fn set_id(&mut self, id: String)
```

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

### shares_to_amount

**Source**: `marginfi/types.rs`

```rust
pub fn shares_to_amount(shares: u128, share_value: u128) -> u64
```

Convert shares to amount using share value

---

### should_handle

**Source**: `events/factory.rs`

```rust
pub fn should_handle(&self, program_id: &solana_sdk::pubkey::Pubkey) -> bool
```

Check if a program ID is supported

---

### signature

**Source**: `zero_copy/events.rs`

```rust
pub fn signature(&self) -> &str
```

Get signature for any lifetime

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

### slot

**Source**: `zero_copy/events.rs`

```rust
pub fn slot(&self) -> u64
```

Get slot for any lifetime

---

### sqrt_price_to_price

**Source**: `orca/types.rs`

```rust
pub fn sqrt_price_to_price(sqrt_price: u128, decimals_a: u8, decimals_b: u8) -> f64
```

Convert sqrt price to price

---

### supported_program_ids

**Source**: `events/factory.rs`

```rust
pub fn supported_program_ids(&self) -> Vec<solana_sdk::pubkey::Pubkey>
```

Get all supported program IDs

---

### supports_program

**Source**: `src/solana_parser.rs`

```rust
pub fn supports_program(&self, program_id: &Pubkey) -> bool
```

Check if a program ID is supported

---

### swap

**Source**: `src/solana_events.rs`

```rust
pub fn swap(params: SwapEventParams) -> Self
```

Create a swap event

---

### system_time_to_millis

**Source**: `common/utils.rs`

```rust
pub fn system_time_to_millis(time: std::time::SystemTime) -> u64
```

Convert SystemTime to milliseconds since epoch

---

### take_output_receiver

**Source**: `pipelines/parsing.rs`

```rust
pub fn take_output_receiver(&mut self) -> mpsc::UnboundedReceiver<ParsingOutput>
```

Get the output receiver for processed events

---

### tick_index_to_price

**Source**: `orca/types.rs`

```rust
pub fn tick_index_to_price(tick_index: i32) -> f64
```

Convert tick index to price

---

### timestamp

**Source**: `zero_copy/events.rs`

```rust
pub fn timestamp(&self) -> std::time::SystemTime
```

Get timestamp for any lifetime

---

### to_core_metadata

**Source**: `src/types.rs`

```rust
pub fn to_core_metadata( &self, kind: riglr_events_core::types::EventKind, source: String, ) -> riglr_events_core::types::EventMetadata
```

Convert to riglr-events-core EventMetadata

---

### to_event_kind

**Source**: `src/types.rs`

```rust
pub fn to_event_kind(&self) -> riglr_events_core::types::EventKind
```

Convert local EventType to riglr-events-core EventKind

---

### to_owned

**Source**: `zero_copy/events.rs`

```rust
pub fn to_owned(&self) -> ZeroCopyEvent<'static>
```

Convert to owned event (clones all data)

---

### to_token_amount

**Source**: `common/utils.rs`

```rust
pub fn to_token_amount(amount: f64, decimals: u8) -> u64
```

Convert human-readable amount to token amount with decimals

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

### with_all_parsers

**Source**: `events/factory.rs`

```rust
pub fn with_all_parsers() -> Self
```

Create a registry with all available parsers

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

### with_data

**Source**: `src/solana_events.rs`

```rust
pub fn with_data<T: Serialize>(mut self, data: T) -> Result<Self, serde_json::Error>
```

Update the data payload

---

### with_legacy_parser

**Source**: `src/solana_parser.rs`

```rust
pub fn with_legacy_parser(legacy_parser: EventParserRegistry) -> Self
```

Create with specific legacy parser

---

### with_metadata

**Source**: `src/solana_events.rs`

```rust
pub fn with_metadata( legacy_metadata: EventMetadata, core_metadata: riglr_events_core::types::EventMetadata, data: serde_json::Value, ) -> Self
```

Create from both legacy and core metadata (for precise control)

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

### with_transfer_data

**Source**: `src/solana_events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

Set transfer data

---

### with_transfer_data

**Source**: `jupiter/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

---

### with_transfer_data

**Source**: `marginfi/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

---

### with_transfer_data

**Source**: `meteora/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

---

### with_transfer_data

**Source**: `orca/events.rs`

```rust
pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self
```

---

## Structs

### AccountMeta

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
```

```rust
pub struct AccountMeta { pub pubkey: Pubkey, pub is_signer: bool, pub is_writable: bool, }
```

Account metadata for swap

---

### BatchEventParser

**Source**: `zero_copy/parsers.rs`

```rust
pub struct BatchEventParser { /// Individual parsers for different protocols parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>, /// Pattern matcher for fast protocol detection pattern_matcher: SIMDPatternMatcher, /// Maximum batch size max_batch_size: usize, }
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
pub struct BatchParserStats { pub registered_parsers: usize, pub max_batch_size: usize, pub pattern_count: usize, }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct BonkPoolCreateEvent { #[serde(skip)]
```

Create pool event

---

### BonkTradeEvent

**Source**: `bonk/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct BonkTradeEvent { #[serde(skip)]
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
pub struct CacheStats { pub token_cache_size: usize, pub price_cache_size: usize, pub token_cache_hit_rate: f64, pub price_cache_hit_rate: f64, }
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
pub struct ConstantCurve { pub supply: u64, pub total_base_sell: u64, pub total_quote_fund_raising: u64, pub migrate_type: u8, }
```

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
pub struct CurveParams { pub constant_curve: Option<ConstantCurve>, pub fixed_curve: Option<FixedCurve>, pub linear_curve: Option<LinearCurve>, }
```

---

### CustomDeserializer

**Source**: `zero_copy/parsers.rs`

```rust
pub struct CustomDeserializer<'a> { /// Data to deserialize from data: &'a [u8], /// Current position pos: usize, }
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
pub struct DlmmBin { pub bin_id: u32, pub reserve_x: u64, pub reserve_y: u64, pub price: f64, pub liquidity_supply: u128, }
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
pub struct DlmmPairConfig { pub pair: Pubkey, pub token_mint_x: Pubkey, pub token_mint_y: Pubkey, pub bin_step: u16, pub base_fee_percentage: u64, pub max_fee_percentage: u64, pub protocol_fee_percentage: u64, pub liquidity_fee_percentage: u64, pub volatility_accumulator: u32, pub volatility_reference: u32, pub id_reference: u32, pub time_of_last_update: u64, pub active_id: u32, pub base_key: Pubkey, }
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
pub struct EventEnricher { /// Enrichment configuration config: EnrichmentConfig, /// Token metadata cache token_cache: Arc<RwLock<HashMap<Pubkey, CacheEntry<TokenMetadata>>>>, /// Price data cache price_cache: Arc<RwLock<HashMap<Pubkey, CacheEntry<PriceData>>>>, /// HTTP client for external API calls http_client: reqwest::Client, }
```

Event enricher with caching and external API integration

---

### EventMetadata

**Source**: `src/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, BorshDeserialize)]
```

```rust
pub struct EventMetadata { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub protocol_type: ProtocolType, pub event_type: EventType, pub program_id: Pubkey, pub index: String, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, }
```

---

### EventParameters

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventParameters { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventParameters { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventParameters { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParameters

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventParameters { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub index: String, }
```

Parameters for creating event metadata, reducing function parameter count

---

### EventParserRegistry

**Source**: `events/factory.rs`

```rust
pub struct EventParserRegistry { parsers: HashMap<Protocol, Arc<dyn EventParser>>, program_id_to_parser: HashMap<solana_sdk::pubkey::Pubkey, Arc<dyn EventParser>>, }
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
pub struct FixedCurve { pub supply: u64, pub total_quote_fund_raising: u64, pub migrate_type: u8, }
```

---

### GenericEventParseConfig

**Source**: `core/traits.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GenericEventParseConfig { pub program_id: solana_sdk::pubkey::Pubkey, pub protocol_type: ProtocolType, pub inner_instruction_discriminator: &'static str, pub instruction_discriminator: &'static [u8], pub event_type: EventType, pub inner_instruction_parser: InnerInstructionEventParser, pub instruction_parser: InstructionEventParser, }
```

Generic event parser configuration

---

### GenericEventParser

**Source**: `core/traits.rs`

```rust
pub struct GenericEventParser { pub program_ids: Vec<solana_sdk::pubkey::Pubkey>, pub inner_instruction_configs: std::collections::HashMap<&'static str, Vec<GenericEventParseConfig>>, pub instruction_configs: std::collections::HashMap<Vec<u8>, Vec<GenericEventParseConfig>>, }
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
pub struct InnerInstructionParseParams<'a> { pub inner_instruction: &'a solana_transaction_status::UiCompiledInstruction, pub signature: &'a str, pub slot: u64, pub block_time: Option<i64>, pub program_received_time_ms: i64, pub index: String, }
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
pub struct InstructionParseParams<'a> { pub instruction: &'a solana_sdk::instruction::CompiledInstruction, pub accounts: &'a [solana_sdk::pubkey::Pubkey], pub signature: &'a str, pub slot: u64, pub block_time: Option<i64>, pub program_received_time_ms: i64, pub index: String, }
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
pub struct JupiterAccountLayout { pub user_transfer_authority: Pubkey, pub user_source_token_account: Pubkey, pub user_destination_token_account: Pubkey, pub destination_token_account: Pubkey, pub source_mint: Pubkey, pub destination_mint: Pubkey, pub platform_fee_account: Option<Pubkey>, }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct JupiterLiquidityEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub user: solana_sdk::pubkey::Pubkey, pub mint_a: solana_sdk::pubkey::Pubkey, pub mint_b: solana_sdk::pubkey::Pubkey, pub amount_a: u64, pub amount_b: u64, pub liquidity_amount: u64, pub is_remove: bool, pub transfer_data: Vec<TransferData>, }
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
pub struct JupiterRouteData { pub instruction: JupiterRouteInstruction, pub analysis: JupiterRouteAnalysis, }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct JupiterSwapBorshEvent { #[serde(skip)]
```

Jupiter swap event with borsh (for simple events)

---

### JupiterSwapData

**Source**: `jupiter/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct JupiterSwapData { pub user: Pubkey, pub input_mint: Pubkey, pub output_mint: Pubkey, pub input_amount: u64, pub output_amount: u64, pub price_impact_pct: Option<String>, pub platform_fee_bps: Option<u32>, pub route_plan: Vec<RoutePlan>, }
```

Jupiter swap event data (for UnifiedEvent)

---

### JupiterSwapEvent

**Source**: `jupiter/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct JupiterSwapEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub swap_data: JupiterSwapData, pub transfer_data: Vec<TransferData>, }
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
pub struct LinearCurve { pub supply: u64, pub total_quote_fund_raising: u64, pub migrate_type: u8, }
```

---

### LiquidityEventParams

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct LiquidityEventParams { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub protocol_type: ProtocolType, pub program_id: solana_sdk::pubkey::Pubkey, pub is_add: bool, pub mint_a: solana_sdk::pubkey::Pubkey, pub mint_b: solana_sdk::pubkey::Pubkey, pub amount_a: u64, pub amount_b: u64, }
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
pub struct MarginFiAccount { pub group: Pubkey, pub authority: Pubkey, pub lending_account: MarginFiLendingAccount, pub account_flags: u64, pub padding: [u128; 8], }
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
pub struct MarginFiBalance { pub active: bool, pub bank_pk: Pubkey, pub asset_shares: u128, pub liability_shares: u128, pub emissions_outstanding: u64, pub last_update: u64, pub padding: [u64; 1], }
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
pub struct MarginFiBankConfig { pub bank: Pubkey, pub mint: Pubkey, pub vault: Pubkey, pub oracle: Pubkey, pub bank_authority: Pubkey, pub collected_insurance_fees_outstanding: u64, pub fee_rate: u64, pub insurance_fee_rate: u64, pub insurance_vault: Pubkey, pub deposit_limit: u64, pub borrow_limit: u64, pub operational_state: u8, pub oracle_setup: u8, pub oracle_keys: [Pubkey; 5], }
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
pub struct MarginFiBankState { pub total_asset_shares: u128, pub total_liability_shares: u128, pub last_update: u64, pub lending_rate: u64, pub borrowing_rate: u64, pub asset_share_value: u128, pub liability_share_value: u128, pub liquidity_vault_authority: Pubkey, pub liquidity_vault_authority_bump: u8, pub insurance_vault_authority: Pubkey, pub insurance_vault_authority_bump: u8, pub collected_group_fees_outstanding: u64, pub fee_vault_authority: Pubkey, pub fee_vault_authority_bump: u8, pub fee_vault: Pubkey, }
```

MarginFi bank state

---

### MarginFiBorrowData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiBorrowData { pub marginfi_group: Pubkey, pub marginfi_account: Pubkey, pub signer: Pubkey, pub bank: Pubkey, pub token_account: Pubkey, pub bank_liquidity_vault: Pubkey, pub bank_liquidity_vault_authority: Pubkey, pub token_program: Pubkey, pub amount: u64, }
```

MarginFi borrow data

---

### MarginFiBorrowEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiBorrowEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub borrow_data: MarginFiBorrowData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

MarginFi borrow event

---

### MarginFiDepositData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiDepositData { pub marginfi_group: Pubkey, pub marginfi_account: Pubkey, pub signer: Pubkey, pub bank: Pubkey, pub token_account: Pubkey, pub bank_liquidity_vault: Pubkey, pub token_program: Pubkey, pub amount: u64, }
```

MarginFi deposit data

---

### MarginFiDepositEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiDepositEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub deposit_data: MarginFiDepositData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
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
pub struct MarginFiLendingAccount { pub balances: [MarginFiBalance; 16], pub padding: [u64; 8], }
```

MarginFi lending account

---

### MarginFiLiquidationData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiLiquidationData { pub marginfi_group: Pubkey, pub asset_bank: Pubkey, pub liab_bank: Pubkey, pub liquidatee_marginfi_account: Pubkey, pub liquidator_marginfi_account: Pubkey, pub liquidator: Pubkey, pub asset_bank_liquidity_vault: Pubkey, pub liab_bank_liquidity_vault: Pubkey, pub liquidator_token_account: Pubkey, pub token_program: Pubkey, pub asset_amount: u64, pub liab_amount: u64, }
```

MarginFi liquidation data

---

### MarginFiLiquidationEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiLiquidationEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub liquidation_data: MarginFiLiquidationData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

MarginFi liquidation event

---

### MarginFiRepayData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiRepayData { pub marginfi_group: Pubkey, pub marginfi_account: Pubkey, pub signer: Pubkey, pub bank: Pubkey, pub token_account: Pubkey, pub bank_liquidity_vault: Pubkey, pub token_program: Pubkey, pub amount: u64, pub repay_all: bool, }
```

MarginFi repay data

---

### MarginFiRepayEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiRepayEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub repay_data: MarginFiRepayData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

MarginFi repay event

---

### MarginFiWithdrawData

**Source**: `marginfi/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiWithdrawData { pub marginfi_group: Pubkey, pub marginfi_account: Pubkey, pub signer: Pubkey, pub bank: Pubkey, pub token_account: Pubkey, pub bank_liquidity_vault: Pubkey, pub bank_liquidity_vault_authority: Pubkey, pub token_program: Pubkey, pub amount: u64, pub withdraw_all: bool, }
```

MarginFi withdraw data

---

### MarginFiWithdrawEvent

**Source**: `marginfi/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MarginFiWithdrawEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub withdraw_data: MarginFiWithdrawData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

MarginFi withdraw event

---

### MemoryMappedParser

**Source**: `zero_copy/parsers.rs`

```rust
pub struct MemoryMappedParser { /// Memory-mapped file mmap: memmap2::Mmap, /// Protocol-specific parsers parsers: HashMap<ProtocolType, Arc<dyn ByteSliceEventParser>>, }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraDynamicLiquidityData { pub pool: Pubkey, pub user: Pubkey, pub token_mint_a: Pubkey, pub token_mint_b: Pubkey, pub vault_a: Pubkey, pub vault_b: Pubkey, pub lp_mint: Pubkey, pub pool_token_amount: u64, pub token_a_amount: u64, pub token_b_amount: u64, pub minimum_pool_token_amount: u64, pub maximum_token_a_amount: u64, pub maximum_token_b_amount: u64, pub is_deposit: bool, }
```

Meteora Dynamic liquidity data

---

### MeteoraDynamicLiquidityEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraDynamicLiquidityEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub liquidity_data: MeteoraDynamicLiquidityData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
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
pub struct MeteoraDynamicPoolData { pub pool: Pubkey, pub token_mint_a: Pubkey, pub token_mint_b: Pubkey, pub vault_a: Pubkey, pub vault_b: Pubkey, pub lp_mint: Pubkey, pub fee_rate: u64, pub admin_fee_rate: u64, pub trade_fee_numerator: u64, pub trade_fee_denominator: u64, pub owner_trade_fee_numerator: u64, pub owner_trade_fee_denominator: u64, pub owner_withdraw_fee_numerator: u64, pub owner_withdraw_fee_denominator: u64, pub host_fee_numerator: u64, pub host_fee_denominator: u64, }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraLiquidityData { pub pair: Pubkey, pub user: Pubkey, pub position: Pubkey, pub token_mint_x: Pubkey, pub token_mint_y: Pubkey, pub reserve_x: Pubkey, pub reserve_y: Pubkey, pub bin_id_from: u32, pub bin_id_to: u32, pub amount_x: u64, pub amount_y: u64, pub liquidity_minted: u128, pub active_id: u32, pub is_add: bool, pub bins_affected: Vec<DlmmBin>, }
```

Meteora DLMM liquidity data

---

### MeteoraLiquidityEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraLiquidityEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub liquidity_data: MeteoraLiquidityData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

Meteora DLMM liquidity event

---

### MeteoraSwapData

**Source**: `meteora/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraSwapData { pub pair: Pubkey, pub user: Pubkey, pub token_mint_x: Pubkey, pub token_mint_y: Pubkey, pub reserve_x: Pubkey, pub reserve_y: Pubkey, pub amount_in: u64, pub min_amount_out: u64, pub actual_amount_out: u64, pub swap_for_y: bool, pub active_id_before: u32, pub active_id_after: u32, pub fee_amount: u64, pub protocol_fee: u64, pub bins_traversed: Vec<u32>, }
```

Meteora DLMM swap data

---

### MeteoraSwapEvent

**Source**: `meteora/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct MeteoraSwapEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub swap_data: MeteoraSwapData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
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
pub struct MintParams { pub decimals: u8, pub name: String, pub symbol: String, pub uri: String, }
```

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
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaLiquidityData { pub whirlpool: Pubkey, pub position: Pubkey, pub position_authority: Pubkey, pub token_mint_a: Pubkey, pub token_mint_b: Pubkey, pub token_vault_a: Pubkey, pub token_vault_b: Pubkey, pub tick_lower_index: i32, pub tick_upper_index: i32, pub liquidity_amount: u128, pub token_max_a: u64, pub token_max_b: u64, pub token_actual_a: u64, pub token_actual_b: u64, pub is_increase: bool, }
```

Orca liquidity change data

---

### OrcaLiquidityEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaLiquidityEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub liquidity_data: OrcaLiquidityData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

Orca liquidity event (increase/decrease)

---

### OrcaPositionData

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaPositionData { pub whirlpool: Pubkey, pub position_mint: Pubkey, pub position: Pubkey, pub position_token_account: Pubkey, pub position_authority: Pubkey, pub tick_lower_index: i32, pub tick_upper_index: i32, pub liquidity: u128, pub fee_growth_checkpoint_a: u128, pub fee_growth_checkpoint_b: u128, pub fee_owed_a: u64, pub fee_owed_b: u64, pub reward_infos: [PositionRewardInfo; 3], }
```

Orca position data

---

### OrcaPositionEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaPositionEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub position_data: OrcaPositionData, pub is_open: bool, pub transfer_data: Vec<TransferData>, #[serde(skip)]
```

Orca position event (open/close)

---

### OrcaSwapData

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaSwapData { pub whirlpool: Pubkey, pub user: Pubkey, pub token_mint_a: Pubkey, pub token_mint_b: Pubkey, pub token_vault_a: Pubkey, pub token_vault_b: Pubkey, pub amount: u64, pub amount_specified_is_input: bool, pub a_to_b: bool, pub sqrt_price_limit: u128, pub amount_in: u64, pub amount_out: u64, pub fee_amount: u64, pub tick_current_index: i32, pub sqrt_price: u128, pub liquidity: u128, }
```

Orca swap event data

---

### OrcaSwapEvent

**Source**: `orca/events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct OrcaSwapEvent { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub block_time_ms: i64, pub program_received_time_ms: i64, pub program_handle_time_consuming_ms: i64, pub index: String, pub swap_data: OrcaSwapData, pub transfer_data: Vec<TransferData>, #[serde(skip)]
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
pub struct PipelineStats { pub max_batch_size: usize, pub max_concurrent_tasks: usize, pub registered_parsers: usize, pub available_permits: usize, }
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
pub struct PositionRewardInfo { pub growth_inside_checkpoint: u128, pub amount_owed: u64, }
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
pub struct ProtocolEventParams { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub protocol_type: ProtocolType, pub event_type: EventType, pub program_id: solana_sdk::pubkey::Pubkey, pub data: serde_json::Value, }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct PumpSwapBuyEvent { #[serde(skip)]
```

Buy event

---

### PumpSwapCreatePoolEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct PumpSwapCreatePoolEvent { #[serde(skip)]
```

Create pool event

---

### PumpSwapDepositEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct PumpSwapDepositEvent { #[serde(skip)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct PumpSwapSellEvent { #[serde(skip)]
```

Sell event

---

### PumpSwapWithdrawEvent

**Source**: `pumpswap/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct PumpSwapWithdrawEvent { #[serde(skip)]
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
pub struct RaydiumAmmV4DepositEvent { pub metadata: EventMetadata, pub max_coin_amount: u64, pub max_pc_amount: u64, pub base_side: u64, // Account keys pub token_program: Pubkey, pub amm: Pubkey, pub amm_authority: Pubkey, pub amm_open_orders: Pubkey, pub amm_target_orders: Pubkey, pub lp_mint_address: Pubkey, pub pool_coin_token_account: Pubkey, pub pool_pc_token_account: Pubkey, pub serum_market: Pubkey, pub user_coin_token_account: Pubkey, pub user_pc_token_account: Pubkey, pub user_lp_token_account: Pubkey, pub user_owner: Pubkey, }
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
pub struct RaydiumAmmV4Initialize2Event { pub metadata: EventMetadata, pub nonce: u8, pub open_time: u64, pub init_pc_amount: u64, pub init_coin_amount: u64, // Account keys pub amm: Pubkey, pub amm_authority: Pubkey, pub amm_open_orders: Pubkey, pub lp_mint_address: Pubkey, pub coin_mint_address: Pubkey, pub pc_mint_address: Pubkey, pub pool_coin_token_account: Pubkey, pub pool_pc_token_account: Pubkey, pub pool_withdraw_queue: Pubkey, pub amm_target_orders: Pubkey, pub pool_lp_token_account: Pubkey, pub pool_temp_lp_token_account: Pubkey, pub serum_program: Pubkey, pub serum_market: Pubkey, pub user_wallet: Pubkey, }
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
pub struct RaydiumAmmV4SwapEvent { pub metadata: EventMetadata, pub amount_in: u64, pub amount_out: u64, pub direction: SwapDirection, // Account keys pub amm: Pubkey, pub amm_authority: Pubkey, pub amm_open_orders: Pubkey, pub pool_coin_token_account: Pubkey, pub pool_pc_token_account: Pubkey, pub serum_program: Pubkey, pub serum_market: Pubkey, pub user_coin_token_account: Pubkey, pub user_pc_token_account: Pubkey, pub user_owner: Pubkey, }
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
pub struct RaydiumAmmV4WithdrawEvent { pub metadata: EventMetadata, pub amount: u64, // Account keys pub token_program: Pubkey, pub amm: Pubkey, pub amm_authority: Pubkey, pub amm_open_orders: Pubkey, pub amm_target_orders: Pubkey, pub lp_mint_address: Pubkey, pub pool_coin_token_account: Pubkey, pub pool_pc_token_account: Pubkey, pub pool_withdraw_queue: Pubkey, pub pool_temp_lp_token_account: Pubkey, pub serum_program: Pubkey, pub serum_market: Pubkey, pub serum_coin_vault_account: Pubkey, pub serum_pc_vault_account: Pubkey, pub serum_vault_signer: Pubkey, pub user_lp_token_account: Pubkey, pub user_coin_token_account: Pubkey, pub user_pc_token_account: Pubkey, pub user_owner: Pubkey, pub serum_event_queue: Pubkey, pub serum_bids: Pubkey, pub serum_asks: Pubkey, }
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
pub struct RaydiumAmmV4WithdrawPnlEvent { pub metadata: EventMetadata, // Account keys pub token_program: Pubkey, pub amm: Pubkey, pub amm_config: Pubkey, pub amm_authority: Pubkey, pub amm_open_orders: Pubkey, pub pool_coin_token_account: Pubkey, pub pool_pc_token_account: Pubkey, pub coin_pnl_token_account: Pubkey, pub pc_pnl_token_account: Pubkey, pub pnl_owner_account: Pubkey, pub amm_target_orders: Pubkey, pub serum_program: Pubkey, pub serum_market: Pubkey, pub serum_event_queue: Pubkey, pub serum_coin_vault_account: Pubkey, pub serum_pc_vault_account: Pubkey, pub serum_vault_signer: Pubkey, }
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
pub struct RaydiumClmmClosePositionEvent { pub metadata: EventMetadata, // Account keys pub nft_owner: Pubkey, pub position_nft_mint: Pubkey, pub position_nft_account: Pubkey, pub personal_position: Pubkey, }
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
pub struct RaydiumClmmCreatePoolEvent { pub metadata: EventMetadata, pub sqrt_price_x64: u128, pub tick_current: i32, pub observation_index: u16, // Account keys pub pool_creator: Pubkey, pub pool_state: Pubkey, pub token_mint0: Pubkey, pub token_mint1: Pubkey, pub token_vault0: Pubkey, pub token_vault1: Pubkey, }
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
pub struct RaydiumClmmDecreaseLiquidityV2Event { pub metadata: EventMetadata, pub liquidity: u128, pub amount0_min: u64, pub amount1_min: u64, // Account keys pub nft_owner: Pubkey, pub position_nft_account: Pubkey, pub pool_state: Pubkey, }
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
pub struct RaydiumClmmIncreaseLiquidityV2Event { pub metadata: EventMetadata, pub liquidity: u128, pub amount0_max: u64, pub amount1_max: u64, pub base_flag: Option<bool>, // Account keys pub nft_owner: Pubkey, pub position_nft_account: Pubkey, pub pool_state: Pubkey, }
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
pub struct RaydiumClmmOpenPositionV2Event { pub metadata: EventMetadata, pub tick_lower_index: i32, pub tick_upper_index: i32, pub tick_array_lower_start_index: i32, pub tick_array_upper_start_index: i32, pub liquidity: u128, pub amount0_max: u64, pub amount1_max: u64, pub with_metadata: bool, pub base_flag: Option<bool>, // Account keys pub payer: Pubkey, pub position_nft_owner: Pubkey, pub position_nft_mint: Pubkey, pub position_nft_account: Pubkey, pub metadata_account: Pubkey, pub pool_state: Pubkey, }
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
pub struct RaydiumClmmOpenPositionWithToken22NftEvent { pub metadata: EventMetadata, pub tick_lower_index: i32, pub tick_upper_index: i32, pub tick_array_lower_start_index: i32, pub tick_array_upper_start_index: i32, pub liquidity: u128, pub amount0_max: u64, pub amount1_max: u64, pub with_metadata: bool, pub base_flag: Option<bool>, // Account keys pub payer: Pubkey, pub position_nft_owner: Pubkey, pub position_nft_mint: Pubkey, pub position_nft_account: Pubkey, pub pool_state: Pubkey, }
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
pub struct RaydiumClmmSwapEvent { pub metadata: EventMetadata, pub amount0: u64, pub amount1: u64, pub sqrt_price_x64: u128, pub liquidity: u128, pub tick_current: i32, // Account keys pub payer: Pubkey, pub pool_state: Pubkey, pub input_token_account: Pubkey, pub output_token_account: Pubkey, pub input_vault: Pubkey, pub output_vault: Pubkey, pub token_mint0: Pubkey, pub token_mint1: Pubkey, }
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
pub struct RaydiumClmmSwapV2Event { pub metadata: EventMetadata, pub amount0: u64, pub amount1: u64, pub sqrt_price_x64: u128, pub liquidity: u128, pub tick_current: i32, pub is_base_input: bool, // Account keys pub payer: Pubkey, pub pool_state: Pubkey, pub input_token_account: Pubkey, pub output_token_account: Pubkey, pub input_vault: Pubkey, pub output_vault: Pubkey, pub token_mint0: Pubkey, pub token_mint1: Pubkey, }
```

Raydium CLMM swap V2 event

---

### RaydiumCpmmDepositEvent

**Source**: `raydium_cpmm/events.rs`

**Attributes**:
```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct RaydiumCpmmDepositEvent { #[serde(skip)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
```

```rust
pub struct RaydiumCpmmSwapEvent { #[serde(skip)]
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
pub struct RoutePlan { pub input_mint: Pubkey, pub output_mint: Pubkey, pub amount_in: u64, pub amount_out: u64, pub dex_label: String, }
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
pub struct RoutePlanStep { pub swap: SwapInfo, pub percent: u8, }
```

Route plan step for Jupiter swaps

---

### RpcConnectionPool

**Source**: `zero_copy/parsers.rs`

```rust
pub struct RpcConnectionPool { /// Pool of RPC clients clients: Vec<Arc<solana_client::rpc_client::RpcClient>>, /// Current index for round-robin current: std::sync::atomic::AtomicUsize, }
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
pub struct RuleResult { pub errors: Vec<ValidationError>, pub warnings: Vec<ValidationWarning>, pub consistency_checks: usize, }
```

Result from a validation rule

---

### SIMDPatternMatcher

**Source**: `zero_copy/parsers.rs`

```rust
pub struct SIMDPatternMatcher { /// Patterns to match (discriminator bytes)
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
pub struct SharedAccountsExactOutRouteData { pub route_plan: Vec<RoutePlanStep>, pub out_amount: u64, pub quoted_in_amount: u64, pub slippage_bps: u16, pub platform_fee_bps: u8, }
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
pub struct SharedAccountsRouteData { pub route_plan: Vec<RoutePlanStep>, pub in_amount: u64, pub quoted_out_amount: u64, pub slippage_bps: u16, pub platform_fee_bps: u8, }
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
pub struct SolanaEvent { /// Legacy metadata for compatibility pub legacy_metadata: EventMetadata, /// Core metadata for new functionality pub core_metadata: riglr_events_core::types::EventMetadata, /// Event data payload pub data: serde_json::Value, /// Transfer data for token movements pub transfer_data: Vec<TransferData>, }
```

A wrapper that implements both UnifiedEvent and Event traits for seamless migration

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
pub struct SwapData { pub input_mint: Pubkey, pub output_mint: Pubkey, pub amount_in: u64, pub amount_out: u64, }
```

---

### SwapEventParams

**Source**: `src/solana_events.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SwapEventParams { pub id: String, pub signature: String, pub slot: u64, pub block_time: i64, pub protocol_type: ProtocolType, pub program_id: solana_sdk::pubkey::Pubkey, pub input_mint: solana_sdk::pubkey::Pubkey, pub output_mint: solana_sdk::pubkey::Pubkey, pub amount_in: u64, pub amount_out: u64, }
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
pub struct SwapInfo { pub source_token: Pubkey, pub destination_token: Pubkey, pub source_token_account: Pubkey, pub destination_token_account: Pubkey, pub swap_program_id: Pubkey, pub swap_accounts: Vec<AccountMeta>, pub swap_data: Vec<u8>, }
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
pub struct TransferData { pub source: Pubkey, pub destination: Pubkey, pub mint: Option<Pubkey>, pub amount: u64, }
```

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
pub struct ValidationPipeline { /// Validation configuration config: ValidationConfig, /// Duplicate detection cache seen_events: Arc<tokio::sync::RwLock<HashMap<String, Instant>>>, /// Validation rule registry rules: Vec<Arc<dyn ValidationRule>>, }
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
pub struct ValidationStats { pub total_rules: usize, pub cached_events: usize, pub strict_mode: bool, }
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
pub struct VestingParams { pub total_locked_amount: u64, pub cliff_period: u64, pub unlock_period: u64, }
```

---

### WhirlpoolAccount

**Source**: `orca/types.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct WhirlpoolAccount { pub whirlpools_config: Pubkey, pub whirlpool_bump: [u8; 1], pub tick_spacing: u16, pub tick_spacing_seed: [u8; 2], pub fee_rate: u16, pub protocol_fee_rate: u16, pub liquidity: u128, pub sqrt_price: u128, pub tick_current_index: i32, pub protocol_fee_owed_a: u64, pub protocol_fee_owed_b: u64, pub token_mint_a: Pubkey, pub token_vault_a: Pubkey, pub fee_growth_global_a: u128, pub token_mint_b: Pubkey, pub token_vault_b: Pubkey, pub fee_growth_global_b: u128, pub reward_last_updated_timestamp: u64, pub reward_infos: [WhirlpoolRewardInfo; 3], }
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
pub struct WhirlpoolRewardInfo { pub mint: Pubkey, pub vault: Pubkey, pub authority: Pubkey, pub emissions_per_second_x64: u128, pub growth_global_x64: u128, }
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


---

*This documentation was automatically generated from the source code.*