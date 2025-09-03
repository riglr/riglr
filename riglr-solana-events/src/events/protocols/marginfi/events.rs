use super::types::{
    marginfi_program_id, MarginFiBorrowData, MarginFiDepositData, MarginFiLiquidationData,
    MarginFiRepayData, MarginFiWithdrawData,
};
use crate::solana_metadata::SolanaEventMetadata;
use crate::types::{metadata_helpers, EventType, ProtocolType, TransferData};
use riglr_events_core::EventMetadata as CoreEventMetadata;
use serde::{Deserialize, Serialize};
use std::any::Any;

// Import new Event trait from riglr-events-core
use riglr_events_core::{Event, EventKind};

/// Parameters for creating event metadata, reducing function parameter count
#[derive(Debug, Clone, Default)]
pub struct EventParameters {
    /// Unique identifier for the event
    pub id: String,
    /// Transaction signature hash
    pub signature: String,
    /// Solana slot number when the transaction was processed
    pub slot: u64,
    /// Block timestamp in seconds since Unix epoch
    pub block_time: i64,
    /// Block timestamp in milliseconds since Unix epoch
    pub block_time_ms: i64,
    /// Timestamp when the program received the transaction in milliseconds
    pub program_received_time_ms: i64,
    /// Index of the instruction within the transaction
    pub index: String,
}

impl EventParameters {
    /// Creates a new EventParameters instance with the provided values
    pub fn new(
        id: String,
        signature: String,
        slot: u64,
        block_time: i64,
        block_time_ms: i64,
        program_received_time_ms: i64,
        index: String,
    ) -> Self {
        Self {
            id,
            signature,
            slot,
            block_time,
            block_time_ms,
            program_received_time_ms,
            index,
        }
    }
}

/// MarginFi deposit event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiDepositEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific deposit operation data
    pub deposit_data: MarginFiDepositData,
    /// Associated token transfer data for this deposit
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiDepositEvent {
    /// Creates a new MarginFi deposit event with the provided parameters and deposit data
    pub fn new(params: EventParameters, deposit_data: MarginFiDepositData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MarginFi,
            EventType::Deposit,
            marginfi_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            deposit_data,
            transfer_data: Vec::default(),
        }
    }

    /// Adds transfer data to the deposit event and returns the modified event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiDepositEvent
impl Event for MarginFiDepositEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

/// MarginFi withdraw event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiWithdrawEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific withdraw operation data
    pub withdraw_data: MarginFiWithdrawData,
    /// Associated token transfer data for this withdrawal
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiWithdrawEvent {
    /// Creates a new MarginFi withdraw event with the provided parameters and withdraw data
    pub fn new(params: EventParameters, withdraw_data: MarginFiWithdrawData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MarginFi,
            EventType::Withdraw,
            marginfi_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            withdraw_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this withdraw event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiWithdrawEvent
impl Event for MarginFiWithdrawEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static LIQUIDITY_KIND: EventKind = EventKind::Liquidity;
        &LIQUIDITY_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

/// MarginFi borrow event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiBorrowEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific borrow operation data
    pub borrow_data: MarginFiBorrowData,
    /// Associated token transfer data for this borrow
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiBorrowEvent {
    /// Creates a new MarginFi borrow event with the provided parameters and borrow data
    pub fn new(params: EventParameters, borrow_data: MarginFiBorrowData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MarginFi,
            EventType::Borrow,
            marginfi_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            borrow_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this borrow event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// MarginFi repay event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiRepayEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific repay operation data
    pub repay_data: MarginFiRepayData,
    /// Associated token transfer data for this repayment
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiRepayEvent {
    /// Creates a new MarginFi repay event with the provided parameters and repay data
    pub fn new(params: EventParameters, repay_data: MarginFiRepayData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MarginFi,
            EventType::Repay,
            marginfi_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            repay_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this repay event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

/// MarginFi liquidation event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginFiLiquidationEvent {
    /// Event metadata
    pub metadata: SolanaEventMetadata,
    /// MarginFi-specific liquidation operation data
    pub liquidation_data: MarginFiLiquidationData,
    /// Associated token transfer data for this liquidation
    pub transfer_data: Vec<TransferData>,
}

impl MarginFiLiquidationEvent {
    /// Creates a new MarginFi liquidation event with the provided parameters and liquidation data
    pub fn new(params: EventParameters, liquidation_data: MarginFiLiquidationData) -> Self {
        let metadata = metadata_helpers::create_solana_metadata(
            params.id,
            params.signature,
            params.slot,
            params.block_time,
            ProtocolType::MarginFi,
            EventType::Liquidate,
            marginfi_program_id(),
            params.index,
            params.program_received_time_ms,
        );

        Self {
            metadata,
            liquidation_data,
            transfer_data: Vec::default(),
        }
    }

    /// Sets the transfer data for this liquidation event
    pub fn with_transfer_data(mut self, transfer_data: Vec<TransferData>) -> Self {
        self.transfer_data = transfer_data;
        self
    }
}

// New Event trait implementation for MarginFiBorrowEvent
impl Event for MarginFiBorrowEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

// New Event trait implementation for MarginFiRepayEvent
impl Event for MarginFiRepayEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

// New Event trait implementation for MarginFiLiquidationEvent
impl Event for MarginFiLiquidationEvent {
    fn id(&self) -> &str {
        &self.metadata.core.id
    }

    fn kind(&self) -> &EventKind {
        static TRANSFER_KIND: EventKind = EventKind::Transfer;
        &TRANSFER_KIND
    }

    fn metadata(&self) -> &CoreEventMetadata {
        &self.metadata.core
    }

    fn metadata_mut(&mut self) -> &mut CoreEventMetadata {
        &mut self.metadata.core
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        serde_json::to_value(self).map_err(|e| {
            riglr_events_core::error::EventError::generic(format!("Serialization failed: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    fn create_test_event_parameters() -> EventParameters {
        EventParameters::new(
            "test_id".to_string(),
            "test_signature".to_string(),
            12345,
            1234567890,
            1234567890000,
            1234567890001,
            "test_index".to_string(),
        )
    }

    fn create_test_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    fn create_test_transfer_data() -> TransferData {
        TransferData {
            source: create_test_pubkey(),
            destination: create_test_pubkey(),
            mint: Some(create_test_pubkey()),
            amount: 1000,
        }
    }

    // EventParameters tests
    #[test]
    fn test_event_parameters_new_when_valid_params_should_create_instance() {
        let params = EventParameters::new(
            "id1".to_string(),
            "sig1".to_string(),
            100,
            1000,
            1000000,
            1000001,
            "idx1".to_string(),
        );

        assert_eq!(params.id, "id1");
        assert_eq!(params.signature, "sig1");
        assert_eq!(params.slot, 100);
        assert_eq!(params.block_time, 1000);
        assert_eq!(params.block_time_ms, 1000000);
        assert_eq!(params.program_received_time_ms, 1000001);
        assert_eq!(params.index, "idx1");
    }

    #[test]
    fn test_event_parameters_default_when_called_should_create_empty_instance() {
        let params = EventParameters::default();

        assert_eq!(params.id, "");
        assert_eq!(params.signature, "");
        assert_eq!(params.slot, 0);
        assert_eq!(params.block_time, 0);
        assert_eq!(params.block_time_ms, 0);
        assert_eq!(params.program_received_time_ms, 0);
        assert_eq!(params.index, "");
    }

    // MarginFiDepositEvent tests
    #[test]
    fn test_marginfi_deposit_event_new_when_valid_params_should_create_event() {
        let params = create_test_event_parameters();
        let deposit_data = MarginFiDepositData {
            marginfi_group: create_test_pubkey(),
            marginfi_account: create_test_pubkey(),
            signer: create_test_pubkey(),
            bank: create_test_pubkey(),
            token_account: create_test_pubkey(),
            bank_liquidity_vault: create_test_pubkey(),
            token_program: create_test_pubkey(),
            amount: 5000,
        };

        let event = MarginFiDepositEvent::new(params.clone(), deposit_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            params.block_time
        );
        assert_eq!(event.metadata.index, params.index);
        assert_eq!(
            event.metadata.program_received_time_ms,
            params.program_received_time_ms
        );
        assert_eq!(event.deposit_data.amount, deposit_data.amount);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_deposit_event_with_transfer_data_when_called_should_set_transfer_data() {
        let params = create_test_event_parameters();
        let deposit_data = MarginFiDepositData::default();
        let transfer_data = vec![create_test_transfer_data()];

        let event = MarginFiDepositEvent::new(params, deposit_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, transfer_data[0].amount);
    }

    #[test]
    fn test_marginfi_deposit_event_default_when_called_should_create_empty_event() {
        let event = MarginFiDepositEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(
            crate::metadata_helpers::get_block_time(&event.metadata.core).unwrap_or(0),
            0
        );
        assert!(event.transfer_data.is_empty());
    }

    // Event trait tests for MarginFiDepositEvent
    #[test]
    fn test_marginfi_deposit_event_id_when_called_should_return_metadata_id() {
        let event = MarginFiDepositEvent::default();
        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_marginfi_deposit_event_kind_when_called_should_return_metadata_kind() {
        let event = MarginFiDepositEvent::default();
        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_marginfi_deposit_event_metadata_when_called_should_return_metadata_reference() {
        let event = MarginFiDepositEvent::default();
        let metadata_ref = event.metadata();
        assert_eq!(metadata_ref.id, event.metadata.id());
    }

    #[test]
    fn test_marginfi_deposit_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = MarginFiDepositEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "new_id".to_string();
        assert_eq!(event.metadata.id(), "new_id");
    }

    #[test]
    fn test_marginfi_deposit_event_as_any_when_called_should_return_any_reference() {
        let event = MarginFiDepositEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MarginFiDepositEvent>().is_some());
    }

    #[test]
    fn test_marginfi_deposit_event_as_any_mut_when_called_should_return_mutable_any() {
        let mut event = MarginFiDepositEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<MarginFiDepositEvent>().is_some());
    }

    #[test]
    fn test_marginfi_deposit_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = MarginFiDepositEvent::default();
        let boxed_clone = event.clone_boxed();
        assert!(boxed_clone
            .as_any()
            .downcast_ref::<MarginFiDepositEvent>()
            .is_some());
    }

    #[test]
    fn test_marginfi_deposit_event_to_json_when_called_should_serialize_to_json() {
        let event = MarginFiDepositEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    // MarginFiWithdrawEvent tests
    #[test]
    fn test_marginfi_withdraw_event_new_when_valid_params_should_create_event() {
        let params = create_test_event_parameters();
        let withdraw_data = MarginFiWithdrawData {
            marginfi_group: create_test_pubkey(),
            marginfi_account: create_test_pubkey(),
            signer: create_test_pubkey(),
            bank: create_test_pubkey(),
            token_account: create_test_pubkey(),
            bank_liquidity_vault: create_test_pubkey(),
            bank_liquidity_vault_authority: create_test_pubkey(),
            token_program: create_test_pubkey(),
            amount: 3000,
            withdraw_all: false,
        };

        let event = MarginFiWithdrawEvent::new(params.clone(), withdraw_data.clone());

        assert_eq!(event.metadata.core.id, params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.withdraw_data.amount, withdraw_data.amount);
        assert_eq!(event.withdraw_data.withdraw_all, withdraw_data.withdraw_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_withdraw_event_with_transfer_data_when_called_should_set_transfer_data() {
        let params = create_test_event_parameters();
        let withdraw_data = MarginFiWithdrawData::default();
        let transfer_data = vec![create_test_transfer_data(), create_test_transfer_data()];

        let event = MarginFiWithdrawEvent::new(params, withdraw_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
    }

    #[test]
    fn test_marginfi_withdraw_event_default_when_called_should_create_empty_event() {
        let event = MarginFiWithdrawEvent::default();

        assert_eq!(event.metadata.core.id, "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.withdraw_data.amount, 0);
        assert!(!event.withdraw_data.withdraw_all);
        assert!(event.transfer_data.is_empty());
    }

    // Event trait tests for MarginFiWithdrawEvent
    #[test]
    fn test_marginfi_withdraw_event_id_when_called_should_return_metadata_id() {
        let event = MarginFiWithdrawEvent::default();
        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_marginfi_withdraw_event_kind_when_called_should_return_metadata_kind() {
        let event = MarginFiWithdrawEvent::default();
        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_marginfi_withdraw_event_metadata_when_called_should_return_metadata_reference() {
        let event = MarginFiWithdrawEvent::default();
        let metadata_ref = event.metadata();
        assert_eq!(metadata_ref.id, event.metadata.id());
    }

    #[test]
    fn test_marginfi_withdraw_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = MarginFiWithdrawEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "withdraw_id".to_string();
        assert_eq!(event.metadata.id(), "withdraw_id");
    }

    #[test]
    fn test_marginfi_withdraw_event_as_any_when_called_should_return_any_reference() {
        let event = MarginFiWithdrawEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MarginFiWithdrawEvent>().is_some());
    }

    #[test]
    fn test_marginfi_withdraw_event_as_any_mut_when_called_should_return_mutable_any() {
        let mut event = MarginFiWithdrawEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<MarginFiWithdrawEvent>().is_some());
    }

    #[test]
    fn test_marginfi_withdraw_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = MarginFiWithdrawEvent::default();
        let boxed_clone = event.clone_boxed();
        assert!(boxed_clone
            .as_any()
            .downcast_ref::<MarginFiWithdrawEvent>()
            .is_some());
    }

    #[test]
    fn test_marginfi_withdraw_event_to_json_when_called_should_serialize_to_json() {
        let event = MarginFiWithdrawEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    // MarginFiBorrowEvent tests
    #[test]
    fn test_marginfi_borrow_event_new_when_valid_params_should_create_event() {
        let params = create_test_event_parameters();
        let borrow_data = MarginFiBorrowData {
            marginfi_group: create_test_pubkey(),
            marginfi_account: create_test_pubkey(),
            signer: create_test_pubkey(),
            bank: create_test_pubkey(),
            token_account: create_test_pubkey(),
            bank_liquidity_vault: create_test_pubkey(),
            bank_liquidity_vault_authority: create_test_pubkey(),
            token_program: create_test_pubkey(),
            amount: 7500,
        };

        let event = MarginFiBorrowEvent::new(params.clone(), borrow_data.clone());

        assert_eq!(event.id(), &params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.borrow_data.amount, borrow_data.amount);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_borrow_event_with_transfer_data_when_called_should_set_transfer_data() {
        let params = create_test_event_parameters();
        let borrow_data = MarginFiBorrowData::default();
        let transfer_data = vec![create_test_transfer_data()];

        let event =
            MarginFiBorrowEvent::new(params, borrow_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, transfer_data[0].amount);
    }

    #[test]
    fn test_marginfi_borrow_event_default_when_called_should_create_empty_event() {
        let event = MarginFiBorrowEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.borrow_data.amount, 0);
        assert!(event.transfer_data.is_empty());
    }

    // Event trait tests for MarginFiBorrowEvent
    #[test]
    fn test_marginfi_borrow_event_id_when_called_should_return_metadata_id() {
        let event = MarginFiBorrowEvent::default();
        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_marginfi_borrow_event_kind_when_called_should_return_metadata_kind() {
        let event = MarginFiBorrowEvent::default();
        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_marginfi_borrow_event_metadata_when_called_should_return_metadata_reference() {
        let event = MarginFiBorrowEvent::default();
        let metadata_ref = event.metadata();
        assert_eq!(metadata_ref.id, event.metadata.id());
    }

    #[test]
    fn test_marginfi_borrow_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = MarginFiBorrowEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "borrow_id".to_string();
        assert_eq!(event.metadata.id(), "borrow_id");
    }

    #[test]
    fn test_marginfi_borrow_event_as_any_when_called_should_return_any_reference() {
        let event = MarginFiBorrowEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MarginFiBorrowEvent>().is_some());
    }

    #[test]
    fn test_marginfi_borrow_event_as_any_mut_when_called_should_return_mutable_any() {
        let mut event = MarginFiBorrowEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<MarginFiBorrowEvent>().is_some());
    }

    #[test]
    fn test_marginfi_borrow_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = MarginFiBorrowEvent::default();
        let boxed_clone = event.clone_boxed();
        assert!(boxed_clone
            .as_any()
            .downcast_ref::<MarginFiBorrowEvent>()
            .is_some());
    }

    #[test]
    fn test_marginfi_borrow_event_to_json_when_called_should_serialize_to_json() {
        let event = MarginFiBorrowEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    // MarginFiRepayEvent tests
    #[test]
    fn test_marginfi_repay_event_new_when_valid_params_should_create_event() {
        let params = create_test_event_parameters();
        let repay_data = MarginFiRepayData {
            marginfi_group: create_test_pubkey(),
            marginfi_account: create_test_pubkey(),
            signer: create_test_pubkey(),
            bank: create_test_pubkey(),
            token_account: create_test_pubkey(),
            bank_liquidity_vault: create_test_pubkey(),
            token_program: create_test_pubkey(),
            amount: 2500,
            repay_all: true,
        };

        let event = MarginFiRepayEvent::new(params.clone(), repay_data.clone());

        assert_eq!(event.id(), &params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(event.repay_data.amount, repay_data.amount);
        assert_eq!(event.repay_data.repay_all, repay_data.repay_all);
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_repay_event_with_transfer_data_when_called_should_set_transfer_data() {
        let params = create_test_event_parameters();
        let repay_data = MarginFiRepayData::default();
        let transfer_data = vec![create_test_transfer_data(), create_test_transfer_data()];

        let event =
            MarginFiRepayEvent::new(params, repay_data).with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 2);
    }

    #[test]
    fn test_marginfi_repay_event_default_when_called_should_create_empty_event() {
        let event = MarginFiRepayEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.repay_data.amount, 0);
        assert!(!event.repay_data.repay_all);
        assert!(event.transfer_data.is_empty());
    }

    // Event trait tests for MarginFiRepayEvent
    #[test]
    fn test_marginfi_repay_event_id_when_called_should_return_metadata_id() {
        let event = MarginFiRepayEvent::default();
        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_marginfi_repay_event_kind_when_called_should_return_metadata_kind() {
        let event = MarginFiRepayEvent::default();
        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_marginfi_repay_event_metadata_when_called_should_return_metadata_reference() {
        let event = MarginFiRepayEvent::default();
        let metadata_ref = event.metadata();
        assert_eq!(metadata_ref.id, event.metadata.id());
    }

    #[test]
    fn test_marginfi_repay_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = MarginFiRepayEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "repay_id".to_string();
        assert_eq!(event.metadata.id(), "repay_id");
    }

    #[test]
    fn test_marginfi_repay_event_as_any_when_called_should_return_any_reference() {
        let event = MarginFiRepayEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MarginFiRepayEvent>().is_some());
    }

    #[test]
    fn test_marginfi_repay_event_as_any_mut_when_called_should_return_mutable_any() {
        let mut event = MarginFiRepayEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<MarginFiRepayEvent>().is_some());
    }

    #[test]
    fn test_marginfi_repay_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = MarginFiRepayEvent::default();
        let boxed_clone = event.clone_boxed();
        assert!(boxed_clone
            .as_any()
            .downcast_ref::<MarginFiRepayEvent>()
            .is_some());
    }

    #[test]
    fn test_marginfi_repay_event_to_json_when_called_should_serialize_to_json() {
        let event = MarginFiRepayEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    // MarginFiLiquidationEvent tests
    #[test]
    fn test_marginfi_liquidation_event_new_when_valid_params_should_create_event() {
        let params = create_test_event_parameters();
        let liquidation_data = MarginFiLiquidationData {
            marginfi_group: create_test_pubkey(),
            asset_bank: create_test_pubkey(),
            liab_bank: create_test_pubkey(),
            liquidatee_marginfi_account: create_test_pubkey(),
            liquidator_marginfi_account: create_test_pubkey(),
            liquidator: create_test_pubkey(),
            asset_bank_liquidity_vault: create_test_pubkey(),
            liab_bank_liquidity_vault: create_test_pubkey(),
            liquidator_token_account: create_test_pubkey(),
            token_program: create_test_pubkey(),
            asset_amount: 8000,
            liab_amount: 9000,
        };

        let event = MarginFiLiquidationEvent::new(params.clone(), liquidation_data.clone());

        assert_eq!(event.id(), &params.id);
        assert_eq!(event.metadata.signature, params.signature);
        assert_eq!(event.metadata.slot, params.slot);
        assert_eq!(
            event.liquidation_data.asset_amount,
            liquidation_data.asset_amount
        );
        assert_eq!(
            event.liquidation_data.liab_amount,
            liquidation_data.liab_amount
        );
        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_marginfi_liquidation_event_with_transfer_data_when_called_should_set_transfer_data() {
        let params = create_test_event_parameters();
        let liquidation_data = MarginFiLiquidationData::default();
        let transfer_data = vec![create_test_transfer_data()];

        let event = MarginFiLiquidationEvent::new(params, liquidation_data)
            .with_transfer_data(transfer_data.clone());

        assert_eq!(event.transfer_data.len(), 1);
        assert_eq!(event.transfer_data[0].amount, transfer_data[0].amount);
    }

    #[test]
    fn test_marginfi_liquidation_event_default_when_called_should_create_empty_event() {
        let event = MarginFiLiquidationEvent::default();

        assert_eq!(event.id(), "");
        assert_eq!(event.metadata.signature, "");
        assert_eq!(event.metadata.slot, 0);
        assert_eq!(event.liquidation_data.asset_amount, 0);
        assert_eq!(event.liquidation_data.liab_amount, 0);
        assert!(event.transfer_data.is_empty());
    }

    // Event trait tests for MarginFiLiquidationEvent
    #[test]
    fn test_marginfi_liquidation_event_id_when_called_should_return_metadata_id() {
        let event = MarginFiLiquidationEvent::default();
        assert_eq!(event.id(), event.metadata.id());
    }

    #[test]
    fn test_marginfi_liquidation_event_kind_when_called_should_return_metadata_kind() {
        let event = MarginFiLiquidationEvent::default();
        assert_eq!(event.kind(), event.metadata.kind());
    }

    #[test]
    fn test_marginfi_liquidation_event_metadata_when_called_should_return_metadata_reference() {
        let event = MarginFiLiquidationEvent::default();
        let metadata_ref = event.metadata();
        assert_eq!(metadata_ref.id, event.metadata.id());
    }

    #[test]
    fn test_marginfi_liquidation_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = MarginFiLiquidationEvent::default();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "liquidation_id".to_string();
        assert_eq!(event.metadata.id(), "liquidation_id");
    }

    #[test]
    fn test_marginfi_liquidation_event_as_any_when_called_should_return_any_reference() {
        let event = MarginFiLiquidationEvent::default();
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MarginFiLiquidationEvent>().is_some());
    }

    #[test]
    fn test_marginfi_liquidation_event_as_any_mut_when_called_should_return_mutable_any() {
        let mut event = MarginFiLiquidationEvent::default();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<MarginFiLiquidationEvent>().is_some());
    }

    #[test]
    fn test_marginfi_liquidation_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = MarginFiLiquidationEvent::default();
        let boxed_clone = event.clone_boxed();
        assert!(boxed_clone
            .as_any()
            .downcast_ref::<MarginFiLiquidationEvent>()
            .is_some());
    }

    #[test]
    fn test_marginfi_liquidation_event_to_json_when_called_should_serialize_to_json() {
        let event = MarginFiLiquidationEvent::default();
        let json_result = event.to_json();
        assert!(json_result.is_ok());
    }

    // Edge case tests
    #[test]
    fn test_event_parameters_with_empty_strings_when_created_should_handle_gracefully() {
        let params = EventParameters::new(
            String::default(),
            String::default(),
            0,
            0,
            0,
            0,
            String::default(),
        );

        assert!(params.id.is_empty());
        assert!(params.signature.is_empty());
        assert!(params.index.is_empty());
        assert_eq!(params.slot, 0);
    }

    #[test]
    fn test_event_parameters_with_max_values_when_created_should_handle_gracefully() {
        let params = EventParameters::new(
            "max_id".to_string(),
            "max_signature".to_string(),
            u64::MAX,
            i64::MAX,
            i64::MAX,
            i64::MAX,
            "max_index".to_string(),
        );

        assert_eq!(params.slot, u64::MAX);
        assert_eq!(params.block_time, i64::MAX);
        assert_eq!(params.block_time_ms, i64::MAX);
        assert_eq!(params.program_received_time_ms, i64::MAX);
    }

    #[test]
    fn test_deposit_event_with_empty_transfer_data_when_called_should_clear_existing_data() {
        let params = create_test_event_parameters();
        let deposit_data = MarginFiDepositData::default();
        let initial_transfer_data = vec![create_test_transfer_data()];

        let event = MarginFiDepositEvent::new(params, deposit_data)
            .with_transfer_data(initial_transfer_data)
            .with_transfer_data(Vec::default());

        assert!(event.transfer_data.is_empty());
    }

    #[test]
    fn test_all_events_with_zero_amounts_when_created_should_handle_gracefully() {
        let params = create_test_event_parameters();

        let deposit_data = MarginFiDepositData {
            amount: 0,
            ..Default::default()
        };
        let deposit_event = MarginFiDepositEvent::new(params.clone(), deposit_data);
        assert_eq!(deposit_event.deposit_data.amount, 0);

        let withdraw_data = MarginFiWithdrawData {
            amount: 0,
            ..Default::default()
        };
        let withdraw_event = MarginFiWithdrawEvent::new(params.clone(), withdraw_data);
        assert_eq!(withdraw_event.withdraw_data.amount, 0);

        let borrow_data = MarginFiBorrowData {
            amount: 0,
            ..Default::default()
        };
        let borrow_event = MarginFiBorrowEvent::new(params.clone(), borrow_data);
        assert_eq!(borrow_event.borrow_data.amount, 0);

        let repay_data = MarginFiRepayData {
            amount: 0,
            ..Default::default()
        };
        let repay_event = MarginFiRepayEvent::new(params.clone(), repay_data);
        assert_eq!(repay_event.repay_data.amount, 0);

        let liquidation_data = MarginFiLiquidationData {
            asset_amount: 0,
            liab_amount: 0,
            ..Default::default()
        };
        let liquidation_event = MarginFiLiquidationEvent::new(params, liquidation_data);
        assert_eq!(liquidation_event.liquidation_data.asset_amount, 0);
        assert_eq!(liquidation_event.liquidation_data.liab_amount, 0);
    }

    #[test]
    fn test_transfer_data_with_none_mint_when_created_should_handle_gracefully() {
        let transfer_data = TransferData {
            source: create_test_pubkey(),
            destination: create_test_pubkey(),
            mint: None,
            amount: 1000,
        };

        let params = create_test_event_parameters();
        let deposit_data = MarginFiDepositData::default();
        let event =
            MarginFiDepositEvent::new(params, deposit_data).with_transfer_data(vec![transfer_data]);

        assert!(event.transfer_data[0].mint.is_none());
        assert_eq!(event.transfer_data[0].amount, 1000);
    }

    // Test JSON serialization with actual data
    #[test]
    fn test_all_events_to_json_with_data_when_called_should_serialize_successfully() {
        let params = create_test_event_parameters();
        let transfer_data = vec![create_test_transfer_data()];

        let deposit_event =
            MarginFiDepositEvent::new(params.clone(), MarginFiDepositData::default())
                .with_transfer_data(transfer_data.clone());
        assert!(deposit_event.to_json().is_ok());

        let withdraw_event =
            MarginFiWithdrawEvent::new(params.clone(), MarginFiWithdrawData::default())
                .with_transfer_data(transfer_data.clone());
        assert!(withdraw_event.to_json().is_ok());

        let borrow_event = MarginFiBorrowEvent::new(params.clone(), MarginFiBorrowData::default())
            .with_transfer_data(transfer_data.clone());
        assert!(borrow_event.to_json().is_ok());

        let repay_event = MarginFiRepayEvent::new(params.clone(), MarginFiRepayData::default())
            .with_transfer_data(transfer_data.clone());
        assert!(repay_event.to_json().is_ok());

        let liquidation_event =
            MarginFiLiquidationEvent::new(params, MarginFiLiquidationData::default())
                .with_transfer_data(transfer_data);
        assert!(liquidation_event.to_json().is_ok());
    }
}
