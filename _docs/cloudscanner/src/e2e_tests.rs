// End-to-end integration tests for cloudscanner

#[cfg(test)]
mod e2e_tests {
    use super::*;
    use crate::engine::DiscoveryEngine;
    use crate::config::load_config;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_full_discovery_workflow() {
        // Test complete workflow from config loading to resource discovery
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let config_content = r#"
[[provider]]
id = "test-provider-1"
type = "builtin-aws"

[[provider]]
id = "test-provider-2"
type = "builtin-aws"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.providers.len(), 2);
        
        // Initialize engine
        let mut engine = DiscoveryEngine::new();
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Run discovery
        let resources = engine.run().await;
        
        // Verify results
        assert!(!resources.is_empty());
        assert_eq!(resources.len(), 4); // 2 providers × 2 resources each
        
        // Verify resource structure
        for resource in &resources {
            assert!(!resource.asset_type.is_empty());
            assert!(!resource.id.is_empty());
        }
    }

    #[tokio::test]
    async fn test_error_recovery_workflow() {
        // Test that system recovers gracefully from provider failures
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("error_config.toml");
        
        let config_content = r#"
[[provider]]
id = "working-provider"
type = "builtin-aws"

[[provider]]
id = "failing-provider"
type = "plugin"
plugin_path = "nonexistent.so"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.providers.len(), 2);
        
        // Initialize engine
        let mut engine = DiscoveryEngine::new();
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Run discovery - should continue despite one provider failing
        let resources = engine.run().await;
        
        // Should still get resources from the working provider
        assert!(!resources.is_empty());
        assert_eq!(resources.len(), 2); // Only from working provider
    }

    #[tokio::test]
    async fn test_concurrent_discovery_performance() {
        // Test that concurrent discovery improves performance
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("perf_config.toml");
        
        // Create config with multiple providers
        let mut config_content = String::new();
        for i in 0..10 {
            config_content.push_str(&format!(
                r#"
[[provider]]
id = "perf-provider-{}"
type = "builtin-aws"
"#, i
            ));
        }
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        assert_eq!(config.providers.len(), 10);
        
        // Initialize engine
        let mut engine = DiscoveryEngine::new();
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Measure discovery time
        let start = std::time::Instant::now();
        let resources = engine.run().await;
        let duration = start.elapsed();
        
        // Verify results
        assert_eq!(resources.len(), 20); // 10 providers × 2 resources each
        
        // Should complete in reasonable time (concurrent execution)
        assert!(duration < std::time::Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_memory_usage_under_load() {
        // Test memory usage with large number of resources
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("memory_config.toml");
        
        let config_content = r#"
[[provider]]
id = "memory-test-provider"
type = "builtin-aws"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        
        // Initialize engine
        let mut engine = DiscoveryEngine::new();
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Run discovery multiple times to test memory management
        for _ in 0..100 {
            let resources = engine.run().await;
            assert!(!resources.is_empty());
            
            // Resources should be dropped properly between runs
            drop(resources);
        }
    }

    #[tokio::test]
    async fn test_configuration_validation_workflow() {
        // Test that invalid configurations are caught early
        let temp_dir = TempDir::new().unwrap();
        
        // Test 1: Invalid provider type
        let config_path1 = temp_dir.path().join("invalid_type.toml");
        let config1 = r#"
[[provider]]
id = "test"
type = ""
"#;
        fs::write(&config_path1, config1).unwrap();
        
        let result1 = load_config(config_path1.to_str().unwrap());
        assert!(result1.is_err());
        
        // Test 2: Dangerous plugin path
        let config_path2 = temp_dir.path().join("dangerous_path.toml");
        let config2 = r#"
[[provider]]
id = "test"
type = "plugin"
plugin_path = "../../../etc/passwd"
"#;
        fs::write(&config_path2, config2).unwrap();
        
        let result2 = load_config(config_path2.to_str().unwrap());
        assert!(result2.is_err());
        
        // Test 3: Oversized config
        let config_path3 = temp_dir.path().join("oversized.toml");
        let config3 = format!(
            r#"
[[provider]]
id = "test"
type = "aws"
config = {{ "data": "{}" }}
"#,
            "x".repeat(11 * 1024 * 1024) // 11MB
        );
        fs::write(&config_path3, config3).unwrap();
        
        let result3 = load_config(config_path3.to_str().unwrap());
        assert!(result3.is_err());
    }

    #[tokio::test]
    async fn test_plugin_lifecycle_workflow() {
        // Test complete plugin lifecycle from loading to cleanup
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("plugin_lifecycle.toml");
        
        let config_content = r#"
[[provider]]
id = "lifecycle-test"
type = "builtin-aws"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        
        // Test engine lifecycle
        {
            let mut engine = DiscoveryEngine::new();
            engine.load_providers_from_config(&config).await.unwrap();
            
            // Run discovery
            let resources = engine.run().await;
            assert!(!resources.is_empty());
            
            // Engine should cleanup properly when dropped
        } // Engine goes out of scope here
        
        // Test that we can create a new engine without conflicts
        let mut engine2 = DiscoveryEngine::new();
        engine2.load_providers_from_config(&config).await.unwrap();
        let resources2 = engine2.run().await;
        assert!(!resources2.is_empty());
    }

    #[tokio::test]
    async fn test_resource_deduplication_workflow() {
        // Test that resources are properly handled when multiple providers return duplicates
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("dedup_config.toml");
        
        let config_content = r#"
[[provider]]
id = "duplicate-test-1"
type = "builtin-aws"

[[provider]]
id = "duplicate-test-2"
type = "builtin-aws"
"#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // Load configuration
        let config = load_config(config_path.to_str().unwrap()).unwrap();
        
        // Initialize engine
        let mut engine = DiscoveryEngine::new();
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Run discovery
        let resources = engine.run().await;
        
        // Should get resources from both providers (deduplication would be handled elsewhere)
        assert_eq!(resources.len(), 4);
        
        // Verify all resources have required fields
        for resource in &resources {
            assert!(!resource.asset_type.is_empty());
            assert!(!resource.id.is_empty());
            assert_eq!(resource.asset_type, "test-resource");
        }
    }

    #[test]
    fn test_error_message_sanitization_workflow() {
        // Test that all error paths properly sanitize sensitive data
        use crate::error::CloudScannerError;
        
        // Test various error types with sensitive data
        let errors = vec![
            CloudScannerError::provider("aws", "AKIA1234567890123456"),
            CloudScannerError::authentication("ec2", "password=secret123"),
            CloudScannerError::plugin("test", "token=abc123def456"),
            CloudScannerError::config("aws_secret_access_key=verysecret"),
        ];
        
        for error in errors {
            let sanitized = error.sanitize_for_logging();
            
            // Should not contain actual sensitive data
            assert!(!sanitized.contains("AKIA"));
            assert!(!sanitized.contains("secret123"));
            assert!(!sanitized.contains("abc123def456"));
            assert!(!sanitized.contains("verysecret"));
            
            // Should contain redaction indicator
            assert!(sanitized.contains("[REDACTED]"));
        }
    }
}