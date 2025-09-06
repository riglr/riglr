//! Test helper utilities and constants for configuration testing
//!
//! This module provides placeholder patterns and helper functions for testing
//! database URLs and other configuration values without hardcoding credentials.

/// Environment variable names for testing
mod env_vars {
    /// Test database username environment variable
    pub const TEST_DB_USER: &str = "TEST_DB_USER";
    /// Test database password environment variable
    pub const TEST_DB_PASS: &str = "TEST_DB_PASS";
    /// Test WebSocket username environment variable
    pub const TEST_WS_USER: &str = "TEST_WS_USER";
    /// Test WebSocket password environment variable
    pub const TEST_WS_PASS: &str = "TEST_WS_PASS";
}

/// Placeholder patterns for test credentials
pub mod placeholders {
    /// Placeholder for username in URL templates
    pub const USERNAME: &str = "<username>";
    /// Placeholder for password in URL templates
    pub const PASSWORD: &str = "<password>";
    /// Placeholder for database name in URL templates
    pub const DATABASE: &str = "<database>";
    /// Placeholder for host address in URL templates
    pub const HOST: &str = "<host>";
    /// Placeholder for port number in URL templates
    pub const PORT: &str = "<port>";
}

/// Test URL templates with placeholders
pub mod templates {
    /// PostgreSQL URL template
    pub const POSTGRES_URL: &str = "postgresql://<username>:<password>@<host>:<port>/<database>";

    /// MySQL URL template
    pub const MYSQL_URL: &str = "mysql://<username>:<password>@<host>:<port>/<database>";

    /// MongoDB URL template
    pub const MONGODB_URL: &str = "mongodb://<username>:<password>@<host>:<port>/<database>";

    /// Redis URL template
    pub const REDIS_URL: &str = "redis://<username>:<password>@<host>:<port>/<database>";

    /// WebSocket URL template with auth
    pub const WSS_URL: &str = "wss://<username>:<password>@<host>:<port>/path";

    /// HTTPS URL template with auth
    pub const HTTPS_URL: &str = "https://<username>:<password>@<host>:<port>/path";
}

/// Helper struct for building test URLs
pub struct TestUrlBuilder {
    template: String,
}

impl TestUrlBuilder {
    /// Create a new builder from a template
    pub fn from_template(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }

    /// Replace username placeholder
    pub fn with_username(mut self, username: &str) -> Self {
        self.template = self.template.replace(placeholders::USERNAME, username);
        self
    }

    /// Replace password placeholder
    pub fn with_password(mut self, password: &str) -> Self {
        self.template = self.template.replace(placeholders::PASSWORD, password);
        self
    }

    /// Replace host placeholder
    pub fn with_host(mut self, host: &str) -> Self {
        self.template = self.template.replace(placeholders::HOST, host);
        self
    }

    /// Replace port placeholder
    pub fn with_port(mut self, port: &str) -> Self {
        self.template = self.template.replace(placeholders::PORT, port);
        self
    }

    /// Replace database placeholder
    pub fn with_database(mut self, database: &str) -> Self {
        self.template = self.template.replace(placeholders::DATABASE, database);
        self
    }

    /// Build the final URL
    pub fn build(self) -> String {
        self.template
    }
}

/// Common test URLs with safe placeholder values
pub mod test_urls {
    use super::{env_vars, templates, TestUrlBuilder};

    /// Get a test PostgreSQL URL
    pub fn postgres() -> String {
        TestUrlBuilder::from_template(templates::POSTGRES_URL)
            .with_username(
                std::env::var(env_vars::TEST_DB_USER)
                    .unwrap_or_else(|_| "user".to_string())
                    .as_str(),
            )
            .with_password(
                std::env::var(env_vars::TEST_DB_PASS)
                    .unwrap_or_else(|_| "pass".to_string())
                    .as_str(),
            )
            .with_host("localhost")
            .with_port("5432")
            .with_database("testdb")
            .build()
    }

    /// Get a test MySQL URL
    pub fn mysql() -> String {
        TestUrlBuilder::from_template(templates::MYSQL_URL)
            .with_username(
                std::env::var(env_vars::TEST_DB_USER)
                    .unwrap_or_else(|_| "user".to_string())
                    .as_str(),
            )
            .with_password(
                std::env::var(env_vars::TEST_DB_PASS)
                    .unwrap_or_else(|_| "pass".to_string())
                    .as_str(),
            )
            .with_host("localhost")
            .with_port("3306")
            .with_database("testdb")
            .build()
    }

    /// Get a test MongoDB URL
    pub fn mongodb() -> String {
        TestUrlBuilder::from_template(templates::MONGODB_URL)
            .with_username(
                std::env::var(env_vars::TEST_DB_USER)
                    .unwrap_or_else(|_| "user".to_string())
                    .as_str(),
            )
            .with_password(
                std::env::var(env_vars::TEST_DB_PASS)
                    .unwrap_or_else(|_| "pass".to_string())
                    .as_str(),
            )
            .with_host("localhost")
            .with_port("27017")
            .with_database("testdb")
            .build()
    }

    /// Get a test Redis URL
    pub fn redis() -> String {
        TestUrlBuilder::from_template(templates::REDIS_URL)
            .with_username(
                std::env::var(env_vars::TEST_DB_USER)
                    .unwrap_or_else(|_| "user".to_string())
                    .as_str(),
            )
            .with_password(
                std::env::var(env_vars::TEST_DB_PASS)
                    .unwrap_or_else(|_| "pass".to_string())
                    .as_str(),
            )
            .with_host("localhost")
            .with_port("6379")
            .with_database("0")
            .build()
    }

    /// Get a test WebSocket URL with auth
    pub fn websocket() -> String {
        TestUrlBuilder::from_template(templates::WSS_URL)
            .with_username(
                std::env::var(env_vars::TEST_WS_USER)
                    .unwrap_or_else(|_| "user".to_string())
                    .as_str(),
            )
            .with_password(
                std::env::var(env_vars::TEST_WS_PASS)
                    .unwrap_or_else(|_| "pass".to_string())
                    .as_str(),
            )
            .with_host("example.com")
            .with_port("443")
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_builder() {
        let url = TestUrlBuilder::from_template(templates::POSTGRES_URL)
            .with_username("myuser")
            .with_password("mypass")
            .with_host("myhost")
            .with_port("5432")
            .with_database("mydb")
            .build();

        assert_eq!(url, "postgresql://myuser:mypass@myhost:5432/mydb");
    }

    #[test]
    fn test_postgres_url() {
        std::env::set_var(env_vars::TEST_DB_USER, "u");
        std::env::set_var(env_vars::TEST_DB_PASS, "p");
        let url = test_urls::postgres();
        assert_eq!(url, "postgresql://u:p@localhost:5432/testdb");
        std::env::remove_var(env_vars::TEST_DB_USER);
        std::env::remove_var(env_vars::TEST_DB_PASS);
    }

    #[test]
    fn test_mysql_url() {
        std::env::set_var(env_vars::TEST_DB_USER, "u");
        std::env::set_var(env_vars::TEST_DB_PASS, "p");
        let url = test_urls::mysql();
        assert_eq!(url, "mysql://u:p@localhost:3306/testdb");
        std::env::remove_var(env_vars::TEST_DB_USER);
        std::env::remove_var(env_vars::TEST_DB_PASS);
    }

    #[test]
    fn test_mongodb_url() {
        std::env::set_var(env_vars::TEST_DB_USER, "u");
        std::env::set_var(env_vars::TEST_DB_PASS, "p");
        let url = test_urls::mongodb();
        assert_eq!(url, "mongodb://u:p@localhost:27017/testdb");
        std::env::remove_var(env_vars::TEST_DB_USER);
        std::env::remove_var(env_vars::TEST_DB_PASS);
    }

    #[test]
    fn test_redis_url() {
        std::env::set_var(env_vars::TEST_DB_USER, "u");
        std::env::set_var(env_vars::TEST_DB_PASS, "p");
        let url = test_urls::redis();
        assert_eq!(url, "redis://u:p@localhost:6379/0");
        std::env::remove_var(env_vars::TEST_DB_USER);
        std::env::remove_var(env_vars::TEST_DB_PASS);
    }

    #[test]
    fn test_websocket_url() {
        std::env::set_var(env_vars::TEST_WS_USER, "u");
        std::env::set_var(env_vars::TEST_WS_PASS, "p");
        let url = test_urls::websocket();
        assert_eq!(url, "wss://u:p@example.com:443/path");
        std::env::remove_var(env_vars::TEST_WS_USER);
        std::env::remove_var(env_vars::TEST_WS_PASS);
    }
}
