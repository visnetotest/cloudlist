use async_trait::async_trait;
use tracing::info;

use crate::providers::base::{DiscoveryProvider, Asset};
use crate::error::CloudScannerError;

/// Mock ELB discovery for testing
pub struct MockElbDiscovery;

impl Default for MockElbDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
pub trait ElbDiscovery: Send + Sync {
    async fn discover_load_balancers(&self) -> Result<Vec<Asset>, CloudScannerError>;
}

#[async_trait]
impl ElbDiscovery for MockElbDiscovery {
    async fn discover_load_balancers(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Mock ELB discovery: returning empty load balancers list");
        Ok(vec![])
    }
}

/// Real ELB discovery implementation
pub struct ElbDiscoveryImpl;

impl ElbDiscoveryImpl {
    pub async fn new(_config: super::AwsProviderConfig) -> Result<Self, CloudScannerError> {
        info!("Initializing real ELB discovery");
        Ok(Self)
    }
}

#[async_trait]
impl ElbDiscovery for ElbDiscoveryImpl {
    async fn discover_load_balancers(&self) -> Result<Vec<Asset>, CloudScannerError> {
        info!("Real ELB discovery not yet implemented");
        Ok(vec![])
    }
}