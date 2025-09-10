//! Input validation utilities

use regex::Regex;

/// Validate project name
pub fn validate_project_name(name: &str) -> Result<(), String> {
    // Check if empty
    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    // Check length
    if name.len() > 64 {
        return Err("Project name must be 64 characters or less".to_string());
    }

    // Check if it starts with a letter or underscore
    if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
        return Err("Project name must start with a letter or underscore".to_string());
    }

    // Check for valid characters (alphanumeric, underscore, hyphen)
    let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
    if !re.is_match(name) {
        return Err(
            "Project name can only contain letters, numbers, underscores, and hyphens".to_string(),
        );
    }

    // Check for reserved names
    let reserved = [
        "test",
        "main",
        "src",
        "target",
        "build",
        "dist",
        "node_modules",
    ];
    if reserved.contains(&name.to_lowercase().as_str()) {
        return Err(format!("'{}' is a reserved name", name));
    }

    Ok(())
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<(), String> {
    // Check for double dots which are invalid
    if email.contains("..") {
        return Err("Invalid email address format".to_string());
    }

    // Updated regex to support apostrophes but exclude double dots
    let re = Regex::new(r"^[a-zA-Z0-9._%+'+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if re.is_match(email) {
        Ok(())
    } else {
        Err("Invalid email address format".to_string())
    }
}

/// Validate URL
#[allow(dead_code)]
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        // Check if there's actual content after the protocol
        let after_protocol = if url.starts_with("https://") {
            &url[8..]
        } else {
            &url[7..]
        };

        if after_protocol.is_empty() {
            Err("URL must contain a host after the protocol".to_string())
        } else {
            Ok(())
        }
    } else {
        Err("URL must start with http:// or https://".to_string())
    }
}

/// Validate port number
#[allow(dead_code)]
pub fn validate_port(port: &str) -> Result<(), String> {
    match port.parse::<u16>() {
        Ok(p) if p > 0 => Ok(()),
        _ => Err("Invalid port number (must be 1-65535)".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name_when_valid_should_return_ok() {
        // Happy path tests
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("MyProject123").is_ok());
        assert!(validate_project_name("_private").is_ok());
        assert!(validate_project_name("a").is_ok()); // Single character
        assert!(validate_project_name("A").is_ok()); // Single uppercase letter
        assert!(validate_project_name("validName123_test-project").is_ok()); // Complex valid name
    }

    #[test]
    fn test_validate_project_name_when_empty_should_return_err() {
        let result = validate_project_name("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Project name cannot be empty");
    }

    #[test]
    fn test_validate_project_name_when_whitespace_only_should_return_err() {
        let result = validate_project_name("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Project name cannot be empty");
    }

    #[test]
    fn test_validate_project_name_when_too_long_should_return_err() {
        let long_name = "a".repeat(65);
        let result = validate_project_name(&long_name);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Project name must be 64 characters or less"
        );
    }

    #[test]
    fn test_validate_project_name_when_exactly_64_chars_should_return_ok() {
        let name = "a".repeat(64);
        assert!(validate_project_name(&name).is_ok());
    }

    #[test]
    fn test_validate_project_name_when_starts_with_number_should_return_err() {
        let result = validate_project_name("123project");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Project name must start with a letter or underscore"
        );
    }

    #[test]
    fn test_validate_project_name_when_starts_with_hyphen_should_return_err() {
        let result = validate_project_name("-project");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Project name must start with a letter or underscore"
        );
    }

    #[test]
    fn test_validate_project_name_when_contains_spaces_should_return_err() {
        let result = validate_project_name("my project");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Project name can only contain letters, numbers, underscores, and hyphens"
        );
    }

    #[test]
    fn test_validate_project_name_when_contains_special_chars_should_return_err() {
        let result = validate_project_name("my@project");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Project name can only contain letters, numbers, underscores, and hyphens"
        );
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_test_should_return_err() {
        let result = validate_project_name("test");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'test' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_main_should_return_err() {
        let result = validate_project_name("main");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'main' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_src_should_return_err() {
        let result = validate_project_name("src");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'src' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_target_should_return_err() {
        let result = validate_project_name("target");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'target' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_build_should_return_err() {
        let result = validate_project_name("build");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'build' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_dist_should_return_err() {
        let result = validate_project_name("dist");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'dist' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_node_modules_should_return_err() {
        let result = validate_project_name("node_modules");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'node_modules' is a reserved name");
    }

    #[test]
    fn test_validate_project_name_when_reserved_name_case_insensitive_should_return_err() {
        let result = validate_project_name("TEST");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'TEST' is a reserved name");

        let result = validate_project_name("Main");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'Main' is a reserved name");
    }

    #[test]
    fn test_validate_email_when_valid_should_return_ok() {
        // Happy path tests
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name@example.co.uk").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
        assert!(validate_email("user_name@example.com").is_ok());
        assert!(validate_email("user%test@example.com").is_ok());
        assert!(validate_email("user'test@example.com").is_ok()); // Apostrophe support
        assert!(validate_email("user-name@example-domain.com").is_ok());
        assert!(validate_email("123@example.com").is_ok());
        assert!(validate_email("a@b.co").is_ok()); // Minimal valid email
    }

    #[test]
    fn test_validate_email_when_contains_double_dots_should_return_err() {
        let result = validate_email("user..name@example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_invalid_format_should_return_err() {
        let result = validate_email("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_missing_at_symbol_should_return_err() {
        let result = validate_email("userexample.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_starts_with_at_should_return_err() {
        let result = validate_email("@example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_ends_with_at_should_return_err() {
        let result = validate_email("user@");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_no_domain_extension_should_return_err() {
        let result = validate_email("user@example");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_empty_should_return_err() {
        let result = validate_email("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_multiple_at_symbols_should_return_err() {
        let result = validate_email("user@domain@example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_email_when_short_domain_extension_should_return_err() {
        let result = validate_email("user@example.c");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid email address format");
    }

    #[test]
    fn test_validate_url_when_http_should_return_ok() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("http://www.example.com").is_ok());
        assert!(validate_url("http://api.example.com:8080/path").is_ok());
        assert!(validate_url("http://localhost").is_ok());
        assert!(validate_url("http://127.0.0.1").is_ok());
        assert!(validate_url("http://example.com/path/to/resource?query=value").is_ok());
    }

    #[test]
    fn test_validate_url_when_https_should_return_ok() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://www.example.com").is_ok());
        assert!(validate_url("https://api.example.com:8080/path").is_ok());
        assert!(validate_url("https://secure.example.com/login").is_ok());
    }

    #[test]
    fn test_validate_url_when_no_protocol_should_return_err() {
        let result = validate_url("example.com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "URL must start with http:// or https://"
        );
    }

    #[test]
    fn test_validate_url_when_ftp_protocol_should_return_err() {
        let result = validate_url("ftp://example.com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "URL must start with http:// or https://"
        );
    }

    #[test]
    fn test_validate_url_when_empty_should_return_err() {
        let result = validate_url("");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "URL must start with http:// or https://"
        );
    }

    #[test]
    fn test_validate_url_when_invalid_protocol_should_return_err() {
        let result = validate_url("htp://example.com");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "URL must start with http:// or https://"
        );
    }

    #[test]
    fn test_validate_url_when_file_protocol_should_return_err() {
        let result = validate_url("file://path/to/file");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "URL must start with http:// or https://"
        );
    }

    #[test]
    fn test_validate_port_when_valid_should_return_ok() {
        // Happy path tests
        assert!(validate_port("1").is_ok()); // Minimum valid port
        assert!(validate_port("80").is_ok()); // Standard HTTP port
        assert!(validate_port("443").is_ok()); // Standard HTTPS port
        assert!(validate_port("8080").is_ok()); // Common development port
        assert!(validate_port("65535").is_ok()); // Maximum valid port
        assert!(validate_port("3000").is_ok()); // Common Node.js port
        assert!(validate_port("5432").is_ok()); // PostgreSQL default port
    }

    #[test]
    fn test_validate_port_when_zero_should_return_err() {
        let result = validate_port("0");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_too_large_should_return_err() {
        let result = validate_port("65536");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_way_too_large_should_return_err() {
        let result = validate_port("99999");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_non_numeric_should_return_err() {
        let result = validate_port("abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_negative_should_return_err() {
        let result = validate_port("-1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_empty_should_return_err() {
        let result = validate_port("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_decimal_should_return_err() {
        let result = validate_port("80.5");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_alphanumeric_should_return_err() {
        let result = validate_port("80a");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }

    #[test]
    fn test_validate_port_when_whitespace_should_return_err() {
        let result = validate_port(" 80 ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid port number (must be 1-65535)");
    }
}
