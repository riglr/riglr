//! Comprehensive coverage tests for riglr-core
//! 
//! This test file contains all tests needed to achieve 100% code coverage

// Include all the test modules
mod unit {
    mod config_tests {
        include!("unit/config_tests.rs");
    }
    
    mod signer_additional_tests {
        include!("unit/signer_additional_tests.rs");
    }
    
    mod granular_traits_tests {
        include!("unit/granular_traits_tests.rs");
    }
    
    mod traits_tests {
        include!("unit/traits_tests.rs");
    }
    
    mod evm_processor_tests {
        include!("unit/evm_processor_tests.rs");
    }
}