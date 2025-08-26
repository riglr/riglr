//! Utility functions for the agent system.

use crate::{CapabilityType, TaskType};

/// Convert task type to capability type.
///
/// This function maps task types to their corresponding capability types.
pub fn task_type_to_capability(task_type: &TaskType) -> CapabilityType {
    match task_type {
        TaskType::Trading => CapabilityType::Trading,
        TaskType::Research => CapabilityType::Research,
        TaskType::RiskAnalysis => CapabilityType::RiskAnalysis,
        TaskType::Portfolio => CapabilityType::Portfolio,
        TaskType::Monitoring => CapabilityType::Monitoring,
        TaskType::Custom(name) => CapabilityType::Custom(name.clone()),
    }
}
