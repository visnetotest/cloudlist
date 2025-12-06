use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_ec2::Client;
use aws_types::region::Region;
use tracing::{info, error, debug};

use crate::models::provider::Resource;
use super::config::AwsConfig;

/// Trait for EC2 discovery services
#[async_trait]
pub trait Ec2Discovery: Send + Sync {
    async fn discover_instances(&self) -> Result<Vec<Resource>>;
}

/// Real EC2 Discovery Service Implementation
pub struct Ec2DiscoveryImpl {
    client: Client,
    region: String,
}

impl Ec2DiscoveryImpl {
    /// Create a new EC2 discovery service
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating EC2 client for region: {}", config.region);
        
        let region = Region::new(config.region.clone());
        
        let shared_config = aws_config::from_env()
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
    async fn discover_instances(&self) -> Result<Vec<Resource>> {
        info!("Starting EC2 instance discovery in region: {}", self.region);
        let mut resources = Vec::new();
        
        match self.client.describe_instances().send().await {
            Ok(response) => {
                if let Some(reservations) = response.reservations() {
                    for reservation in reservations {
                        if let Some(instances) = reservation.instances() {
                            for instance in instances {
                                debug!("Found instance: {:?}", instance);
                                
                                let instance_id = instance.instance_id()
                                    .unwrap_or("unknown")
                                    .to_string();
                                
                                let is_running = instance.state()
                                    .and_then(|state| state.name())
                                    .map(|s| matches!(s, aws_sdk_ec2::types::InstanceStateName::Running))
                                    .unwrap_or(false);
                                
                                // Only include running instances for demo
                                if is_running {
                                    let mut resource = Resource::new(
                                        "ec2-instance".to_string(),
                                        instance_id.clone()
                                    );
                                    
                                    // Add metadata
                                    resource.add_metadata("region".to_string(), self.region.clone());
                                    resource.add_metadata("instance_id".to_string(), instance_id.clone());
                                    resource.add_metadata("state".to_string(), "Running".to_string());
                                    
                                    if let Some(instance_type) = instance.instance_type() {
                                        resource.add_metadata(
                                            "instance_type".to_string(),
                                            format!("{:?}", instance_type)
                                        );
                                    }
                                    
                                    if let Some(public_ip) = instance.public_ip_address() {
                                        resource.add_metadata(
                                            "public_ip".to_string(),
                                            public_ip.to_string()
                                        );
                                    }
                                    
                                    if let Some(private_ip) = instance.private_ip_address() {
                                        resource.add_metadata(
                                            "private_ip".to_string(),
                                            private_ip.to_string()
                                        );
                                    }
                                    
                                    if let Some(launch_time) = instance.launch_time() {
                                        // Convert DateTime to string using Debug format
                                        resource.add_metadata(
                                            "launch_time".to_string(),
                                            format!("{:?}", launch_time)
                                        );
                                    }
                                    
                                    resources.push(resource);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to describe EC2 instances: {}", e);
            }
        }
        
        info!("Discovered {} EC2 instances in region {}", resources.len(), self.region);
        Ok(resources)
    }
}

/// Mock EC2 Discovery for testing
pub struct MockEc2Discovery;

#[async_trait]
impl Ec2Discovery for MockEc2Discovery {
    async fn discover_instances(&self) -> Result<Vec<Resource>> {
        info!("Mock EC2 discovery returning test instances");
        Ok(vec![
            Resource::new("ec2-instance".to_string(), "i-1234567890abcdef0".to_string())
                .with_metadata("region".to_string(), "us-east-1".to_string())
                .with_metadata("state".to_string(), "running".to_string())
                .with_metadata("instance_type".to_string(), "t3.micro".to_string())
                .with_metadata("public_ip".to_string(), "3.84.123.45".to_string()),
        ])
    }
}