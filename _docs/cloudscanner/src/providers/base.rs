// cloudscanner/src/providers/base.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::CloudScannerError;

/// Represents a discovered cloud asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Asset type (e.g., "ec2:instance", "s3:bucket")
    pub asset_type: String,
    /// Unique identifier for the asset
    pub id: String,
    /// Asset name (if available)
    pub name: Option<String>,
    /// Region where the asset is located
    pub region: Option<String>,
    /// Additional metadata about the asset
    pub metadata: HashMap<String, String>,
    /// Tags associated with the asset
    pub tags: HashMap<String, String>,
}

impl Asset {
    pub fn new(asset_type: String, id: String) -> Self {
        Self {
            asset_type,
            id,
            name: None,
            region: None,
            metadata: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Returns a unique identifier for this asset across providers
    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.asset_type, self.id)
    }
}

/// Trait that all discovery providers must implement
#[async_trait]
pub trait DiscoveryProvider: Send + Sync {
    /// Discover assets and return a list of discovered assets
    async fn discover(&self) -> Result<Vec<Asset>, CloudScannerError>;

    /// Get the provider name
    fn provider_name(&self) -> String;

    /// Get the provider version
    fn provider_version(&self) -> String;

    /// Get the list of supported services
    fn supported_services(&self) -> Vec<String>;

    /// Optional: Health check for the provider
    async fn health_check(&self) -> Result<bool, CloudScannerError> {
        Ok(true)
    }

    /// Optional: Cleanup resources used by the provider
    async fn cleanup(&self) -> Result<(), CloudScannerError> {
        Ok(())
    }
}