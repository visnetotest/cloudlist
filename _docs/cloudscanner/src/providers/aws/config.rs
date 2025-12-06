use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AWS Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// AWS Access Key ID
    pub access_key_id: Option<String>,
    
    /// AWS Secret Access Key
    pub secret_access_key: Option<String>,
    
    /// AWS Session Token (for temporary credentials)
    pub session_token: Option<String>,
    
    /// AWS Region
    pub region: String,
    
    /// AWS Profile Name (from ~/.aws/credentials)
    pub profile: Option<String>,
    
    /// Role ARN to assume
    pub role_arn: Option<String>,
    
    /// External ID for role assumption
    pub external_id: Option<String>,
    
    /// Custom endpoint URL
    pub endpoint_url: Option<String>,
    
    /// Whether to use SSL
    pub use_ssl: Option<bool>,
    
    /// Services to discover
    pub services: Vec<AwsService>,
}

/// AWS Services Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsService {
    /// Service name
    pub name: String,
    
    /// Whether the service is enabled
    pub enabled: bool,
    
    /// Service-specific configuration
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Default AWS configuration
impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            region: "us-east-1".to_string(),
            profile: None,
            role_arn: None,
            external_id: None,
            endpoint_url: None,
            use_ssl: Some(true),
            services: vec![
                AwsService {
                    name: "ec2".to_string(),
                    enabled: true,
                    config: None,
                },
                AwsService {
                    name: "s3".to_string(),
                    enabled: true,
                    config: None,
                },
                AwsService {
                    name: "lambda".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "rds".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "ecs".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "vpc".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "iam".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "cloudtrail".to_string(),
                    enabled: false,
                    config: None,
                },
                AwsService {
                    name: "cloudwatch".to_string(),
                    enabled: false,
                    config: None,
                },
            ],
        }
    }
}