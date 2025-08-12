pub mod types;
pub mod utils;

/// Automatically generate UnifiedEvent trait implementation macro
#[macro_export]
macro_rules! impl_unified_event {
    // Version with just struct name (no extra fields)
    ($struct_name:ident) => {
        impl $crate::events::core::traits::UnifiedEvent for $struct_name {
            fn id(&self) -> &str {
                &self.metadata.id
            }

            fn event_type(&self) -> $crate::events::common::types::EventType {
                self.metadata.event_type.clone()
            }

            fn signature(&self) -> &str {
                &self.metadata.signature
            }

            fn slot(&self) -> u64 {
                self.metadata.slot
            }

            fn program_received_time_ms(&self) -> i64 {
                self.metadata.program_received_time_ms
            }

            fn program_handle_time_consuming_ms(&self) -> i64 {
                self.metadata.program_handle_time_consuming_ms
            }

            fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64) {
                self.metadata.program_handle_time_consuming_ms = program_handle_time_consuming_ms;
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn $crate::events::core::traits::UnifiedEvent> {
                Box::new(self.clone())
            }

            fn merge(&mut self, _other: Box<dyn $crate::events::core::traits::UnifiedEvent>) {
                // No extra fields to merge
            }

            fn set_transfer_datas(&mut self, transfer_datas: Vec<$crate::events::common::types::TransferData>, swap_data: Option<$crate::events::common::types::SwapData>) {
                self.metadata.set_transfer_datas(transfer_datas, swap_data);
            }

            fn index(&self) -> String {
                self.metadata.index.clone()
            }

            fn protocol_type(&self) -> $crate::events::common::types::ProtocolType {
                self.metadata.protocol.clone()
            }
        }
    };
    // Version with custom ID expression and extra fields
    ($struct_name:ident, $($field:ident),*) => {
        impl $crate::events::core::traits::UnifiedEvent for $struct_name {
            fn id(&self) -> &str {
                &self.metadata.id
            }

            fn event_type(&self) -> $crate::events::common::types::EventType {
                self.metadata.event_type.clone()
            }

            fn signature(&self) -> &str {
                &self.metadata.signature
            }

            fn slot(&self) -> u64 {
                self.metadata.slot
            }

            fn program_received_time_ms(&self) -> i64 {
                self.metadata.program_received_time_ms
            }

            fn program_handle_time_consuming_ms(&self) -> i64 {
                self.metadata.program_handle_time_consuming_ms
            }

            fn set_program_handle_time_consuming_ms(&mut self, program_handle_time_consuming_ms: i64) {
                self.metadata.program_handle_time_consuming_ms = program_handle_time_consuming_ms;
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn $crate::events::core::traits::UnifiedEvent> {
                Box::new(self.clone())
            }

            fn merge(&mut self, other: Box<dyn $crate::events::core::traits::UnifiedEvent>) {
                if let Some(_e) = other.as_any().downcast_ref::<$struct_name>() {
                    $(
                        self.$field = _e.$field.clone();
                    )*
                }
            }

            fn set_transfer_datas(&mut self, transfer_datas: Vec<$crate::events::common::types::TransferData>, swap_data: Option<$crate::events::common::types::SwapData>) {
                self.metadata.set_transfer_datas(transfer_datas, swap_data);
            }

            fn index(&self) -> String {
                self.metadata.index.clone()
            }

            fn protocol_type(&self) -> $crate::events::common::types::ProtocolType {
                self.metadata.protocol.clone()
            }
        }
    };
}

pub use types::*;
pub use utils::*;