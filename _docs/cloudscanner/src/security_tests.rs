// Security-focused tests for cloudscanner

#[cfg(test)]
mod security_tests {
    use super::*;
    use crate::config::{load_config, validate_config};
    use crate::error::CloudScannerError;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_injection_prevention() {
        // Test that script injection attempts are blocked
        let malicious_config = r#"
[[provider]]
id = "test<script>alert('xss')</script>"
type = "aws"
plugin_path = "/etc/passwd"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(malicious_config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Security { .. } => (),
            _ => panic!("Expected security error for script injection"),
        }
    }

    #[test]
    fn test_path_traversal_prevention() {
        // Test that path traversal attempts are blocked
        let malicious_config = r#"
[[provider]]
id = "test"
type = "aws"
plugin_path = "../../../etc/passwd"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(malicious_config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Security { .. } => (),
            _ => panic!("Expected security error for path traversal"),
        }
    }

    #[test]
    fn test_credential_sanitization() {
        // Test that credentials in error messages are sanitized
        let error_with_creds = CloudScannerError::provider(
            "aws", 
            "Authentication failed with AKIA1234567890123456 and aws_secret_access_key=verysecret"
        );
        
        let sanitized = error_with_creds.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("AKIA"));
        assert!(!sanitized.contains("verysecret"));
    }

    #[test]
    fn test_plugin_size_limits() {
        // Test that oversized plugins are rejected
        let oversized_config = r#"
[[provider]]
id = "test"
type = "plugin"
plugin_path = "test.so"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(oversized_config.as_bytes()).unwrap();
        
        // Create a fake oversized plugin file
        let plugin_path = temp_file.path().with_extension("so");
        let oversized_data = vec![0u8; 15 * 1024 * 1024]; // 15MB
        fs::write(&plugin_path, oversized_data).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Plugin { .. } => (),
            _ => panic!("Expected plugin error for oversized file"),
        }
    }

    #[test]
    fn test_dangerous_file_extensions() {
        // Test that dangerous file extensions are blocked
        let dangerous_config = r#"
[[provider]]
id = "test"
type = "plugin"
plugin_path = "malware.exe"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(dangerous_config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Security { .. } => (),
            _ => panic!("Expected security error for dangerous file extension"),
        }
    }

    #[test]
    fn test_config_size_limits() {
        // Test that oversized configurations are rejected
        let oversized_config = r#"
[[provider]]
id = "test"
type = "aws"
config = { "#;
        
        // Create a config that exceeds the size limit
        let large_config = format!("{}\"data\": \"{}\"}}", 
            oversized_config, 
            "x".repeat(11 * 1024 * 1024) // 11MB of data
        );
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(large_config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Config { .. } => (),
            _ => panic!("Expected config error for oversized file"),
        }
    }

    #[test]
    fn test_null_byte_injection() {
        // Test that null bytes are blocked
        let malicious_config = r#"
[[provider]]
id = "test\0"
type = "aws\0"
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(malicious_config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Security { .. } => (),
            _ => panic!("Expected security error for null bytes"),
        }
    }

    #[test]
    fn test_base64_credential_detection() {
        // Test that base64-encoded credentials are detected
        let fake_credential = "QUtJQTEyMzQ1Njc4OTAxMjM0NTY3ODkw"; // base64 for "AKIA12345678901234567890"
        let error_with_base64 = CloudScannerError::provider("aws", fake_credential);
        
        let sanitized = error_with_base64.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED: Potential credential data]"));
        assert!(!sanitized.contains(fake_credential));
    }

    #[test]
    fn test_provider_count_limits() {
        // Test that excessive number of providers is blocked
        let mut config = String::from("[[provider]]\nid = \"test\"\ntype = \"aws\"\n");
        
        // Create config with too many providers
        for i in 1..=150 { // Exceeds MAX_PROVIDERS (100)
            config.push_str(&format!(
                "[[provider]]\nid = \"test{}\"\ntype = \"aws\"\n", i
            ));
        }
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config.as_bytes()).unwrap();
        
        let result = load_config(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        
        match result.unwrap_err() {
            CloudScannerError::Config { .. } => (),
            _ => panic!("Expected config error for too many providers"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_execution_isolation() {
        // Test that concurrent provider execution doesn't interfere
        use crate::engine::DiscoveryEngine;
        use crate::models::provider::{Provider, ProviderInfo, Resource};
        
        struct TestProvider {
            name: String,
            delay: std::time::Duration,
        }
        
        #[async_trait::async_trait]
        impl Provider for TestProvider {
            fn info(&self) -> ProviderInfo {
                ProviderInfo {
                    name: self.name.clone(),
                    version: "1.0.0".to_string(),
                    description: "Test provider".to_string(),
                    supported_resource_types: vec!["test".to_string()],
                }
            }

            async fn discover(&self) -> crate::error::Result<Vec<Resource>> {
                tokio::time::sleep(self.delay).await;
                Ok(vec![Resource::new("test".to_string(), "test-id".to_string())])
            }
        }
        
        let mut engine = DiscoveryEngine::new();
        
        // Add multiple providers with different delays
        for i in 0..5 {
            let provider = TestProvider {
                name: format!("test-{}", i),
                delay: std::time::Duration::from_millis(100 + i as u64 * 50),
            };
            engine.providers.push(std::sync::Arc::new(provider));
        }
        
        let start = std::time::Instant::now();
        let resources = engine.run().await;
        let duration = start.elapsed();
        
        // Should complete in roughly the time of the longest provider, not sum of all
        assert!(duration < std::time::Duration::from_millis(400)); // Less than sum of all delays
        assert_eq!(resources.len(), 5); // All providers should complete
    }
}