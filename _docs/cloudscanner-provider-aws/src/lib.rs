// cloudscanner-provider-aws/src/lib.rs

use anyhow::Result;
use async_trait::async_trait;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::sync::Arc;
use tracing::{info, error, debug};

use cloudscanner::models::provider::{Provider, Resource, ProviderInfo};
use cloudscanner::providers::aws::{AwsProvider, AwsConfig};

/// AWS Provider Plugin
pub struct AwsPlugin {
    provider: Option<AwsProvider>,
}

#[no_mangle]
pub extern "C" fn _create_provider() -> *mut std::ffi::c_void {
    info!("AWS plugin _create_provider called");
    
    // Create a dummy provider for now
    let provider = Box::new(42u32); // Dummy data
    Box::into_raw(provider) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn _destroy_provider(provider: *mut std::ffi::c_void) {
    info!("AWS plugin _destroy_provider called");
    
    unsafe {
        if !provider.is_null() {
            let _ = Box::from_raw(provider as *mut u8);
        }
    }
}

/// Convert AWS config to JSON string for the plugin
fn aws_config_to_json(config: &AwsConfig) -> Result<String> {
    let config_json = serde_json::to_string(&serde_json::json!({
        "access_key_id": config.access_key_id,
        "secret_access_key": config.secret_access_key,
        "session_token": config.session_token,
        "profile": config.profile,
        "role_arn": config.role_arn,
        "external_id": config.external_id,
        "endpoint_url": config.endpoint_url,
        "use_ssl": config.use_ssl,
        "region": config.region,
        "services": config.services
    }))?;
    
    Ok(config_json)
}

/// Create AWS provider from JSON configuration
fn create_aws_provider_from_json(config_json: &str) -> Result<Box<dyn Provider>> {
    debug!("Creating AWS provider from JSON: {}", config_json);
    
    let aws_config: AwsConfig = serde_json::from_str(config_json)?;
    let provider = AwsProvider::new(aws_config)?;
    
    Ok(Box::new(provider))
}

/// Plugin creator function type
type ProviderCreator = unsafe extern "C" fn() -> *mut std::ffi::c_void;

/// Create AWS provider plugin
#[no_mangle]
pub extern "C" fn create_provider(config_json: *const i8) -> *mut std::ffi::c_void {
    info!("AWS plugin create_provider called with config: {}", unsafe {
        CStr::from_ptr(config_json).to_string_lossy()
    });
    
    // Create AWS provider from JSON config
    let config_str = unsafe {
        CStr::from_ptr(config_json).to_string_lossy()
    };
    
    match create_aws_provider_from_json(&config_str) {
        Ok(provider) => {
            info!("Successfully created AWS provider plugin");
            // Convert Box to raw pointer
            let raw_provider = Box::into_raw(provider);
            raw_provider as *mut std::ffi::c_void
        }
        Err(e) => {
            error!("Failed to create AWS provider: {}", e);
            std::ptr::null_mut()
        }
    }
}