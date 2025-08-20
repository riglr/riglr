//! riglr-server: a turnkey, configurable HTTP server for riglr agents
//!
//! This crate provides a production-ready Axum/Actix server wiring around
//! `riglr-web-adapters`, exposing standardized endpoints for:
//! - SSE streaming: POST /v1/stream
//! - Completion:   POST /v1/completion
//! - Health:       GET  /health
//! - Info:         GET  /
//!
//! It integrates the SignerFactory pattern, structured logging, and room for metrics.

/// Server implementation providing HTTP endpoints for riglr agents.
pub mod server;

pub use server::{start_axum, ServerConfig};

#[cfg(feature = "actix")]
pub use server::start_actix;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_server_config_default_should_create_valid_config() {
        // Happy Path: Test default implementation
        let config = ServerConfig::default();

        assert_eq!(config.addr, "0.0.0.0:8080".parse::<SocketAddr>().unwrap());
        assert_eq!(config.addr.ip(), IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
        assert_eq!(config.addr.port(), 8080);

        // Verify configuration is properly initialized
    }

    #[test]
    fn test_server_config_clone_should_create_identical_copy() {
        // Test Clone trait implementation
        let original = ServerConfig::default();
        let cloned = original.clone();

        assert_eq!(original.addr, cloned.addr);
    }

    #[test]
    fn test_server_config_debug_should_format_properly() {
        // Test Debug trait implementation
        let config = ServerConfig::default();
        let debug_string = format!("{:?}", config);

        assert!(debug_string.contains("ServerConfig"));
        assert!(debug_string.contains("addr"));
        assert!(debug_string.contains("0.0.0.0:8080"));
    }

    #[test]
    fn test_server_config_custom_values_should_preserve_fields() {
        // Test with custom values
        let custom_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

        let config = ServerConfig { addr: custom_addr };

        assert_eq!(config.addr, custom_addr);
        assert_eq!(config.addr.ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(config.addr.port(), 3000);
    }

    #[test]
    fn test_server_config_edge_case_addresses() {
        // Edge cases for socket addresses
        let edge_cases = vec![
            "0.0.0.0:0",             // Port 0
            "255.255.255.255:65535", // Max IPv4 and port
            "127.0.0.1:1",           // Localhost with minimal port
            "192.168.1.1:8080",      // Private IP
        ];

        for addr_str in edge_cases {
            let addr: SocketAddr = addr_str.parse().unwrap();
            let config = ServerConfig { addr };

            assert_eq!(config.addr, addr);
            // Verify the config can be cloned and debugged without panics
            let _ = config.clone();
            let _ = format!("{:?}", config);
        }
    }

    #[test]
    fn test_module_re_exports_are_accessible() {
        // Test that re-exported items are accessible through the lib.rs interface

        // Test ServerConfig is accessible
        let _config: ServerConfig = ServerConfig::default();

        // Test that we can create a ServerConfig using the re-exported type
        let custom_config = ServerConfig {
            addr: "0.0.0.0:9000".parse().unwrap(),
        };
        assert_eq!(custom_config.addr.port(), 9000);
    }

    #[cfg(feature = "axum")]
    #[test]
    fn test_start_axum_function_exists() {
        // Test that start_axum function is accessible when axum feature is enabled
        // We can't easily test the actual function without mocking dependencies,
        // but we can verify it's properly re-exported by checking if the function compiles

        // Create a simple mock agent for testing
        #[derive(Clone)]
        struct TestAgent;

        #[async_trait::async_trait]
        impl riglr_web_adapters::Agent for TestAgent {
            type Error = std::io::Error;
            async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
                Ok("test".to_string())
            }
            async fn prompt_stream(
                &self,
                _prompt: &str,
            ) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>
            {
                Ok(Box::pin(futures_util::stream::empty()))
            }
        }

        // This should compile if the function is properly exported
        let _function_pointer = start_axum::<TestAgent>;
    }

    #[cfg(feature = "actix")]
    #[test]
    fn test_start_actix_function_exists() {
        // Test that start_actix function is accessible when actix feature is enabled
        // We can't easily test the actual function without mocking dependencies,
        // but we can verify it's properly re-exported by checking if the function compiles

        // Create a simple mock agent for testing
        #[derive(Clone)]
        struct TestAgent;

        #[async_trait::async_trait]
        impl riglr_web_adapters::Agent for TestAgent {
            type Error = std::io::Error;
            async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
                Ok("test".to_string())
            }
            async fn prompt_stream(
                &self,
                _prompt: &str,
            ) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>
            {
                Ok(Box::pin(futures_util::stream::empty()))
            }
        }

        // This should compile if the function is properly exported
        let _function_pointer = start_actix::<TestAgent>;
    }

    #[test]
    fn test_server_config_partial_eq_behavior() {
        // Test equality behavior for ServerConfig
        let config1 = ServerConfig::default();
        let config2 = ServerConfig::default();

        // Even though ServerConfig doesn't derive PartialEq, we can test field-by-field equality
        assert_eq!(config1.addr, config2.addr);

        // Test inequality
        let config3 = ServerConfig {
            addr: "127.0.0.1:8081".parse().unwrap(),
        };
        assert_ne!(config1.addr, config3.addr);
    }

    #[test]
    fn test_server_config_memory_layout() {
        // Test that ServerConfig has expected memory characteristics
        let config = ServerConfig::default();

        // Verify the struct has the expected fields accessible
        let _addr = config.addr;

        // Verify size constraints (struct should not be unexpectedly large)
        let size = std::mem::size_of::<ServerConfig>();
        // SocketAddr is typically 28 bytes
        assert!(size > 0, "ServerConfig should have non-zero size");
        assert!(size < 1024, "ServerConfig should not be unexpectedly large");
    }
}
