use async_trait::async_trait;
use aws_sdk_ec2::Client;
use aws_types::region::Region;
use tracing::{info, error};

use crate::providers::base::Asset;
use crate::error::CloudScannerError;
use super::AwsProviderConfig as AwsConfig;

/// Trait for EC2 discovery services
#[async_trait]
pub trait Ec2Discovery: Send + Sync {
    async fn discover_instances(&self) -> Result<Vec<Asset>, CloudScannerError>;
}

/// Real EC2 Discovery Service Implementation
pub struct Ec2DiscoveryImpl {
    client: Client,
    region: String,
}

impl Ec2DiscoveryImpl {
    /// Create a new EC2 discovery service
    pub async fn new(config: AwsConfig) -> Result<Self, CloudScannerError> {
        info!("Creating EC2 client for region: {}", config.region);
        
        let region = Region::new(config.region.clone());
        
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region)
            .load()
            .await;
            
        let client = Client::new(&shared_config);
        
        Ok(Self {
            client,
            region: config.region.clone(),
        })
    }
}

#[async_trait]
impl Ec2Discovery for Ec2DiscoveryImpl {
    async fn discover_instances(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Starting EC2 instance discovery in region: {}", self.region);
        let mut assets = Vec::new();
        
        match self.client.describe_instances().send().await {
            Ok(response) => {
                let reservations = response.reservations();
                for reservation in reservations {
                    let instances = reservation.instances();
                    for instance in instances {
                        let state_str = instance.state()
                            .and_then(|s| s.name())
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        let instance_type_str = instance.instance_type()
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        let asset = Asset::new("ec2-instance".to_string(), 
                            instance.instance_id().unwrap_or("unknown").to_string())
                            .with_metadata("region".to_string(), self.region.clone())
                            .with_metadata("state".to_string(), state_str)
                            .with_metadata("instance_type".to_string(), instance_type_str)
                            .with_metadata("public_ip".to_string(), 
                                instance.public_ip_address().unwrap_or("").to_string());
                        
                        assets.push(asset);
                    }
                }
            }
            Err(e) => {
                error!("Failed to describe EC2 instances: {}", e);
                return Err(CloudScannerError::provider("EC2", format!("failed to describe instances: {}", e)));
            }
        }
        
        info!("Discovered {} EC2 instances in region {}", assets.len(), self.region);
        Ok(assets)
    }
}

/// Mock EC2 Discovery for testing
pub struct MockEc2Discovery;

impl Default for MockEc2Discovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Ec2Discovery for MockEc2Discovery {
    async fn discover_instances(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Mock EC2 discovery returning test instances");
        Ok(vec![
            Asset::new("ec2-instance".to_string(), "i-1234567890abcdef0".to_string())
                .with_metadata("region".to_string(), "us-east-1".to_string())
                .with_metadata("state".to_string(), "running".to_string())
                .with_metadata("instance_type".to_string(), "t3.micro".to_string())
                .with_metadata("public_ip".to_string(), "3.84.123.45".to_string()),
        ])
    }
}