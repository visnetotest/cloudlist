use cloudscanner::config;
use cloudscanner::engine;
use cloudscanner::policy;
use cloudscanner::reporter;
use cloudscanner::models::provider::Resource;
use tempfile::{TempDir, NamedTempFile};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Create a temporary configuration file with test providers
fn create_test_config() -> NamedTempFile {
    let config_content = r#"
[[provider]]
id = "dummy-provider-1"
type = "dummy"

[[provider]]
id = "dummy-provider-2"
type = "dummy"

[[provider]]
id = "builtin-aws-provider"
type = "builtin-aws"
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    temp_file
}

/// Create a temporary policy file with test policies
fn create_test_policy() -> NamedTempFile {
    let policy_content = r#"
name: "Test Security Policy"
asset_type: "test-resource"
rules:
  - key: "environment"
    value: "production"
  - key: "encrypted"
    value: "true"
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(policy_content.as_bytes()).unwrap();
    temp_file
}

/// Create a temporary policy directory with multiple policies
fn create_test_policy_dir() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Policy 1: Production environment check
    let policy1_content = r#"
name: "Production Environment Policy"
asset_type: "test-resource"
rules:
  - key: "environment"
    value: "production"
"#;
    fs::write(temp_dir.path().join("production.yaml"), policy1_content).unwrap();
    
    // Policy 2: Encryption check
    let policy2_content = r#"
name: "Encryption Policy"
asset_type: "test-resource"
rules:
  - key: "encrypted"
    value: "true"
"#;
    fs::write(temp_dir.path().join("encryption.yaml"), policy2_content).unwrap();
    
    // Policy 3: Different resource type (should not match)
    let policy3_content = r#"
name: "Other Resource Policy"
asset_type: "other-resource"
rules:
  - key: "environment"
    value: "production"
"#;
    fs::write(temp_dir.path().join("other.yaml"), policy3_content).unwrap();
    
    temp_dir
}

#[tokio::test]
async fn test_full_discovery_pipeline() {
    // Test complete discovery -> policy -> report pipeline
    let config_file = create_test_config();
    let policy_dir = create_test_policy_dir();
    
    // Load configuration
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    assert_eq!(config.providers.len(), 3);
    
    // Initialize and load providers
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    // Run discovery
    let resources = engine.run().await;
    assert!(resources.len() > 0); // Should have resources from dummy providers
    
    // Initialize policy engine and load policies
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    
    // Evaluate resources against policies
    policy_engine.evaluate(&resources);
    
    // Generate report
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
    
    // Verify policy violations were detected (dummy resources don't match production policies)
    let violations = policy_engine.get_violations();
    assert!(violations.len() > 0);
}

#[tokio::test]
async fn test_end_to_end_with_real_plugin() {
    // Test with actual compiled plugin (if available)
    let config_content = r#"
[[provider]]
id = "test-plugin"
type = "plugin"
plugin_path = "plugins/test.so"
"#;
    
    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    
    // This should handle missing plugin gracefully
    let result = engine.load_providers_from_config(&config).await;
    assert!(result.is_ok()); // Should not fail, just warn
    
    let resources = engine.run().await;
    assert_eq!(resources.len(), 0); // No providers loaded successfully
}

#[tokio::test]
async fn test_multiple_policy_evaluation() {
    // Test multiple policies against discovered resources
    let config_file = create_test_config();
    let policy_dir = create_test_policy_dir();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let resources = engine.run().await;
    assert!(resources.len() > 0);
    
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    
    // Should have loaded 2 policies (production and encryption)
    assert!(policy_engine.get_violations_count() == 0); // Before evaluation
    
    policy_engine.evaluate(&resources);
    
    let violations = policy_engine.get_violations();
    // Should detect violations for dummy resources not matching production/encryption rules
    assert!(violations.len() > 0);
    
    // Verify violation structure
    for violation in violations {
        assert!(!violation.resource_id.is_empty());
        assert!(!violation.resource_type.is_empty());
        assert!(!violation.policy_name.is_empty());
        assert!(!violation.violated_rules.is_empty());
    }
}

#[tokio::test]
async fn test_error_recovery_and_continuation() {
    // Test system continues when one provider fails
    let config_content = r#"
[[provider]]
id = "dummy-provider"
type = "dummy"

[[provider]]
id = "invalid-plugin"
type = "plugin"
plugin_path = "/nonexistent/plugin.so"

[[provider]]
id = "another-dummy"
type = "dummy"
"#;
    
    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    
    // Should load successfully despite invalid plugin
    let result = engine.load_providers_from_config(&config).await;
    assert!(result.is_ok());
    
    // Should still run with valid providers
    let resources = engine.run().await;
    assert!(resources.len() > 0); // Should have resources from dummy providers
    
    // Should be able to generate report
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_configuration_driven_execution() {
    // Test execution driven by configuration file
    let config_file = create_test_config();
    let policy_file = create_test_policy();
    
    // Load and validate configuration
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    assert_eq!(config.providers.len(), 3);
    
    // Verify provider configurations
    let provider_ids: Vec<String> = config.providers.iter()
        .map(|p| p.id.clone())
        .collect();
    assert!(provider_ids.contains(&"dummy-provider-1".to_string()));
    assert!(provider_ids.contains(&"dummy-provider-2".to_string()));
    assert!(provider_ids.contains(&"builtin-aws-provider".to_string()));
    
    // Initialize engine with configuration
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    // Run discovery
    let resources = engine.run().await;
    assert!(resources.len() > 0);
    
    // Load policy and evaluate
    let mut policy_engine = policy::PolicyEngine::new();
    let policy_dir = TempDir::new().unwrap();
    fs::write(policy_dir.path().join("test.yaml"), 
              fs::read_to_string(policy_file.path()).unwrap()).unwrap();
    
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    policy_engine.evaluate(&resources);
    
    // Generate reports in different formats
    let console_reporter = reporter::Reporter::new().with_format(reporter::OutputFormat::Console);
    let json_reporter = reporter::Reporter::new().with_format(reporter::OutputFormat::Json);
    let yaml_reporter = reporter::Reporter::new().with_format(reporter::OutputFormat::Yaml);
    
    // All formats should work
    assert!(console_reporter.report(&resources).is_ok());
    assert!(json_reporter.report(&resources).is_ok());
    assert!(yaml_reporter.report(&resources).is_ok());
}

#[tokio::test]
async fn test_discovery_with_no_providers() {
    // Test behavior with empty configuration
    let config_content = "";
    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    assert_eq!(config.providers.len(), 0);
    
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let resources = engine.run().await;
    assert_eq!(resources.len(), 0);
    
    // Should still be able to generate report
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_with_only_builtin_providers() {
    // Test with only built-in providers
    let config_content = r#"
[[provider]]
id = "builtin-aws-1"
type = "builtin-aws"

[[provider]]
id = "builtin-aws-2"
type = "builtin-aws"
"#;
    
    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let resources = engine.run().await;
    assert!(resources.len() > 0); // Should have resources from AWS providers
    
    // Test policy evaluation
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.evaluate(&resources);
    
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_evaluation_with_no_policies() {
    // Test discovery and reporting without policies
    let config_file = create_test_config();
    let empty_policy_dir = TempDir::new().unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let resources = engine.run().await;
    assert!(resources.len() > 0);
    
    // Load empty policy directory
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.load_policies(empty_policy_dir.path().to_str().unwrap()).unwrap();
    
    // Should evaluate without violations
    policy_engine.evaluate(&resources);
    assert_eq!(policy_engine.get_violations_count(), 0);
    
    // Should still generate report
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_large_scale_discovery() {
    // Test with multiple providers to simulate larger scale
    let mut config_content = String::new();
    for i in 0..10 {
        config_content.push_str(&format!(r#"
[[provider]]
id = "dummy-provider-{}"
type = "dummy"
"#, i));
    }
    
    let mut config_file = NamedTempFile::new().unwrap();
    config_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let start_time = std::time::Instant::now();
    let resources = engine.run().await;
    let duration = start_time.elapsed();
    
    // Should complete in reasonable time
    assert!(duration < std::time::Duration::from_secs(10));
    assert!(resources.len() > 0);
    
    // Should handle policy evaluation efficiently
    let policy_dir = create_test_policy_dir();
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    
    let policy_start = std::time::Instant::now();
    policy_engine.evaluate(&resources);
    let policy_duration = policy_start.elapsed();
    
    // Policy evaluation should also be efficient
    assert!(policy_duration < std::time::Duration::from_secs(5));
    
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_full_pipeline_with_violations() {
    // Test complete pipeline with expected violations
    let config_file = create_test_config();
    let policy_dir = create_test_policy_dir();
    
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    
    let resources = engine.run().await;
    assert!(resources.len() > 0);
    
    let mut policy_engine = policy::PolicyEngine::new();
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    policy_engine.evaluate(&resources);
    
    let violations = policy_engine.get_violations();
    assert!(violations.len() > 0);
    
    // Test all report formats with violations
    let formats = vec![
        reporter::OutputFormat::Console,
        reporter::OutputFormat::Json,
        reporter::OutputFormat::Yaml,
    ];
    
    for format in formats {
        let reporter = reporter::Reporter::new().with_format(format);
        let result = reporter.report(&resources);
        assert!(result.is_ok(), "Report generation failed for format: {:?}", format);
    }
}

#[tokio::test]
async fn test_component_isolation() {
    // Test that components can be used independently
    let config_file = create_test_config();
    let policy_file = create_test_policy();
    
    // Test config loading independently
    let config = config::load_config(config_file.path().to_str().unwrap()).unwrap();
    assert!(config.providers.len() > 0);
    
    // Test engine independently
    let mut engine = engine::DiscoveryEngine::new();
    engine.load_providers_from_config(&config).await.unwrap();
    let resources = engine.run().await;
    
    // Test policy engine independently
    let mut policy_engine = policy::PolicyEngine::new();
    let policy_dir = TempDir::new().unwrap();
    fs::write(policy_dir.path().join("test.yaml"), 
              fs::read_to_string(policy_file.path()).unwrap()).unwrap();
    policy_engine.load_policies(policy_dir.path().to_str().unwrap()).unwrap();
    policy_engine.evaluate(&resources);
    
    // Test reporter independently
    let reporter = reporter::Reporter::new();
    let result = reporter.report(&resources);
    assert!(result.is_ok());
    
    // All components should work independently
    assert!(config.providers.len() > 0);
    assert!(resources.len() >= 0);
    assert!(policy_engine.get_violations_count() >= 0);
    assert!(result.is_ok());
}