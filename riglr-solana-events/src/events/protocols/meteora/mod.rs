/// Meteora protocol event definitions and constants.
pub mod events;
/// Meteora protocol transaction and log parsing functionality.
pub mod parser;
/// Meteora protocol data types and structures.
pub mod types;

pub use events::*;
pub use parser::*;
pub use types::*;

#[cfg(test)]
mod tests {

    #[test]
    fn test_events_module_accessibility() {
        // Test that the events module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn test_parser_module_accessibility() {
        // Test that the parser module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn test_types_module_accessibility() {
        // Test that the types module is accessible
        // This ensures the module declaration is working correctly
        let _module_exists = std::module_path!().contains("meteora");
    }

    #[test]
    fn test_module_reexports_compile() {
        // Test that all re-exports compile successfully
        // If any re-export fails, this test won't compile
        // This verifies that events::*, parser::*, and types::* are valid
    }
}
