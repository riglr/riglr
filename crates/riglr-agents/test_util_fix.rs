use riglr_agents::{TaskType, CapabilityType, util::task_type_to_capability};

fn main() {
    // Test all enum mappings to ensure they work correctly
    assert_eq!(task_type_to_capability(&TaskType::Trading), CapabilityType::Trading);
    assert_eq!(task_type_to_capability(&TaskType::Research), CapabilityType::Research);
    assert_eq!(task_type_to_capability(&TaskType::RiskAnalysis), CapabilityType::RiskAnalysis);
    assert_eq!(task_type_to_capability(&TaskType::Portfolio), CapabilityType::Portfolio);
    assert_eq!(task_type_to_capability(&TaskType::Monitoring), CapabilityType::Monitoring);
    
    // Test custom type
    let custom_task = TaskType::Custom("custom_strategy".to_string());
    let expected_capability = CapabilityType::Custom("custom_strategy".to_string());
    assert_eq!(task_type_to_capability(&custom_task), expected_capability);
    
    println!("All tests passed! The util function works correctly with explicit returns.");
}
