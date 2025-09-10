//! Centralized dependency version management
//!
//! This module contains pinned, known-good versions of all external dependencies
//! used in generated projects. This ensures reproducible builds and eliminates
//! the need for network access during project generation.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Known-good versions for all external dependencies
pub static DEPENDENCY_VERSIONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut deps = HashMap::new();

    // Core dependencies
    deps.insert("tokio", "1.40");
    deps.insert("anyhow", "1.0");
    deps.insert("tracing", "0.1");
    deps.insert("tracing-subscriber", "0.3");
    deps.insert("serde", "1.0");
    deps.insert("serde_json", "1.0");
    deps.insert("chrono", "0.4");
    deps.insert("async-trait", "0.1");

    // Server frameworks
    deps.insert("actix-web", "4.9");
    deps.insert("actix-cors", "0.7");
    deps.insert("actix-web-lab", "0.20");
    deps.insert("axum", "0.7");
    deps.insert("tower", "0.5");
    deps.insert("tower-http", "0.6");
    deps.insert("warp", "0.3");
    deps.insert("rocket", "0.5");
    deps.insert("rocket_cors", "0.6");
    deps.insert("tokio-stream", "0.1");

    // Database and storage
    deps.insert("redis", "0.27");
    deps.insert("sqlx", "0.8");

    // Blockchain specific
    deps.insert("solana-client", "2.0");
    deps.insert("solana-sdk", "2.0");
    deps.insert("alloy", "0.6");

    // HTTP and networking
    deps.insert("reqwest", "0.12");
    deps.insert("hyper", "1.4");

    // Utilities
    deps.insert("uuid", "1.10");
    deps.insert("once_cell", "1.20");
    deps.insert("lazy_static", "1.5");
    deps.insert("regex", "1.11");
    deps.insert("dotenv", "0.15");

    deps
});

/// Get a specific dependency version with proper caret prefix
pub fn get_dependency_version(name: &str) -> String {
    DEPENDENCY_VERSIONS
        .get(name)
        .map(|v| format!("^{}", v))
        .unwrap_or_else(|| {
            tracing::warn!("No pinned version found for dependency: {}, using ^1", name);
            "^1".to_string()
        })
}

/// Get a dependency version without caret prefix (for exact version requirements)
#[allow(dead_code)]
pub fn get_exact_version(name: &str) -> String {
    DEPENDENCY_VERSIONS
        .get(name)
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            tracing::warn!("No pinned version found for dependency: {}, using 1", name);
            "1".to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dependency_version_known_crate() {
        assert_eq!(get_dependency_version("tokio"), "^1.40");
        assert_eq!(get_dependency_version("axum"), "^0.7");
        assert_eq!(get_dependency_version("serde"), "^1.0");
    }

    #[test]
    fn test_get_dependency_version_unknown_crate() {
        assert_eq!(get_dependency_version("unknown-crate"), "^1");
    }

    #[test]
    fn test_get_exact_version_known_crate() {
        assert_eq!(get_exact_version("tokio"), "1.40");
        assert_eq!(get_exact_version("axum"), "0.7");
    }

    #[test]
    fn test_all_server_frameworks_have_versions() {
        let frameworks = vec![
            "actix-web",
            "actix-cors",
            "axum",
            "tower",
            "tower-http",
            "warp",
            "rocket",
            "rocket_cors",
        ];

        for framework in frameworks {
            let version = get_dependency_version(framework);
            assert_ne!(
                version, "^1",
                "Framework {} should have a pinned version",
                framework
            );
        }
    }

    #[test]
    fn test_blockchain_dependencies_have_versions() {
        assert_eq!(get_dependency_version("solana-client"), "^2.0");
        assert_eq!(get_dependency_version("solana-sdk"), "^2.0");
        assert_eq!(get_dependency_version("alloy"), "^0.6");
    }
}
