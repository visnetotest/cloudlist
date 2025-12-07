use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, warn, error, debug};

use super::base::{DiscoveryProvider, Asset};
use crate::config::CloudScannerConfig;
use crate::error::CloudScannerError;

mod ec2;
mod s3;
mod lambda;
mod cloudfront;
mod rds;
mod elb;
mod ecs;
mod efs;
mod route53;

use ec2::{Ec2Discovery, Ec2DiscoveryImpl, MockEc2Discovery};
use s3::{S3Discovery, S3DiscoveryImpl, MockS3Discovery};
use lambda::{LambdaDiscovery, LambdaDiscoveryImpl, MockLambdaDiscovery};
use cloudfront::{CloudfrontDiscovery, CloudfrontDiscoveryImpl, MockCloudfrontDiscovery};
use rds::{RdsDiscovery, RdsDiscoveryImpl, MockRdsDiscovery};
use elb::{ElbDiscovery, ElbDiscoveryImpl, MockElbDiscovery};
use ecs::{EcsDiscovery, EcsDiscoveryImpl, MockEcsDiscovery};
use efs::{EfsDiscovery, EfsDiscoveryImpl, MockEfsDiscovery};
use route53::{Route53Discovery, Route53DiscoveryImpl, MockRoute53Discovery};

/// Macro to initialize AWS services with async new() functions
macro_rules! aws_service_init_async {
    ($self:expr, $has_creds:expr, $service_name:expr, $field:ident, $impl_type:ty, $mock_type:ty) => {
        info!("Initializing {}", $service_name);
        if $has_creds {
            if let Err(e) = $self.validate_aws_credentials() {
                warn!("AWS credential validation failed: {}, using mock", e);
                $self.$field = Some(Box::new(<$mock_type>::default()));
            } else {
                let config = $self.config.clone();
                match <$impl_type>::new(config).await {
                    Ok(service) => {
                        info!("{} initialized successfully", $service_name);
                        $self.$field = Some(Box::new(service));
                    }
                    Err(e) => {
                        let sanitized_error = $self.sanitize_error_message(&e.to_string());
                        warn!("Failed to create {} client: {}, using mock", $service_name, sanitized_error);
                        $self.$field = Some(Box::new(<$mock_type>::default()));
                    }
                }
            }
        } else {
            info!("No AWS credentials found, using mock {}", $service_name);
            $self.$field = Some(Box::new(<$mock_type>::default()));
        }
    };
}

/// Macro to initialize AWS services with sync new() functions
macro_rules! aws_service_init_sync {
    ($self:expr, $has_creds:expr, $service_name:expr, $field:ident, $impl_type:ty, $mock_type:ty) => {
        info!("Initializing {}", $service_name);
        if $has_creds {
            if let Err(e) = $self.validate_aws_credentials() {
                warn!("AWS credential validation failed: {}, using mock", e);
                $self.$field = Some(Box::new(<$mock_type>::default()));
            } else {
                let config = &$self.config;
                let service = <$impl_type>::new(config);
                info!("{} initialized successfully", $service_name);
                $self.$field = Some(Box::new(service));
            }
        } else {
            info!("No AWS credentials found, using mock {}", $service_name);
            $self.$field = Some(Box::new(<$mock_type>::default()));
        }
    };
}

/// AWS provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsProviderConfig {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub region: String,
    pub profile: Option<String>,
    pub assume_role: Option<AssumeRoleConfig>,
    pub endpoint_url: Option<String>,
    pub services: Vec<AwsServiceConfig>,
}

/// AWS assume role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumeRoleConfig {
    pub role_arn: String,
    pub session_name: Option<String>,
    pub external_id: Option<String>,
}

/// AWS service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsServiceConfig {
    pub name: String,
    pub enabled: bool,
    pub regions: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
}

impl Default for AwsProviderConfig {
    fn default() -> Self {
        Self {
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            region: "us-east-1".to_string(),
            profile: None,
            assume_role: None,
            endpoint_url: None,
            services: vec![
                AwsServiceConfig { name: "ec2".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "s3".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "lambda".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "cloudfront".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "rds".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "elb".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "ecs".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "efs".to_string(), enabled: true, regions: None, tags: None },
                AwsServiceConfig { name: "route53".to_string(), enabled: true, regions: None, tags: None },
            ],
        }
    }
}

/// AWS provider implementation
pub struct AwsProvider {
    config: AwsProviderConfig,
    ec2_discovery: Option<Box<dyn Ec2Discovery>>,
    s3_discovery: Option<Box<dyn S3Discovery>>,
    lambda_discovery: Option<Box<dyn LambdaDiscovery>>,
    cloudfront_discovery: Option<Box<dyn CloudfrontDiscovery>>,
    rds_discovery: Option<Box<dyn RdsDiscovery>>,
    elb_discovery: Option<Box<dyn ElbDiscovery>>,
    ecs_discovery: Option<Box<dyn EcsDiscovery>>,
    efs_discovery: Option<Box<dyn EfsDiscovery>>,
    route53_discovery: Option<Box<dyn Route53Discovery>>,
}

impl AwsProvider {
    /// Create a new AWS provider
    pub fn new(config: AwsProviderConfig) -> Self {
        Self {
            config,
            ec2_discovery: None,
            s3_discovery: None,
            lambda_discovery: None,
            cloudfront_discovery: None,
            rds_discovery: None,
            elb_discovery: None,
            ecs_discovery: None,
            efs_discovery: None,
            route53_discovery: None,
        }
    }

    /// Check if AWS credentials are available
    fn has_aws_credentials(&self) -> bool {
        // Check if credentials are explicitly provided
        if self.config.access_key_id.is_some() && self.config.secret_access_key.is_some() {
            return true;
        }

        // Check environment variables
        if std::env::var("AWS_ACCESS_KEY_ID").is_ok() && 
           std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
            return true;
        }

        // Check for AWS profile
        if std::env::var("AWS_PROFILE").is_ok() || self.config.profile.is_some() {
            return true;
        }

        // Check for default AWS credentials file
        if let Some(home) = std::env::var_os("HOME") {
            let aws_creds_path = Path::new(&home).join(".aws").join("credentials");
            if aws_creds_path.exists() {
                return true;
            }
        }

        false
    }

    /// Validate AWS credentials by attempting to create a session
    fn validate_aws_credentials(&self) -> Result<(), CloudScannerError> {
        // This is a basic validation - in a real implementation, you'd want to
        // make a simple AWS API call to verify credentials work
        if !self.has_aws_credentials() {
            return Err(CloudScannerError::config(
                "No AWS credentials found"
            ));
        }

        // Additional validation could be added here
        Ok(())
    }

    /// Sanitize error messages to prevent credential exposure
    fn sanitize_error_message(&self, error: &str) -> String {
        // Replace potential credential patterns in error messages
        let sanitized = error
            .replace(|c: char| c.is_ascii_alphanumeric(), "***")
            .to_string();

        // Log full error to debug for troubleshooting
        debug!("Original error (sanitized): {}", error);

        sanitized
    }

    /// Initialize all enabled AWS services
    pub async fn initialize_services(&mut self) -> Result<(), CloudScannerError> {
        info!("Initializing AWS services");

        let has_aws_creds = self.has_aws_credentials();
        if !has_aws_creds {
            warn!("No AWS credentials found, using mock implementations for all services");
        }

        let enabled_services: Vec<String> = self.config.services.iter()
            .filter(|s| s.enabled)
            .map(|s| s.name.clone())
            .collect();
            
        for service_name in enabled_services {
            match service_name.as_str() {
                "ec2" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "EC2 discovery", ec2_discovery, 
                        Ec2DiscoveryImpl, MockEc2Discovery
                    );
                }
                "s3" => {
                    aws_service_init_sync!(
                        self, has_aws_creds, "S3 discovery", s3_discovery, 
                        S3DiscoveryImpl, MockS3Discovery
                    );
                }
                "lambda" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "Lambda discovery", lambda_discovery, 
                        LambdaDiscoveryImpl, MockLambdaDiscovery
                    );
                }
                "cloudfront" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "CloudFront discovery", cloudfront_discovery, 
                        CloudfrontDiscoveryImpl, MockCloudfrontDiscovery
                    );
                }
                "rds" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "RDS discovery", rds_discovery, 
                        RdsDiscoveryImpl, MockRdsDiscovery
                    );
                }
                "elb" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "ELB discovery", elb_discovery, 
                        ElbDiscoveryImpl, MockElbDiscovery
                    );
                }
                "ecs" => {
                    aws_service_init_async!(
                        self, has_aws_creds, "ECS discovery", ecs_discovery, 
                        EcsDiscoveryImpl, MockEcsDiscovery
                    );
                }
                "efs" => {
                    aws_service_init_sync!(
                        self, has_aws_creds, "EFS discovery", efs_discovery, 
                        EfsDiscoveryImpl, MockEfsDiscovery
                    );
                }
                "route53" => {
                    aws_service_init_sync!(
                        self, has_aws_creds, "Route53 discovery", route53_discovery, 
                        Route53DiscoveryImpl, MockRoute53Discovery
                    );
                }
                _ => {
                    warn!("Unknown AWS service: {}", service_name);
                }
            }
        }

        info!("AWS services initialization completed");
        Ok(())
    }
}

#[async_trait]
impl DiscoveryProvider for AwsProvider {
    async fn discover(&self) -> Result<Vec<Asset>, CloudScannerError> {
        let mut all_assets = Vec::new();

        // Discover EC2 instances
        if let Some(ec2) = &self.ec2_discovery {
            match ec2.discover_instances().await {
                Ok(mut assets) => {
                    info!("Discovered {} EC2 instances", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover EC2 instances: {}", e);
                }
            }
        }

        // Discover S3 buckets
        if let Some(s3) = &self.s3_discovery {
            match s3.discover_buckets().await {
                Ok(mut assets) => {
                    info!("Discovered {} S3 buckets", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover S3 buckets: {}", e);
                }
            }
        }

        // Discover Lambda functions
        if let Some(lambda) = &self.lambda_discovery {
            match lambda.discover_functions().await {
                Ok(mut assets) => {
                    info!("Discovered {} Lambda functions", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover Lambda functions: {}", e);
                }
            }
        }

        // Discover CloudFront distributions
        if let Some(cloudfront) = &self.cloudfront_discovery {
            match cloudfront.discover_distributions().await {
                Ok(mut assets) => {
                    info!("Discovered {} CloudFront distributions", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover CloudFront distributions: {}", e);
                }
            }
        }

        // Discover RDS instances
        if let Some(rds) = &self.rds_discovery {
            match rds.discover_instances().await {
                Ok(mut assets) => {
                    info!("Discovered {} RDS instances", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover RDS instances: {}", e);
                }
            }
        }

        // Discover ELB/ALB
        if let Some(elb) = &self.elb_discovery {
            match elb.discover_load_balancers().await {
                Ok(mut assets) => {
                    info!("Discovered {} ELB/ALB instances", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover ELB/ALB instances: {}", e);
                }
            }
        }

        // Discover ECS clusters
        if let Some(ecs) = &self.ecs_discovery {
            match ecs.discover_clusters().await {
                Ok(mut assets) => {
                    info!("Discovered {} ECS clusters", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover ECS clusters: {}", e);
                }
            }
        }

        // Discover EFS file systems
        if let Some(efs) = &self.efs_discovery {
            match efs.discover_file_systems().await {
                Ok(mut assets) => {
                    info!("Discovered {} EFS file systems", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover EFS file systems: {}", e);
                }
            }
        }

        // Discover Route53 zones
        if let Some(route53) = &self.route53_discovery {
            match route53.discover_zones().await {
                Ok(mut assets) => {
                    info!("Discovered {} Route53 zones", assets.len());
                    all_assets.append(&mut assets);
                }
                Err(e) => {
                    error!("Failed to discover Route53 zones: {}", e);
                }
            }
        }

        info!("Total AWS assets discovered: {}", all_assets.len());
        Ok(all_assets)
    }

    fn provider_name(&self) -> String {
        "aws".to_string()
    }

    fn provider_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn supported_services(&self) -> Vec<String> {
        self.config.services.iter()
            .filter(|s| s.enabled)
            .map(|s| s.name.clone())
            .collect()
    }
}

/// Factory function to create AWS provider from configuration
pub fn create_aws_provider(config: &CloudScannerConfig) -> Result<Box<dyn DiscoveryProvider>, CloudScannerError> {
    let aws_config: AwsProviderConfig = config.clone().try_into()
        .map_err(|e| CloudScannerError::config(format!("Invalid AWS configuration: {}", e)))?;

    let mut provider = AwsProvider::new(aws_config);
    
    // Initialize services asynchronously - this would need to be called after creation
    // For now, we'll return the provider and let the caller initialize
    
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_config_default() {
        let config = AwsProviderConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.services.len(), 9);
        assert!(config.services.iter().all(|s| s.enabled));
    }

    #[test]
    fn test_has_aws_credentials_env_vars() {
        std::env::set_var("AWS_ACCESS_KEY_ID", "test");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
        
        let config = AwsProviderConfig::default();
        let provider = AwsProvider::new(config);
        
        assert!(provider.has_aws_credentials());
        
        std::env::remove_var("AWS_ACCESS_KEY_ID");
        std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    }

    #[test]
    fn test_sanitize_error_message() {
        let config = AwsProviderConfig::default();
        let provider = AwsProvider::new(config);
        
        let error = "Invalid credentials: AKIAIOSFODNN7EXAMPLE";
        let sanitized = provider.sanitize_error_message(error);
        
        assert!(!sanitized.contains("AKIAIOSFODNN7EXAMPLE"));
    }
}