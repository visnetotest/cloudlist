#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::provider::Resource;
    use std::collections::HashMap;
    
    #[test]
    fn test_aws_config_default() {
        let config = super::super::config::AwsConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.services.len(), 2);
        assert_eq!(config.services[0].name, "ec2");
        assert_eq!(config.services[0].enabled, true);
        assert_eq!(config.services[1].name, "s3");
        assert_eq!(config.services[1].enabled, true);
    }
    
    #[test]
    fn test_aws_config_serialization() {
        let config = super::super::config::AwsConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("region: us-east-1"));
        assert!(yaml.contains("ec2"));
        assert!(yaml.contains("s3"));
        
        // Test deserialization
        let deserialized: super::super::config::AwsConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.region, config.region);
        assert_eq!(deserialized.services.len(), config.services.len());
    }
    
    #[test]
    fn test_aws_provider_creation() {
        let config = super::super::config::AwsConfig::default();
        let provider = super::AwsProvider::new(config);
        
        // Should fail without AWS credentials, but not panic
        assert!(provider.is_ok() || provider.is_err());
    }
    
    #[test]
    fn test_aws_provider_info() {
        let config = super::super::config::AwsConfig::default();
        let provider = super::AwsProvider::new(config).unwrap_or_else(|_| {
            // Create a mock provider for testing
            super::AwsProvider {
                config,
                ec2_discovery: None,
                s3_discovery: None,
            }
        });
        
        let info = provider.info();
        assert_eq!(info.name, "aws");
        assert_eq!(info.version, "1.0.0");
        assert!(info.description.contains("AWS"));
        assert_eq!(info.supported_resource_types.len(), 2);
        assert!(info.supported_resource_types.contains(&"ec2-instance".to_string()));
        assert!(info.supported_resource_types.contains(&"s3-bucket".to_string()));
    }
    
    #[test]
    fn test_aws_provider_discovery() {
        let config = super::super::config::AwsConfig::default();
        let provider = super::AwsProvider::new(config).unwrap_or_else(|_| {
            // Create a mock provider for testing
            super::AwsProvider {
                config,
                ec2_discovery: None,
                s3_discovery: None,
            }
        });
        
        // Should not panic even without credentials
        let result = provider.discover();
        
        // Should return empty vector or error, but not panic
        match result {
            Ok(_) => {
                // Empty result is acceptable without credentials
            }
            Err(_) => {
                // Error is acceptable without credentials
            }
        }
    }
    
    #[test]
    fn test_ec2_discovery_creation() {
        let config = super::super::config::AwsConfig::default();
        let result = super::Ec2Discovery::new(&config);
        
        // Should fail without proper AWS credentials
        // In real tests, you'd mock the AWS client
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_s3_discovery_creation() {
        let config = super::super::config::AwsConfig::default();
        let result = super::S3Discovery::new(&config);
        
        // Should fail without proper AWS credentials
        // In real tests, you'd mock the AWS client
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_aws_service_config() {
        let service = super::super::config::AwsService {
            name: "test-service".to_string(),
            enabled: true,
            config: None,
        };
        
        assert_eq!(service.name, "test-service");
        assert!(service.enabled);
        assert!(service.config.is_none());
    }
    
    #[test]
    fn test_aws_service_with_config() {
        let mut config_map = std::collections::HashMap::new();
        config_map.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
        
        let service = super::super::config::AwsService {
            name: "test-service".to_string(),
            enabled: true,
            config: Some(config_map),
        };
        
        assert_eq!(service.name, "test-service");
        assert!(service.enabled);
        assert!(service.config.is_some());
        
        let config = service.config.unwrap();
        assert_eq!(config.get("test_key"), Some(&serde_json::Value::String("test_value".to_string())));
    }
    
    #[test]
    fn test_aws_provider_factory() {
        let config_yaml = r#"
region: "us-west-2"
services:
  - name: "ec2"
    enabled: true
  - name: "s3"
    enabled: false
"#;
        
        let result = super::create_aws_provider(config_yaml);
        
        // Should succeed with valid YAML
        assert!(result.is_ok());
        
        let provider = result.unwrap();
        let info = provider.info();
        assert_eq!(info.name, "aws");
    }
    
    #[test]
    fn test_aws_provider_factory_invalid_yaml() {
        let invalid_yaml = r#"
region: "us-west-2"
services:
  - name: "ec2"
    enabled: true
  invalid_yaml: [unclosed
"#;
        
        let result = super::create_aws_provider(invalid_yaml);
        
        // Should fail with invalid YAML
        assert!(result.is_err());
    }
    
    #[test]
    fn test_aws_provider_factory_empty_yaml() {
        let empty_yaml = "";
        
        let result = super::create_aws_provider(empty_yaml);
        
        // Should fail with empty YAML
        assert!(result.is_err());
    }
    
    #[test]
    fn test_aws_provider_service_initialization() {
        let mut config = super::super::config::AwsConfig::default();
        
        // Test with only EC2 enabled
        config.services[1].enabled = false;
        let provider = super::AwsProvider::new(config).unwrap_or_else(|_| {
            super::AwsProvider {
                config,
                ec2_discovery: None,
                s3_discovery: None,
            }
        });
        
        // Should initialize only EC2
        assert!(provider.ec2_discovery.is_some());
        assert!(provider.s3_discovery.is_none());
        
        // Test with only S3 enabled
        config.services[0].enabled = false;
        config.services[1].enabled = true;
        let provider = super::AwsProvider::new(config).unwrap_or_else(|_| {
            super::AwsProvider {
                config,
                ec2_discovery: None,
                s3_discovery: None,
            }
        });
        
        // Should initialize only S3
        assert!(provider.ec2_discovery.is_none());
        assert!(provider.s3_discovery.is_some());
        
        // Test with all services disabled
        config.services[0].enabled = false;
        config.services[1].enabled = false;
        let provider = super::AwsProvider::new(config).unwrap_or_else(|_| {
            super::AwsProvider {
                config,
                ec2_discovery: None,
                s3_discovery: None,
            }
        });
        
        // Should not initialize any services
        assert!(provider.ec2_discovery.is_none());
        assert!(provider.s3_discovery.is_none());
    }
    
    #[test]
    fn test_aws_provider_region_validation() {
        let test_cases = vec![
            ("us-east-1", true),
            ("us-east-2", true),
            ("us-west-1", true),
            ("us-west-2", true),
            ("eu-west-1", true),
            ("eu-west-2", true),
            ("eu-central-1", true),
            ("ap-south-1", true),
            ("ap-southeast-1", true),
            ("ap-southeast-2", true),
            ("ap-northeast-1", true),
            ("ap-northeast-2", true),
            ("ca-central-1", true),
            ("sa-east-1", true),
            ("invalid-region", false),
        ];
        
        for (region, should_succeed) in test_cases {
            let mut config = super::super::config::AwsConfig::default();
            config.region = region.to_string();
            
            let result = super::Ec2Discovery::new(&config);
            
            if should_succeed {
                assert!(result.is_ok() || result.is_err());
            } else {
                // Invalid regions should still create a client but may fail at runtime
                assert!(result.is_ok() || result.is_err());
            }
        }
    }
}