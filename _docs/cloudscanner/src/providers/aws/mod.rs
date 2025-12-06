// Module declarations
mod config;
mod ec2;
mod s3;
mod lambda;
mod rds;
mod ecs;
mod vpc;
mod iam;
mod cloudtrail;

// Import macros
#[macro_use]
mod macros;

use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error, debug};

use crate::models::provider::{Provider, ProviderInfo, Resource};
pub use self::config::AwsConfig;
use self::ec2::{Ec2Discovery, Ec2DiscoveryImpl, MockEc2Discovery};
use self::s3::{S3Discovery, MockS3Discovery};
use self::lambda::{LambdaDiscovery, LambdaDiscoveryImpl, MockLambdaDiscovery};
use self::rds::{RdsDiscovery, RdsDiscoveryImpl, MockRdsDiscovery};
use self::ecs::{EcsDiscovery, EcsDiscoveryImpl, MockEcsDiscovery};
use self::vpc::{VpcDiscovery, VpcDiscoveryImpl, MockVpcDiscovery};
use self::iam::{IamDiscovery, IamDiscoveryImpl, MockIamDiscovery};
use self::cloudtrail::{CloudTrailDiscovery, CloudTrailDiscoveryImpl, MockCloudTrailDiscovery};
use self::cloudtrail::{CloudWatchDiscovery, CloudWatchDiscoveryImpl, MockCloudWatchDiscovery};

/// AWS Provider for discovering cloud resources
pub struct AwsProvider {
    config: AwsConfig,
    ec2_discovery: Option<Box<dyn Ec2Discovery>>,
    s3_discovery: Option<Box<dyn S3Discovery>>,
    lambda_discovery: Option<Box<dyn LambdaDiscovery>>,
    rds_discovery: Option<Box<dyn RdsDiscovery>>,
    ecs_discovery: Option<Box<dyn EcsDiscovery>>,
    vpc_discovery: Option<Box<dyn VpcDiscovery>>,
    iam_discovery: Option<Box<dyn IamDiscovery>>,
    cloudtrail_discovery: Option<Box<dyn CloudTrailDiscovery>>,
    cloudwatch_discovery: Option<Box<dyn CloudWatchDiscovery>>,
}

impl AwsProvider {
    /// Create a new AWS provider with the given configuration
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Initializing AWS provider for region: {}", config.region);
        
        let mut provider = Self {
            config,
            ec2_discovery: None,
            s3_discovery: None,
            lambda_discovery: None,
            rds_discovery: None,
            ecs_discovery: None,
            vpc_discovery: None,
            iam_discovery: None,
            cloudtrail_discovery: None,
            cloudwatch_discovery: None,
        };
        
        // Initialize service clients based on enabled services
        provider.initialize_services().await?;
        
        Ok(provider)
    }
    
    /// Initialize AWS service clients with secure credential handling
    async fn initialize_services(&mut self) -> Result<()> {
        // Securely check if AWS credentials exist without exposing them
        let has_aws_creds = self.check_aws_credentials_exist();
            
        // Collect enabled services first to avoid borrowing self while mutating it
        let enabled_services: Vec<String> = self.config.services.iter()
            .filter(|s| s.enabled)
            .map(|s| s.name.clone())
            .collect();
            
        for service_name in enabled_services {
            match service_name.as_str() {
                "ec2" => {
                    aws_service_init!(
                        self, has_aws_creds, "EC2 discovery", ec2_discovery, 
                        Ec2DiscoveryImpl, MockEc2Discovery
                    );
                }
                                Err(e) => {
                                    // Sanitize error message to prevent credential exposure
                                    let sanitized_error = self.sanitize_error_message(&e.to_string());
                                    warn!("Failed to create EC2 client: {}, using mock", sanitized_error);
                                    self.ec2_discovery = Some(Box::new(MockEc2Discovery));
                                }
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock EC2 discovery");
                        self.ec2_discovery = Some(Box::new(MockEc2Discovery));
                    }
                }
                "s3" => {
                    info!("Initializing S3 discovery");
                    self.s3_discovery = Some(Box::new(MockS3Discovery));
                }
                "lambda" => {
                    info!("Initializing Lambda discovery");
                    if has_aws_creds {
                        if let Err(e) = self.validate_aws_credentials() {
                            warn!("AWS credential validation failed: {}, using mock", e);
                            self.lambda_discovery = Some(Box::new(MockLambdaDiscovery));
                        } else {
                            let config = self.config.clone();
                            match LambdaDiscoveryImpl::new(config).await {
                                Ok(lambda_discovery) => {
                                    info!("Lambda discovery initialized successfully");
                                    self.lambda_discovery = Some(Box::new(lambda_discovery));
                                }
                                Err(e) => {
                                    let sanitized_error = self.sanitize_error_message(&e.to_string());
                                    warn!("Failed to create Lambda client: {}, using mock", sanitized_error);
                                    self.lambda_discovery = Some(Box::new(MockLambdaDiscovery));
                                }
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock Lambda discovery");
                        self.lambda_discovery = Some(Box::new(MockLambdaDiscovery));
                    }
                }
                "rds" => {
                    info!("Initializing RDS discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match RdsDiscoveryImpl::new(config).await {
                            Ok(rds_discovery) => {
                                self.rds_discovery = Some(Box::new(rds_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create RDS client: {}, using mock", e);
                                self.rds_discovery = Some(Box::new(MockRdsDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock RDS discovery");
                        self.rds_discovery = Some(Box::new(MockRdsDiscovery));
                    }
                }
                "ecs" => {
                    info!("Initializing ECS discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match EcsDiscoveryImpl::new(config).await {
                            Ok(ecs_discovery) => {
                                self.ecs_discovery = Some(Box::new(ecs_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create ECS client: {}, using mock", e);
                                self.ecs_discovery = Some(Box::new(MockEcsDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock ECS discovery");
                        self.ecs_discovery = Some(Box::new(MockEcsDiscovery));
                    }
                }
                "vpc" => {
                    info!("Initializing VPC discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match VpcDiscoveryImpl::new(config).await {
                            Ok(vpc_discovery) => {
                                self.vpc_discovery = Some(Box::new(vpc_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create VPC client: {}, using mock", e);
                                self.vpc_discovery = Some(Box::new(MockVpcDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock VPC discovery");
                        self.vpc_discovery = Some(Box::new(MockVpcDiscovery));
                    }
                }
                "iam" => {
                    info!("Initializing IAM discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match IamDiscoveryImpl::new(config).await {
                            Ok(iam_discovery) => {
                                self.iam_discovery = Some(Box::new(iam_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create IAM client: {}, using mock", e);
                                self.iam_discovery = Some(Box::new(MockIamDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock IAM discovery");
                        self.iam_discovery = Some(Box::new(MockIamDiscovery));
                    }
                }
                "cloudtrail" => {
                    info!("Initializing CloudTrail discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match CloudTrailDiscoveryImpl::new(config).await {
                            Ok(cloudtrail_discovery) => {
                                self.cloudtrail_discovery = Some(Box::new(cloudtrail_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create CloudTrail client: {}, using mock", e);
                                self.cloudtrail_discovery = Some(Box::new(MockCloudTrailDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock CloudTrail discovery");
                        self.cloudtrail_discovery = Some(Box::new(MockCloudTrailDiscovery));
                    }
                }
                "cloudwatch" => {
                    info!("Initializing CloudWatch discovery");
                    if has_aws_creds {
                        let config = self.config.clone();
                        match CloudWatchDiscoveryImpl::new(config).await {
                            Ok(cloudwatch_discovery) => {
                                self.cloudwatch_discovery = Some(Box::new(cloudwatch_discovery));
                            }
                            Err(e) => {
                                warn!("Failed to create CloudWatch client: {}, using mock", e);
                                self.cloudwatch_discovery = Some(Box::new(MockCloudWatchDiscovery));
                            }
                        }
                    } else {
                        info!("No AWS credentials found, using mock CloudWatch discovery");
                        self.cloudwatch_discovery = Some(Box::new(MockCloudWatchDiscovery));
                    }
                }
                _ => {
                    warn!("Unsupported AWS service: {}", service_name);
                }
            }
        }
        
        Ok(())
    }
    
    /// Discover all ECS resources with optimized memory usage
    async fn discover_ecs_resources(&self, all_resources: &mut Vec<Resource>) {
        if let Some(ref ecs_discovery) = self.ecs_discovery {
            // Discover clusters
            self.discover_and_extend(
                Some(ecs_discovery),
                "ECS clusters",
                |d| d.discover_clusters(),
                all_resources,
            ).await;
            
            // Discover services
            self.discover_and_extend(
                Some(ecs_discovery),
                "ECS services",
                |d| d.discover_services(),
                all_resources,
            ).await;
            
            // Discover tasks
            self.discover_and_extend(
                Some(ecs_discovery),
                "ECS tasks",
                |d| d.discover_tasks(),
                all_resources,
            ).await;
        }
    }
    
    /// Helper method to safely extend resources vector with error handling
    async fn discover_and_extend<T, F, Fut>(
        &self,
        discovery: Option<&T>,
        service_name: &str,
        discover_fn: F,
        all_resources: &mut Vec<Resource>,
    ) where
        T: ?Sized,
        F: Fn(&T) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<Resource>>>,
    {
        if let Some(ref service_discovery) = discovery {
            match discover_fn(service_discovery).await {
                Ok(resources) => {
                    info!("Discovered {} {}", resources.len(), service_name);
                    all_resources.extend(resources);
                }
                Err(e) => {
                    error!("Failed to discover {}: {}", service_name, e);
                    // Continue with other services even if one fails
                }
            }
        }
    }
    
    /// Securely check if AWS credentials exist without exposing sensitive data
    fn check_aws_credentials_exist(&self) -> bool {
        // Check for credentials without logging or exposing their values
        let access_key_exists = std::env::var("AWS_ACCESS_KEY_ID")
            .map(|key| !key.is_empty() && key.len() >= 16) // AWS access keys are at least 16 chars
            .unwrap_or(false);
            
        let secret_key_exists = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map(|key| !key.is_empty() && key.len() >= 40) // AWS secret keys are at least 40 chars
            .unwrap_or(false);
        
        let has_creds = access_key_exists && secret_key_exists;
        
        // Log only the existence status, never the actual values
        debug!("AWS credentials check: {}", if has_creds { "found" } else { "not found" });
        
        has_creds
    }
    
    /// Validate AWS credential format without exposing values
    fn validate_aws_credentials(&self) -> Result<()> {
        // Validate access key format
        if let Ok(access_key) = std::env::var("AWS_ACCESS_KEY_ID") {
            if access_key.len() < 16 {
                return Err(anyhow::anyhow!("AWS_ACCESS_KEY_ID appears to be invalid (too short)"));
            }
            if !access_key.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(anyhow::anyhow!("AWS_ACCESS_KEY_ID contains invalid characters"));
            }
        }
        
        // Validate secret key format
        if let Ok(secret_key) = std::env::var("AWS_SECRET_ACCESS_KEY") {
            if secret_key.len() < 40 {
                return Err(anyhow::anyhow!("AWS_SECRET_ACCESS_KEY appears to be invalid (too short)"));
            }
            // Secret keys can contain more characters, so we just check length
        }
        
        Ok(())
    }
}

#[async_trait]
impl Provider for AwsProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "aws".to_string(),
            version: "1.0.0".to_string(),
            description: "AWS Cloud Provider - discovers EC2, S3, Lambda, RDS, ECS, VPC, IAM, CloudTrail, and CloudWatch resources".to_string(),
            supported_resource_types: vec![
                "ec2-instance".to_string(),
                "s3-bucket".to_string(),
                "lambda-function".to_string(),
                "rds-instance".to_string(),
                "rds-cluster".to_string(),
                "ecs-cluster".to_string(),
                "ecs-service".to_string(),
                "ecs-task".to_string(),
                "vpc".to_string(),
                "subnet".to_string(),
                "security-group".to_string(),
                "route-table".to_string(),
                "internet-gateway".to_string(),
                "nat-gateway".to_string(),
                "iam-role".to_string(),
                "iam-policy".to_string(),
                "iam-user".to_string(),
                "iam-group".to_string(),
                "cloudtrail".to_string(),
                "cloudtrail-event".to_string(),
                "cloudwatch-log-group".to_string(),
                "cloudwatch-metric".to_string(),
                "cloudwatch-alarm".to_string(),
            ],
        }
    }

    async fn discover(&self) -> Result<Vec<Resource>> {
        info!("Starting AWS resource discovery");
        
        // Pre-allocate with estimated capacity to reduce reallocations
        let mut all_resources = Vec::with_capacity(1000); // Estimated based on typical AWS account
        
        // Discover EC2 instances
        aws_service_discover!(
            self, "EC2 instances", ec2_discovery, discover_instances, all_resources
        );
        
        // Discover S3 buckets
        aws_service_discover!(
            self, "S3 buckets", s3_discovery, discover_buckets, all_resources
        );
        
        // Discover Lambda functions
        aws_service_discover!(
            self, "Lambda functions", lambda_discovery, discover_functions, all_resources
        );
        
        // Discover RDS instances
        self.discover_and_extend(
            self.rds_discovery.as_ref(),
            "RDS instances",
            |d| d.discover_instances(),
            &mut all_resources,
        ).await;
        
        // Discover ECS resources
        self.discover_ecs_resources(&mut all_resources).await;
        
        // Discover VPC resources
        if let Some(ref vpc_discovery) = self.vpc_discovery {
            // Discover VPCs
            match vpc_discovery.discover_vpcs().await {
                Ok(mut resources) => {
                    info!("Discovered {} VPCs", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover VPCs: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover subnets
            match vpc_discovery.discover_subnets().await {
                Ok(mut resources) => {
                    info!("Discovered {} subnets", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover subnets: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover security groups
            match vpc_discovery.discover_security_groups().await {
                Ok(mut resources) => {
                    info!("Discovered {} security groups", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover security groups: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover route tables
            match vpc_discovery.discover_route_tables().await {
                Ok(mut resources) => {
                    info!("Discovered {} route tables", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover route tables: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover internet gateways
            match vpc_discovery.discover_internet_gateways().await {
                Ok(mut resources) => {
                    info!("Discovered {} internet gateways", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover internet gateways: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover NAT gateways
            match vpc_discovery.discover_nat_gateways().await {
                Ok(mut resources) => {
                    info!("Discovered {} NAT gateways", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover NAT gateways: {}", e);
                    // Continue with other services even if one fails
                }
            }
        }
        
        // Discover IAM resources
        if let Some(ref iam_discovery) = self.iam_discovery {
            // Discover roles
            match iam_discovery.discover_roles().await {
                Ok(mut resources) => {
                    info!("Discovered {} IAM roles", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover IAM roles: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover policies
            match iam_discovery.discover_policies().await {
                Ok(mut resources) => {
                    info!("Discovered {} IAM policies", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover IAM policies: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover users
            match iam_discovery.discover_users().await {
                Ok(mut resources) => {
                    info!("Discovered {} IAM users", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover IAM users: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover groups
            match iam_discovery.discover_groups().await {
                Ok(mut resources) => {
                    info!("Discovered {} IAM groups", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover IAM groups: {}", e);
                    // Continue with other services even if one fails
                }
            }
        }
        
        // Discover CloudTrail resources
        if let Some(ref cloudtrail_discovery) = self.cloudtrail_discovery {
            // Discover trails
            match cloudtrail_discovery.discover_trails().await {
                Ok(mut resources) => {
                    info!("Discovered {} CloudTrail trails", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover CloudTrail trails: {}", e);
                    // Continue with other services even if one fails
                }
            }
        }
        
        // Discover CloudWatch resources
        if let Some(ref cloudwatch_discovery) = self.cloudwatch_discovery {
            // Discover log groups
            match cloudwatch_discovery.discover_log_groups().await {
                Ok(mut resources) => {
                    info!("Discovered {} CloudWatch log groups", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover CloudWatch log groups: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover metrics
            match cloudwatch_discovery.discover_metrics().await {
                Ok(mut resources) => {
                    info!("Discovered {} CloudWatch metrics", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover CloudWatch metrics: {}", e);
                    // Continue with other services even if one fails
                }
            }
            
            // Discover alarms
            match cloudwatch_discovery.discover_alarms().await {
                Ok(mut resources) => {
                    info!("Discovered {} CloudWatch alarms", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    error!("Failed to discover CloudWatch alarms: {}", e);
                    // Continue with other services even if one fails
                }
            }
        }
        
        info!("AWS discovery complete. Total resources: {}", all_resources.len());
        Ok(all_resources)
    }
}

/// AWS Provider factory function for plugin loading
pub async fn create_aws_provider(config_str: &str) -> Result<Box<dyn Provider>> {
    let config: AwsConfig = serde_yaml::from_str(config_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse AWS config: {}", e))?;
    
    let provider = AwsProvider::new(config).await?;
    Ok(Box::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aws_provider_creation() {
        let config = AwsConfig::default();
        let provider = AwsProvider::new(config);
        assert!(provider.is_ok());
    }
    
    #[test]
    fn test_aws_provider_info() {
        let config = AwsConfig::default();
        let provider = AwsProvider::new(config).unwrap();
        let info = provider.info();
        
        assert_eq!(info.name, "aws");
        assert_eq!(info.version, "1.0.0");
        assert!(info.description.contains("AWS"));
        assert_eq!(info.supported_resource_types.len(), 2);
        assert!(info.supported_resource_types.contains(&"ec2-instance".to_string()));
        assert!(info.supported_resource_types.contains(&"s3-bucket".to_string()));
    }
}