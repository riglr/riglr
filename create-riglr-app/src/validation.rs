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
        return Err("Project name can only contain letters, numbers, underscores, and hyphens".to_string());
    }
    
    // Check for reserved names
    let reserved = vec!["test", "main", "src", "target", "build", "dist", "node_modules"];
    if reserved.contains(&name.to_lowercase().as_str()) {
        return Err(format!("'{}' is a reserved name", name));
    }
    
    Ok(())
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<(), String> {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if re.is_match(email) {
        Ok(())
    } else {
        Err("Invalid email address format".to_string())
    }
}

/// Validate URL
pub fn validate_url(url: &str) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err("URL must start with http:// or https://".to_string())
    }
}

/// Validate port number
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
    fn test_validate_project_name() {
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("MyProject123").is_ok());
        assert!(validate_project_name("_private").is_ok());
        
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("123project").is_err());
        assert!(validate_project_name("my project").is_err());
        assert!(validate_project_name("test").is_err());
        assert!(validate_project_name("a".repeat(65).as_str()).is_err());
    }
    
    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name@example.co.uk").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
        
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@example").is_err());
    }
    
    #[test]
    fn test_validate_url() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://api.example.com:8080/path").is_ok());
        
        assert!(validate_url("example.com").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("").is_err());
    }
    
    #[test]
    fn test_validate_port() {
        assert!(validate_port("80").is_ok());
        assert!(validate_port("8080").is_ok());
        assert!(validate_port("65535").is_ok());
        
        assert!(validate_port("0").is_err());
        assert!(validate_port("65536").is_err());
        assert!(validate_port("abc").is_err());
        assert!(validate_port("-1").is_err());
    }
}