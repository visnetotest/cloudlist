// cloudscanner-provider-dummy/src/lib.rs

use cloudscanner::models::provider::{Provider, Resource, TraitObject, ProviderInfo, Plugin, PluginInfo};
use anyhow::Result;
use std::collections::HashMap;
use std::mem;
use async_trait::async_trait;
use tokio::time::{sleep, Duration};

struct DummyProvider;

#[async_trait]
impl Provider for DummyProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "dummy".to_string(),
            version: "1.0.0".to_string(),
            description: "Dummy provider for testing purposes".to_string(),
            supported_resource_types: vec!["s3-bucket".to_string(), "ec2-instance".to_string()],
        }
    }

    async fn discover(&self) -> Result<Vec<Resource>> {
        println!("Dummy provider is discovering resources...");
        
        // Simulate some async work
        sleep(Duration::from_millis(100)).await;
        
        let mut resources = Vec::new();
        
        // Create a dummy S3 bucket
        let mut s3_metadata = HashMap::new();
        s3_metadata.insert("public".to_string(), "true".to_string());
        s3_metadata.insert("region".to_string(), "us-east-1".to_string());
        
        let s3_resource = Resource {
            asset_type: "s3-bucket".to_string(),
            id: "dummy-s3-bucket-123".to_string(),
            metadata: s3_metadata,
        };
        resources.push(s3_resource);
        
        // Create a dummy EC2 instance
        let mut ec2_metadata = HashMap::new();
        ec2_metadata.insert("state".to_string(), "running".to_string());
        ec2_metadata.insert("instance_type".to_string(), "t3.micro".to_string());
        
        let ec2_resource = Resource {
            asset_type: "ec2-instance".to_string(),
            id: "i-1234567890abcdef0".to_string(),
            metadata: ec2_metadata,
        };
        resources.push(ec2_resource);

        Ok(resources)
    }

    async fn health_check(&self) -> Result<bool> {
        // Simulate health check
        sleep(Duration::from_millis(50)).await;
        Ok(true)
    }

    async fn cleanup(&self) -> Result<()> {
        println!("Dummy provider cleaning up...");
        Ok(())
    }
}

struct DummyPlugin;

#[async_trait]
impl Plugin for DummyPlugin {
    fn create_provider(&self) -> Box<dyn Provider> {
        Box::new(DummyProvider)
    }
    
    fn plugin_info(&self) -> PluginInfo {
        PluginInfo {
            name: "dummy-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Dummy plugin for testing purposes".to_string(),
            provider_name: "dummy".to_string(),
        }
    }
}

/// The function that creates an instance of the new-style plugin.
#[no_mangle]
pub extern "C" fn _create_plugin() -> *mut dyn Plugin {
    let plugin = DummyPlugin;
    let boxed_plugin: Box<dyn Plugin> = Box::new(plugin);
    Box::into_raw(boxed_plugin)
}

/// Legacy function for backward compatibility.
#[no_mangle]
pub extern "C" fn _create_provider() -> TraitObject {
    // Create a concrete instance of the provider
    let provider = DummyProvider;

    // Box it, so it has a stable address
    let boxed_provider: Box<dyn Provider> = Box::new(provider);

    // Transmute the fat pointer (Box<dyn Provider>) to our C-compatible TraitObject.
    // This is unsafe, but it's the core of the plugin interface.
    unsafe {
        mem::transmute(boxed_provider)
    }
}
