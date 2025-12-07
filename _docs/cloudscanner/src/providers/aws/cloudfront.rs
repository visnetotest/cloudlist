use async_trait::async_trait;
use tracing::info;

use crate::providers::base::{DiscoveryProvider, Asset};
use crate::error::CloudScannerError;

/// Mock CloudFront discovery for testing
pub struct MockCloudfrontDiscovery;

impl Default for MockCloudfrontDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
pub trait CloudfrontDiscovery: Send + Sync {
    async fn discover_distributions(&self) -> Result<Vec<Asset>, CloudScannerError>;
}

#[async_trait]
impl CloudfrontDiscovery for MockCloudfrontDiscovery {
    async fn discover_distributions(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Mock CloudFront discovery: returning empty distributions list");
        Ok(vec![])
    }
}

/// Real CloudFront discovery implementation
pub struct CloudfrontDiscoveryImpl;

impl CloudfrontDiscoveryImpl {
    pub async fn new(_config: super::AwsProviderConfig) -> Result<Self, CloudScannerError> {
        info!("Initializing real CloudFront discovery");
        Ok(Self)
    }
}

#[async_trait]
impl CloudfrontDiscovery for CloudfrontDiscoveryImpl {
    async fn discover_distributions(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Real CloudFront discovery not yet implemented");
        Ok(vec![])
    }
}