//! Create RIGLR App library
//!
//! This library provides functionality for scaffolding RIGLR-powered blockchain AI agents.

pub mod config;
pub mod dependencies;
pub mod generator;
pub mod templates;
pub mod validation;

pub use config::{Chain, Feature, ProjectConfig, ServerFramework, Template, TemplateInfo};
pub use generator::ProjectGenerator;
pub use templates::TemplateManager;
pub use validation::{validate_email, validate_port, validate_project_name, validate_url};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_re_exports_are_accessible() {
        // Test that re-exported config types are accessible
        let template = Template::Custom;
        assert_eq!(template, Template::Custom);

        let server_framework = ServerFramework::Axum;
        assert_eq!(server_framework, ServerFramework::Axum);

        // Test TemplateInfo can be created
        let template_info = TemplateInfo::from_template(&Template::Custom);
        assert_eq!(template_info.name, "custom");

        // Test ProjectConfig can be created
        let config = ProjectConfig {
            name: "test_project".to_string(),
            template: Template::Custom,
            chains: vec![Chain::Solana],
            server_framework: Some(ServerFramework::Axum),
            features: vec![Feature::Logging],
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            description: "Test project".to_string(),
            include_examples: true,
            include_tests: true,
            include_docs: true,
        };
        assert_eq!(config.name, "test_project");
    }

    #[test]
    fn test_generator_re_export_is_accessible() {
        // Test that ProjectGenerator can be created
        let config = ProjectConfig {
            name: "test_project".to_string(),
            template: Template::Custom,
            chains: vec![Chain::Solana],
            server_framework: None,
            features: vec![],
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            description: "Test project".to_string(),
            include_examples: false,
            include_tests: false,
            include_docs: false,
        };

        let generator = ProjectGenerator::new(config);
        // Just verify the generator was created successfully
        // We can't easily test the internal state without exposing it
        assert!(std::ptr::addr_of!(generator) as usize > 0);
    }

    #[test]
    fn test_templates_re_export_is_accessible() {
        // Test that TemplateManager can be created
        let manager = TemplateManager::default();

        // Test that we can call methods on the re-exported type
        let templates_result = manager.list_templates();
        assert!(templates_result.is_ok());

        let templates = templates_result.unwrap();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.name == "custom"));
    }

    #[test]
    fn test_validation_functions_are_accessible() {
        // Test validate_project_name re-export
        assert!(validate_project_name("valid_project").is_ok());
        assert!(validate_project_name("").is_err());

        // Test validate_email re-export
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());

        // Test validate_url re-export
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("invalid").is_err());

        // Test validate_port re-export
        assert!(validate_port("8080").is_ok());
        assert!(validate_port("0").is_err());
    }

    #[test]
    fn test_all_template_variants_are_accessible() {
        // Test all Template enum variants are accessible through re-export
        let templates = vec![
            Template::ApiServiceBackend,
            Template::DataAnalyticsBot,
            Template::EventDrivenTradingEngine,
            Template::MinimalApi,
            Template::Custom,
        ];

        // Verify all templates can be created and compared
        for template in templates {
            let template_info = TemplateInfo::from_template(&template);
            assert!(!template_info.name.is_empty());
            assert!(!template_info.description.is_empty());
        }
    }

    #[test]
    fn test_all_server_framework_variants_are_accessible() {
        // Test all ServerFramework enum variants are accessible through re-export
        let frameworks = vec![
            ServerFramework::Actix,
            ServerFramework::Axum,
            ServerFramework::Warp,
            ServerFramework::Rocket,
        ];

        // Verify all frameworks can be created and compared
        for framework in frameworks {
            let deps = framework.dependencies();
            assert!(!deps.is_empty());
        }
    }

    #[test]
    fn test_template_manager_get_template_info_functionality() {
        let manager = TemplateManager::default();

        // Test valid template name
        let result = manager.get_template_info("custom");
        assert!(result.is_ok());
        let template_info = result.unwrap();
        assert_eq!(template_info.name, "custom");

        // Test invalid template name
        let result = manager.get_template_info("invalid-template");
        assert!(result.is_err());
    }

    #[test]
    fn test_template_manager_get_template_content_functionality() {
        let manager = TemplateManager::default();

        // Test different template types
        let templates_to_test = vec![
            Template::ApiServiceBackend,
            Template::DataAnalyticsBot,
            Template::EventDrivenTradingEngine,
            Template::Custom,
        ];

        for template in templates_to_test {
            let result = manager.get_template_content(&template);
            assert!(result.is_ok());
            let content = result.unwrap();
            assert!(!content.main_rs.is_empty());
            assert!(!content.cargo_toml.is_empty());
            assert!(!content.env_example.is_empty());
        }
    }

    #[test]
    fn test_template_string_conversion() {
        // Test Template Display implementation through re-export
        assert_eq!(Template::ApiServiceBackend.to_string(), "api-service");
        assert_eq!(Template::DataAnalyticsBot.to_string(), "data-analytics");
        assert_eq!(
            Template::EventDrivenTradingEngine.to_string(),
            "event-driven"
        );
        assert_eq!(Template::MinimalApi.to_string(), "minimal-api");
        assert_eq!(Template::Custom.to_string(), "custom");
    }

    #[test]
    fn test_template_from_str_functionality() {
        // Test Template from_str method through re-export
        assert_eq!(Template::parse("custom").unwrap(), Template::Custom);
        assert_eq!(
            Template::parse("api-service").unwrap(),
            Template::ApiServiceBackend
        );
        assert_eq!(Template::parse("api").unwrap(), Template::ApiServiceBackend);
        assert_eq!(
            Template::parse("analytics").unwrap(),
            Template::DataAnalyticsBot
        );
        assert_eq!(
            Template::parse("data-analytics").unwrap(),
            Template::DataAnalyticsBot
        );
        assert_eq!(
            Template::parse("event-driven").unwrap(),
            Template::EventDrivenTradingEngine
        );
        assert_eq!(
            Template::parse("trading-engine").unwrap(),
            Template::EventDrivenTradingEngine
        );

        // Test error case
        assert!(Template::parse("invalid-template").is_err());
    }

    #[test]
    fn test_template_description_functionality() {
        // Test Template description method through re-export
        assert_eq!(
            Template::Custom.description(),
            "Minimal template with basic structure"
        );
        assert_eq!(
            Template::ApiServiceBackend.description(),
            "RESTful API service with blockchain integration and AI agents"
        );
        assert_eq!(
            Template::DataAnalyticsBot.description(),
            "Real-time blockchain data analysis and insights generation"
        );
        assert_eq!(
            Template::EventDrivenTradingEngine.description(),
            "Event-driven automated trading with complex strategies"
        );
        assert_eq!(
            Template::MinimalApi.description(),
            "A barebones API service with a health check and a single agent endpoint"
        );
        assert_eq!(
            Template::Custom.description(),
            "Minimal template with basic structure"
        );
    }

    #[test]
    fn test_template_default_features_functionality() {
        // Test Template default_features method through re-export
        let api_features = Template::ApiServiceBackend.default_features();
        assert!(api_features.contains(&Feature::WebTools));
        assert!(api_features.contains(&Feature::Auth));
        assert!(api_features.contains(&Feature::Redis));
        assert!(api_features.contains(&Feature::Database));
        assert!(api_features.contains(&Feature::ApiDocs));
        assert!(api_features.contains(&Feature::Logging));

        let analytics_features = Template::DataAnalyticsBot.default_features();
        assert!(analytics_features.contains(&Feature::WebTools));
        assert!(analytics_features.contains(&Feature::GraphMemory));
        assert!(analytics_features.contains(&Feature::Streaming));
        assert!(analytics_features.contains(&Feature::Database));
        assert!(analytics_features.contains(&Feature::Redis));
        assert!(analytics_features.contains(&Feature::Logging));

        let event_features = Template::EventDrivenTradingEngine.default_features();
        assert!(event_features.contains(&Feature::WebTools));
        assert!(event_features.contains(&Feature::Streaming));
        assert!(event_features.contains(&Feature::Redis));
        assert!(event_features.contains(&Feature::Database));
        assert!(event_features.contains(&Feature::Logging));

        // Test MinimalApi and Custom templates get basic logging feature
        let minimal_features = Template::MinimalApi.default_features();
        assert_eq!(minimal_features, vec![Feature::Logging]);

        let custom_features = Template::Custom.default_features();
        assert_eq!(custom_features, vec![Feature::Logging]);
    }

    #[test]
    fn test_edge_cases_and_boundary_conditions() {
        // Test edge cases for validation functions

        // Project name edge cases
        assert!(validate_project_name("a").is_ok()); // Minimum valid length
        assert!(validate_project_name(&"a".repeat(64)).is_ok()); // Maximum valid length
        assert!(validate_project_name(&"a".repeat(65)).is_err()); // Over maximum length

        // Email edge cases
        assert!(validate_email("a@b.co").is_ok()); // Minimum valid email
        assert!(validate_email("user..name@example.com").is_err()); // Double dots
        assert!(validate_email("user'name@example.com").is_ok()); // Apostrophe should be valid

        // URL edge cases
        assert!(validate_url("http://").is_err()); // Just protocol
        assert!(validate_url("https://a.b").is_ok()); // Minimum valid URL

        // Port edge cases
        assert!(validate_port("1").is_ok()); // Minimum valid port
        assert!(validate_port("65535").is_ok()); // Maximum valid port
        assert!(validate_port("65536").is_err()); // Over maximum
    }
}
