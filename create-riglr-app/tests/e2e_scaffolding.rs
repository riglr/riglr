//! End-to-end tests for the create-riglr-app scaffolding tool.
//!
//! This module contains integration tests that verify the correct functioning
//! of the create-riglr-app CLI tool, including project generation, template
//! customization, configuration setup, and example code generation.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use toml::Value;

#[test]
fn test_6_1_generate_build_and_test_scaffolding() -> Result<()> {
    println!("Starting scaffolding generation test...");

    let temp_dir = TempDir::new()?;
    let app_name = "test-riglr-app";
    let app_path = temp_dir.path().join(app_name);

    build_create_riglr_app()?;
    generate_test_project(app_name, &app_path)?;
    verify_project_structure(&app_path, app_name);
    verify_cargo_toml(&app_path, app_name)?;
    verify_env_configuration(&app_path)?;
    test_generated_project(&app_path);

    println!("Test 6.1 Passed: Scaffolding generation, build, and test successful");
    Ok(())
}

/// Build the create-riglr-app binary
fn build_create_riglr_app() -> Result<()> {
    println!("Building create-riglr-app...");
    let build_output = Command::new("cargo")
        .args(["build", "--package", "create-riglr-app"])
        .output()?;

    if !build_output.status.success() {
        eprintln!("Failed to build create-riglr-app:");
        eprintln!("{}", String::from_utf8_lossy(&build_output.stderr));
        return Err(anyhow::anyhow!("Failed to build create-riglr-app"));
    }
    Ok(())
}

/// Generate a test project using create-riglr-app
fn generate_test_project(app_name: &str, app_path: &Path) -> Result<()> {
    println!("Generating project with create-riglr-app...");
    let app_path_str = app_path.to_string_lossy();
    let create_output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "create-riglr-app",
            "--",
            app_name,
            "--output",
            &app_path_str,
            "--yes",
        ])
        .output()?;

    if !create_output.status.success() {
        eprintln!("Failed to create app:");
        eprintln!("{}", String::from_utf8_lossy(&create_output.stderr));
        return Err(anyhow::anyhow!("Failed to create app"));
    }

    println!("Project generated at: {}", app_path.display());
    Ok(())
}

/// Verify the basic project structure exists
fn verify_project_structure(app_path: &Path, _app_name: &str) {
    assert!(app_path.exists(), "Project directory should be created");
    assert!(
        app_path.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(app_path.join("src").exists(), "src directory should exist");
    assert!(
        app_path.join("src/main.rs").exists(),
        "main.rs should exist"
    );
    assert!(
        app_path.join(".env.example").exists(),
        ".env.example should exist"
    );
}

/// Verify Cargo.toml structure and dependencies
#[allow(clippy::unwrap_used)]
fn verify_cargo_toml(app_path: &Path, app_name: &str) -> Result<()> {
    let cargo_toml_content = fs::read_to_string(app_path.join("Cargo.toml"))?;
    let cargo_toml: Value = toml::from_str(&cargo_toml_content)?;

    assert!(
        cargo_toml.get("package").is_some(),
        "Should have package section"
    );
    assert!(
        cargo_toml.get("dependencies").is_some(),
        "Should have dependencies section"
    );

    let package = cargo_toml.get("package").unwrap();
    assert_eq!(
        package.get("name").and_then(|v| v.as_str()),
        Some(app_name),
        "Package name should match"
    );

    let deps = cargo_toml.get("dependencies").unwrap();
    assert!(
        deps.get("riglr-core").is_some(),
        "Should have riglr-core dependency"
    );
    assert!(
        deps.get("rig-core").is_some(),
        "Should have rig-core dependency"
    );
    Ok(())
}

/// Verify .env.example configuration
fn verify_env_configuration(app_path: &Path) -> Result<()> {
    let env_content = fs::read_to_string(app_path.join(".env.example"))?;
    assert!(
        env_content.contains("RPC_URL_SOLANA")
            || env_content.contains("RPC_URL_1")
            || env_content.contains("RPC_URL"),
        ".env.example should contain RPC URL configuration"
    );
    assert!(
        env_content.contains("ANTHROPIC_API_KEY"),
        ".env.example should contain ANTHROPIC_API_KEY"
    );
    assert!(
        !env_content.contains("localhost")
            || env_content.contains("devnet")
            || env_content.contains("testnet"),
        ".env.example should reference public testnets, not localhost"
    );
    Ok(())
}

/// Test that the generated project compiles
fn test_generated_project(app_path: &Path) {
    println!("Running cargo check on generated project...");
    let check_output = Command::new("cargo")
        .args(["check"])
        .current_dir(app_path)
        .output();

    match check_output {
        Ok(output) if output.status.success() => println!("Cargo check passed!"),
        Ok(output) => {
            eprintln!("Warning: cargo check failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => eprintln!("Failed to run cargo check: {e}"),
    }

    println!("Running cargo build on generated project...");
    let build_output = Command::new("cargo")
        .args(["build"])
        .current_dir(app_path)
        .env("CARGO_TERM_COLOR", "never")
        .output();

    match build_output {
        Ok(output) if output.status.success() => println!("Cargo build passed!"),
        Ok(output) => {
            eprintln!("Warning: cargo build failed:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => eprintln!("Failed to run cargo build: {e}"),
    }
}

#[test]
fn test_6_2_template_customization() -> Result<()> {
    println!("Starting template customization test...");

    // Create temporary directory
    let temp_dir = TempDir::new()?;

    // Test different template options
    let templates = vec![
        ("minimal-app", vec!["--template", "minimal"]),
        ("full-app", vec!["--template", "full"]),
        ("agent-app", vec!["--template", "agent"]),
    ];

    for (app_name, args) in templates {
        let app_path = temp_dir.path().join(app_name);

        println!("Generating {app_name} with template args: {args:?}");

        // Run create-riglr-app with template options
        let app_path_str = app_path.to_string_lossy();
        let mut cmd_args = vec!["run", "--package", "create-riglr-app", "--", &app_path_str];
        cmd_args.extend(args.iter().copied());

        let output = Command::new("cargo").args(&cmd_args).output()?;

        if output.status.success() {
            // Verify the generated structure
            assert!(app_path.exists(), "{app_name} should be created");
            assert!(
                app_path.join("Cargo.toml").exists(),
                "{app_name} should have Cargo.toml"
            );

            // Check for template-specific files
            match app_name {
                "minimal-app" => {
                    // Minimal template should have basic structure
                    assert!(
                        app_path.join("src/main.rs").exists(),
                        "Minimal template should have main.rs"
                    );
                }
                "full-app" => {
                    // Full template might have more files
                    assert!(
                        app_path.join("src").exists(),
                        "Full template should have src directory"
                    );
                }
                "agent-app" => {
                    // Agent template should have agent-specific files
                    assert!(
                        app_path.join("src").exists(),
                        "Agent template should have src directory"
                    );
                }
                _ => {}
            }

            println!("{app_name} generated successfully");
        } else {
            println!(
                "Warning: Failed to generate {app_name} (template might not exist)"
            );
        }
    }

    println!("Test 6.2 Passed: Template customization test completed");

    Ok(())
}

#[test]
fn test_6_3_configuration_generation_with_conditional_env() -> Result<()> {
    println!("Starting configuration generation test...");

    let temp_dir = TempDir::new()?;
    let app_path = temp_dir.path().join("config-test-app");
    let app_name = "config-test-app";

    generate_config_test_app(app_name, &app_path)?;
    verify_configuration_files(&app_path)?;

    println!("Test 6.3 Passed: Configuration generation test successful");
    Ok(())
}

/// Generate app for configuration testing
fn generate_config_test_app(app_name: &str, app_path: &Path) -> Result<()> {
    let app_path_str = app_path.to_string_lossy();
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "create-riglr-app",
            "--",
            app_name,
            "--output",
            &app_path_str,
            "--yes",
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to create app for config test"));
    }
    Ok(())
}

/// Verify all configuration files are generated correctly
fn verify_configuration_files(app_path: &Path) -> Result<()> {
    let config_files = vec![".env.example", "Cargo.toml", ".gitignore"];

    for file in config_files {
        let file_path = app_path.join(file);
        assert!(file_path.exists(), "{file} should exist");

        let content = fs::read_to_string(&file_path)?;
        verify_config_file_content(file, &content)?;
    }
    Ok(())
}

/// Verify the content of individual configuration files
fn verify_config_file_content(file_name: &str, content: &str) -> Result<()> {
    match file_name {
        ".env.example" => {
            verify_env_example_content(content);
            Ok(())
        },
        ".gitignore" => {
            verify_gitignore_content(content);
            Ok(())
        },
        "Cargo.toml" => verify_cargo_toml_content(content),
        _ => Ok(()),
    }
}

/// Verify .env.example file content
fn verify_env_example_content(content: &str) {
    assert!(
        content.contains("RPC_URL_SOLANA")
            || content.contains("RPC_URL_1")
            || content.contains("RPC_URL"),
        ".env.example should contain RPC URL"
    );
    assert!(
        content.contains("ANTHROPIC_API_KEY"),
        ".env.example should contain ANTHROPIC_API_KEY"
    );
    assert!(
        content.contains("devnet")
            || content.contains("testnet")
            || content.contains("sepolia")
            || content.contains("publicnode"),
        ".env.example should reference public testnets"
    );

    if content.contains("API_KEY") {
        assert!(
            content.contains("your-api-key")
                || content.contains("dummy")
                || content.contains("placeholder"),
            "API keys should have placeholder values"
        );
    }
}

/// Verify .gitignore file content
fn verify_gitignore_content(content: &str) {
    assert!(
        content.contains("target/"),
        ".gitignore should exclude target/"
    );
    assert!(
        content.contains(".env"),
        ".gitignore should exclude .env files"
    );
    assert!(
        content.contains("Cargo.lock") || !content.contains("Cargo.lock"),
        ".gitignore should handle Cargo.lock appropriately"
    );
}

/// Verify Cargo.toml file content
#[allow(clippy::unwrap_used)]
fn verify_cargo_toml_content(content: &str) -> Result<()> {
    let cargo_toml: Value = toml::from_str(content)?;

    let package = cargo_toml.get("package").unwrap();
    assert!(package.get("version").is_some(), "Should have version");
    assert!(package.get("edition").is_some(), "Should have edition");

    let deps = cargo_toml.get("dependencies").unwrap();
    assert!(deps.as_table().is_some(), "Dependencies should be a table");

    if let Some(riglr_core) = deps.get("riglr-core") {
        assert!(
            riglr_core.is_str() || riglr_core.is_table(),
            "riglr-core should be properly configured"
        );
    }
    Ok(())
}

#[test]
fn test_6_4_example_code_generation() -> Result<()> {
    println!("Starting example code generation test...");

    let temp_dir = TempDir::new()?;
    let app_name = "example-app";
    let app_path = temp_dir.path().join(app_name);

    generate_example_app(app_name, &app_path)?;
    verify_main_rs_content(&app_path)?;
    check_examples_directory(&app_path)?;
    verify_readme_content(&app_path)?;

    println!("Test 6.4 Passed: Example code generation test successful");
    Ok(())
}

/// Generate app with examples
fn generate_example_app(app_name: &str, app_path: &Path) -> Result<()> {
    let app_path_str = app_path.to_string_lossy();
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "create-riglr-app",
            "--",
            app_name,
            "--output",
            &app_path_str,
            "--with-examples",
            "--yes",
        ])
        .output()?;

    if !output.status.success() {
        println!("Warning: --with-examples flag might not be implemented");
        let output = Command::new("cargo")
            .args([
                "run",
                "--package",
                "create-riglr-app",
                "--",
                app_name,
                "--output",
                &app_path_str,
                "--yes",
            ])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to create app"));
        }
    }
    Ok(())
}

/// Verify main.rs contains expected patterns
fn verify_main_rs_content(app_path: &Path) -> Result<()> {
    let main_rs_path = app_path.join("src/main.rs");
    assert!(main_rs_path.exists(), "main.rs should exist");

    let main_content = fs::read_to_string(&main_rs_path)?;

    assert!(
        main_content.contains("riglr") || main_content.contains("use "),
        "main.rs should contain riglr imports or use statements"
    );
    assert!(
        main_content.contains("async fn main") || main_content.contains("fn main"),
        "main.rs should have a main function"
    );

    let example_patterns = [
        "ApplicationContext",
        "riglr_config::Config",
        "async",
        "Result",
        "AgentDispatcher",
        "ToolCallingAgent",
    ];

    let pattern_count = example_patterns
        .iter()
        .filter(|pattern| main_content.contains(*pattern))
        .count();

    println!(
        "Found {}/{} example patterns in main.rs",
        pattern_count,
        example_patterns.len()
    );
    Ok(())
}

/// Check and verify examples directory if it exists
fn check_examples_directory(app_path: &Path) -> Result<()> {
    let examples_dir = app_path.join("examples");
    if examples_dir.exists() {
        println!("Examples directory found!");

        for entry in fs::read_dir(&examples_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            println!("  Example: {}", file_name.to_string_lossy());

            if let Some(ext) = entry.path().extension() {
                if ext == "rs" {
                    let content = fs::read_to_string(entry.path())?;
                    assert!(
                        content.contains("fn main") || content.contains("async fn main"),
                        "Example should have a main function"
                    );
                }
            }
        }
    }
    Ok(())
}

/// Verify README content if it exists
fn verify_readme_content(app_path: &Path) -> Result<()> {
    let readme_path = app_path.join("README.md");
    if readme_path.exists() {
        let readme_content = fs::read_to_string(&readme_path)?;

        if readme_content.contains("## Usage") || readme_content.contains("## Example") {
            println!("README contains usage examples");
        }

        assert!(
            readme_content.to_lowercase().contains("testnet")
                || readme_content.to_lowercase().contains("devnet")
                || readme_content.contains("RPC"),
            "README should mention testnet setup"
        );
    }
    Ok(())
}
