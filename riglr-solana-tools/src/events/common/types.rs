use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiInstruction;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::Mutex;

// Object pool size configuration
const EVENT_METADATA_POOL_SIZE: usize = 1000;
const TRANSFER_DATA_POOL_SIZE: usize = 2000;

/// Event metadata object pool
pub struct EventMetadataPool {
    pool: Arc<Mutex<Vec<EventMetadata>>>,
}

impl Default for EventMetadataPool {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMetadataPool {
    pub fn new() -> Self {
        Self { pool: Arc::new(Mutex::new(Vec::with_capacity(EVENT_METADATA_POOL_SIZE))) }
    }

    pub async fn acquire(&self) -> Option<EventMetadata> {
        let mut pool = self.pool.lock().await;
        pool.pop()
    }

    pub async fn release(&self, metadata: EventMetadata) {
        let mut pool = self.pool.lock().await;
        if pool.len() < EVENT_METADATA_POOL_SIZE {
            pool.push(metadata);
        }
    }
}

/// Transfer data object pool
pub struct TransferDataPool {
    pool: Arc<Mutex<Vec<TransferData>>>,
}

impl Default for TransferDataPool {
    fn default() -> Self {
        Self::new()
    }
}

impl TransferDataPool {
    pub fn new() -> Self {
        Self { pool: Arc::new(Mutex::new(Vec::with_capacity(TRANSFER_DATA_POOL_SIZE))) }
    }

    pub async fn acquire(&self) -> Option<TransferData> {
        let mut pool = self.pool.lock().await;
        pool.pop()
    }

    pub async fn release(&self, transfer_data: TransferData) {
        let mut pool = self.pool.lock().await;
        if pool.len() < TRANSFER_DATA_POOL_SIZE {
            pool.push(transfer_data);
        }
    }
}

// Global object pool instances
lazy_static::lazy_static! {
    pub static ref EVENT_METADATA_POOL: EventMetadataPool = EventMetadataPool::new();
    pub static ref TRANSFER_DATA_POOL: TransferDataPool = TransferDataPool::new();
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum ProtocolType {
    #[default]
    PumpSwap,
    PumpFun,
    Bonk,
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    Common,
}

/// Event type enumeration
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum EventType {
    // PumpSwap events
    #[default]
    PumpSwapBuy,
    PumpSwapSell,
    PumpSwapCreatePool,
    PumpSwapDeposit,
    PumpSwapWithdraw,

    // PumpFun events
    PumpFunCreateToken,
    PumpFunBuy,
    PumpFunSell,
    PumpFunMigrate,

    // Bonk events
    BonkBuyExactIn,
    BonkBuyExactOut,
    BonkSellExactIn,
    BonkSellExactOut,
    BonkInitialize,
    BonkMigrateToAmm,
    BonkMigrateToCpswap,

    // Raydium CPMM events
    RaydiumCpmmSwapBaseInput,
    RaydiumCpmmSwapBaseOutput,
    RaydiumCpmmDeposit,
    RaydiumCpmmInitialize,
    RaydiumCpmmWithdraw,

    // Raydium CLMM events
    RaydiumClmmSwap,
    RaydiumClmmSwapV2,
    RaydiumClmmClosePosition,
    RaydiumClmmIncreaseLiquidityV2,
    RaydiumClmmDecreaseLiquidityV2,
    RaydiumClmmCreatePool,
    RaydiumClmmOpenPositionWithToken22Nft,
    RaydiumClmmOpenPositionV2,

    // Raydium AMM V4 events
    RaydiumAmmV4SwapBaseIn,
    RaydiumAmmV4SwapBaseOut,
    RaydiumAmmV4Deposit,
    RaydiumAmmV4Initialize2,
    RaydiumAmmV4Withdraw,
    RaydiumAmmV4WithdrawPnl,

    // Common events
    BlockMeta,
    Unknown,
}

impl EventType {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            EventType::PumpSwapBuy => "PumpSwapBuy".to_string(),
            EventType::PumpSwapSell => "PumpSwapSell".to_string(),
            EventType::PumpSwapCreatePool => "PumpSwapCreatePool".to_string(),
            EventType::PumpSwapDeposit => "PumpSwapDeposit".to_string(),
            EventType::PumpSwapWithdraw => "PumpSwapWithdraw".to_string(),
            EventType::PumpFunCreateToken => "PumpFunCreateToken".to_string(),
            EventType::PumpFunBuy => "PumpFunBuy".to_string(),
            EventType::PumpFunSell => "PumpFunSell".to_string(),
            EventType::PumpFunMigrate => "PumpFunMigrate".to_string(),
            EventType::BonkBuyExactIn => "BonkBuyExactIn".to_string(),
            EventType::BonkBuyExactOut => "BonkBuyExactOut".to_string(),
            EventType::BonkSellExactIn => "BonkSellExactIn".to_string(),
            EventType::BonkSellExactOut => "BonkSellExactOut".to_string(),
            EventType::BonkInitialize => "BonkInitialize".to_string(),
            EventType::BonkMigrateToAmm => "BonkMigrateToAmm".to_string(),
            EventType::BonkMigrateToCpswap => "BonkMigrateToCpswap".to_string(),
            EventType::RaydiumCpmmSwapBaseInput => "RaydiumCpmmSwapBaseInput".to_string(),
            EventType::RaydiumCpmmSwapBaseOutput => "RaydiumCpmmSwapBaseOutput".to_string(),
            EventType::RaydiumCpmmDeposit => "RaydiumCpmmDeposit".to_string(),
            EventType::RaydiumCpmmInitialize => "RaydiumCpmmInitialize".to_string(),
            EventType::RaydiumCpmmWithdraw => "RaydiumCpmmWithdraw".to_string(),
            EventType::RaydiumClmmSwap => "RaydiumClmmSwap".to_string(),
            EventType::RaydiumClmmSwapV2 => "RaydiumClmmSwapV2".to_string(),
            EventType::RaydiumClmmClosePosition => "RaydiumClmmClosePosition".to_string(),
            EventType::RaydiumClmmDecreaseLiquidityV2 => {
                "RaydiumClmmDecreaseLiquidityV2".to_string()
            }
            EventType::RaydiumClmmCreatePool => "RaydiumClmmCreatePool".to_string(),
            EventType::RaydiumClmmIncreaseLiquidityV2 => {
                "RaydiumClmmIncreaseLiquidityV2".to_string()
            }
            EventType::RaydiumClmmOpenPositionWithToken22Nft => {
                "RaydiumClmmOpenPositionWithToken22Nft".to_string()
            }
            EventType::RaydiumClmmOpenPositionV2 => "RaydiumClmmOpenPositionV2".to_string(),
            EventType::RaydiumAmmV4SwapBaseIn => "RaydiumAmmV4SwapBaseIn".to_string(),
            EventType::RaydiumAmmV4SwapBaseOut => "RaydiumAmmV4SwapBaseOut".to_string(),
            EventType::RaydiumAmmV4Deposit => "RaydiumAmmV4Deposit".to_string(),
            EventType::RaydiumAmmV4Initialize2 => "RaydiumAmmV4Initialize2".to_string(),
            EventType::RaydiumAmmV4Withdraw => "RaydiumAmmV4Withdraw".to_string(),
            EventType::RaydiumAmmV4WithdrawPnl => "RaydiumAmmV4WithdrawPnl".to_string(),
            EventType::BlockMeta => "BlockMeta".to_string(),
            EventType::Unknown => "Unknown".to_string(),
        }
    }
}

/// Parse result
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ParseResult<T> {
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }

    pub fn failure(error: String) -> Self {
        Self { success: false, data: None, error: Some(error) }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_failure(&self) -> bool {
        !self.success
    }
}

/// Protocol information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolInfo {
    pub name: String,
    pub program_ids: Vec<Pubkey>,
}

impl ProtocolInfo {
    pub fn new(name: String, program_ids: Vec<Pubkey>) -> Self {
        Self { name, program_ids }
    }

    pub fn supports_program(&self, program_id: &Pubkey) -> bool {
        self.program_ids.contains(program_id)
    }
}

/// Transfer data
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct TransferData {
    pub token_program: Pubkey,
    pub source: Pubkey,
    pub destination: Pubkey,
    pub authority: Option<Pubkey>,
    pub amount: u64,
    pub decimals: Option<u8>,
    pub mint: Option<Pubkey>,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct SwapData {
    pub from_mint: Pubkey,
    pub to_mint: Pubkey,
    pub from_amount: u64,
    pub to_amount: u64,
    pub description: Option<String>,
}

/// Event metadata
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct EventMetadata {
    pub id: String,
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    pub block_time_ms: i64,
    pub program_received_time_ms: i64,
    pub program_handle_time_consuming_ms: i64,
    pub protocol: ProtocolType,
    pub event_type: EventType,
    pub program_id: Pubkey,
    pub transfer_datas: Vec<TransferData>,
    pub swap_data: Option<SwapData>,
    pub index: String,
}

impl EventMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        protocol: ProtocolType,
        event_type: EventType,
        program_id: Pubkey,
        index: String,
        program_received_time_ms: i64,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            program_received_time_ms,
            program_handle_time_consuming_ms: 0,
            protocol,
            event_type,
            program_id,
            transfer_datas: Vec::with_capacity(4), // Pre-allocate capacity
            swap_data: None,
            index,
        }
    }

    pub fn set_id(&mut self, id: String) {
        let _id = format!("{}-{}-{}", self.signature, self.event_type.to_string(), id);
        let mut hasher = DefaultHasher::new();
        _id.hash(&mut hasher);
        let hash_value = hasher.finish();
        self.id = format!("{:x}", hash_value);
    }

    pub fn set_transfer_datas(
        &mut self,
        transfer_datas: Vec<TransferData>,
        swap_data: Option<SwapData>,
    ) {
        self.transfer_datas = transfer_datas;
        self.swap_data = swap_data;
    }

    /// Recycle EventMetadata to object pool
    pub async fn recycle(self) {
        EVENT_METADATA_POOL.release(self).await;
    }
}

/// Parse token transfer data from next instructions
pub fn parse_transfer_datas_from_next_instructions(
    _event: Box<dyn crate::events::core::traits::UnifiedEvent>,
    inner_instruction: &solana_transaction_status::UiInnerInstructions,
    current_index: i8,
    accounts: &[Pubkey],
) -> (Vec<TransferData>, Option<SwapData>) {
    let mut transfer_datas = vec![];
    // Get the next two instructions after the current instruction
    let next_instructions: Vec<&UiInstruction> =
        inner_instruction.instructions.iter().skip((current_index + 1) as usize).collect();

    let system_programs = vec![
        // Token Program
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap(),
        // Token 2022 Program
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap(),
        // System Program
        Pubkey::from_str("11111111111111111111111111111111").unwrap(),
    ];
    for instruction in next_instructions {
        if let UiInstruction::Compiled(compiled) = instruction {
            if !system_programs.contains(&accounts[compiled.program_id_index as usize]) {
                break;
            }
            if let Ok(data) = bs58::decode(compiled.data.clone()).into_vec() {
                // Token Program: transferChecked
                // Token 2022 Program: transferChecked
                if data[0] == 12 {
                    let account_pubkeys: Vec<Pubkey> =
                        compiled.accounts.iter().map(|a| accounts[*a as usize]).collect();
                    if account_pubkeys.len() < 4 {
                        continue;
                    }
                    let (source, mint, destination, authority) = (
                        account_pubkeys[0],
                        account_pubkeys[1],
                        account_pubkeys[2],
                        account_pubkeys[3],
                    );
                    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                    let decimals = data[9];
                    let token_program = accounts[compiled.program_id_index as usize];
                    transfer_datas.push(TransferData {
                        amount,
                        decimals: Some(decimals),
                        mint: Some(mint),
                        source,
                        destination,
                        authority: Some(authority),
                        token_program,
                    });
                }
                // Token Program: transfer
                else if data[0] == 3 {
                    let account_pubkeys: Vec<Pubkey> =
                        compiled.accounts.iter().map(|a| accounts[*a as usize]).collect();
                    if account_pubkeys.len() < 3 {
                        continue;
                    }
                    let (source, destination, authority) =
                        (account_pubkeys[0], account_pubkeys[1], account_pubkeys[2]);
                    let amount = u64::from_le_bytes(data[1..9].try_into().unwrap());
                    let token_program = accounts[compiled.program_id_index as usize];
                    transfer_datas.push(TransferData {
                        amount,
                        decimals: None,
                        mint: None,
                        source,
                        destination,
                        authority: Some(authority),
                        token_program,
                    });
                }
                //System Program: transfer
                else if data[0] == 2 {
                    let account_pubkeys: Vec<Pubkey> =
                        compiled.accounts.iter().map(|a| accounts[*a as usize]).collect();
                    if account_pubkeys.len() < 2 {
                        continue;
                    }
                    let (source, destination) = (account_pubkeys[0], account_pubkeys[1]);
                    let amount = u64::from_le_bytes(data[4..12].try_into().unwrap());
                    let token_program = accounts[compiled.program_id_index as usize];
                    transfer_datas.push(TransferData {
                        amount,
                        decimals: None,
                        mint: None,
                        source,
                        destination,
                        authority: None,
                        token_program,
                    });
                }
            }
        }
    }
    // Infer swap by netting flows per mint, using most-frequent authority as the user.
    let mut swap_data: SwapData = SwapData {
        from_mint: Pubkey::default(),
        to_mint: Pubkey::default(),
        from_amount: 0,
        to_amount: 0,
        description: None,
    };

    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    if !transfer_datas.is_empty() {
        use std::collections::HashMap;

        // 1) Determine the most common authority (probable user wallet)
        let mut auth_counts: HashMap<Pubkey, usize> = HashMap::new();
        for td in &transfer_datas {
            if let Some(auth) = td.authority {
                *auth_counts.entry(auth).or_insert(0) += 1;
            }
        }
        let user_authority = auth_counts
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(k, _)| *k);

        // 2) Aggregate spent/received amounts per mint
        let mut spent_by_mint: HashMap<Pubkey, u128> = HashMap::new();
        let mut recv_by_mint: HashMap<Pubkey, u128> = HashMap::new();

        for td in &transfer_datas {
            // Resolve mint: system transfers are SOL; SPL transfers may miss mint for `transfer` (opcode 3)
            let mint = match (td.mint, td.token_program == system_program) {
                (Some(m), _) => Some(m),
                (None, true) => Some(sol_mint),
                _ => None, // Unknown mint, skip for swap inference
            };

            if let Some(mint) = mint {
                let amount = td.amount as u128;
                match (user_authority, td.authority) {
                    (Some(user), Some(auth)) if auth == user => {
                        // Outflow from user
                        *spent_by_mint.entry(mint).or_insert(0) += amount;
                    }
                    // If we don't know the user, we still try to infer inflows as transfers not authorized by the spender
                    (Some(user), Some(auth)) if auth != user => {
                        *recv_by_mint.entry(mint).or_insert(0) += amount;
                    }
                    (Some(_), None) => {
                        // System transfers (SOL) without authority: conservative, treat as outflow only if source equals an earlier source
                        // Without ownership context it's ambiguous; don't classify.
                    }
                    (None, _) => {
                        // No authority context at all; skip classification
                    }
                    (Some(_), Some(_)) => {
                        // Remaining cases should be covered by guards; keep as no-op for exhaustiveness
                    }
                }
            }
        }

        // 3) Choose dominant from/to mints by volume
        let from = spent_by_mint.iter().max_by_key(|(_, v)| *v).map(|(k, v)| (*k, *v));
        let mut to = recv_by_mint.iter().max_by_key(|(_, v)| *v).map(|(k, v)| (*k, *v));

        // Avoid same mint for from/to when possible
        if let (Some((from_m, _)), Some((to_m, _))) = (from, to) {
            if from_m == to_m {
                // Try the second-best receive mint
                let mut entries: Vec<(&Pubkey, &u128)> = recv_by_mint.iter().collect();
                entries.sort_by(|a, b| b.1.cmp(a.1));
                if let Some((m, v)) = entries.into_iter().find(|(m, _)| **m != from_m) {
                    to = Some((*m, *v));
                }
            }
        }

        if let (Some((from_mint, from_amt)), Some((to_mint, to_amt))) = (from, to) {
            if from_mint != to_mint && from_amt > 0 && to_amt > 0 {
                swap_data.from_mint = from_mint;
                swap_data.to_mint = to_mint;
                swap_data.from_amount = (from_amt as u64).saturating_sub(0);
                swap_data.to_amount = (to_amt as u64).saturating_sub(0);
                if let Some(user) = user_authority {
                    swap_data.description = Some(format!(
                        "inferred swap by user {} across {} transfers",
                        user, transfer_datas.len()
                    ));
                } else {
                    swap_data.description = Some(format!(
                        "inferred swap across {} transfers",
                        transfer_datas.len()
                    ));
                }
            }
        } else if user_authority.is_some() {
            // Fallback: if exactly two distinct mints appear (including SOL), use their summed directions
            let mut by_mint: HashMap<Pubkey, (u128, u128)> = HashMap::new(); // (spent, recv)
            for (m, v) in spent_by_mint.iter() {
                by_mint.entry(*m).or_insert((0, 0)).0 += *v;
            }
            for (m, v) in recv_by_mint.iter() {
                by_mint.entry(*m).or_insert((0, 0)).1 += *v;
            }
            if by_mint.len() == 2 {
                let mut it = by_mint.into_iter();
                let (m1, (s1, r1)) = it.next().unwrap();
                let (m2, (s2, r2)) = it.next().unwrap();
                // pick direction by net
                let net1 = r1 as i128 - s1 as i128;
                let net2 = r2 as i128 - s2 as i128;
                if net1 > 0 && net2 < 0 {
                    swap_data.from_mint = m2;
                    swap_data.to_mint = m1;
                    swap_data.from_amount = (s2 as u64).max((-net2) as u64);
                    swap_data.to_amount = (r1 as u64).max(net1 as u64);
                } else if net2 > 0 && net1 < 0 {
                    swap_data.from_mint = m1;
                    swap_data.to_mint = m2;
                    swap_data.from_amount = (s1 as u64).max((-net1) as u64);
                    swap_data.to_amount = (r2 as u64).max(net2 as u64);
                }
            }
        }
    }

    if swap_data.from_mint != Pubkey::default()
        && swap_data.to_mint != Pubkey::default()
        && swap_data.from_amount > 0
        && swap_data.to_amount > 0
    {
        (transfer_datas, Some(swap_data))
    } else {
        (transfer_datas, None)
    }
}