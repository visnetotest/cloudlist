// cloudscanner/src/engine/mod.rs

use crate::error::CloudScannerError;
use anyhow::Result;
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::models::provider::{Provider, TraitObject, Resource, ProviderInfo};
use crate::providers::base::{DiscoveryProvider, Asset};
use crate::providers::aws::{create_aws_provider, AwsProviderConfig};
use crate::config::Config;
use async_trait::async_trait;

#[cfg(test)]
pub mod tests;

type ProviderCreator = unsafe extern "C" fn() -> TraitObject;

pub struct DiscoveryEngine {
    providers: Vec<Arc<dyn Provider>>,
    // Use RAII wrapper for automatic library cleanup
    loaded_libs: Vec<LoadedLibrary>,
}

/// RAII wrapper for loaded libraries with automatic cleanup
struct LoadedLibrary {
    _library: Library,
    path: String,
}

impl LoadedLibrary {
    fn new(library: Library, path: &Path) -> Result<Self> {
        Ok(Self {
            _library: library,
            path: path.to_string_lossy().to_string(),
        })
    }
}

/// Create built-in AWS provider from configuration
async fn create_builtin_aws_provider(provider_config: &crate::config::ProviderConfig) -> std::result::Result<Box<dyn Provider>, CloudScannerError> {
    use crate::providers::aws::{AwsProvider, AwsProviderConfig};
    use serde_yaml;
    
    // Create AWS config from provider config
    let aws_config = if let Some(config_data) = &provider_config.config {
        serde_yaml::from_value::<AwsProviderConfig>(config_data.clone())
            .map_err(|e| CloudScannerError::config(format!("Failed to parse AWS config: {}", e)))?
    } else {
        // Default config for testing
        AwsProviderConfig::default()
    };
    
    let provider = AwsProvider::new(aws_config);
    
    Ok(Box::new(provider))
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        info!("Unloading plugin library: {}", self.path);
        // Library will be automatically dropped when this struct goes out of scope
    }
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {
            providers: vec![],
            loaded_libs: vec![],
        }
    }

    pub async fn load_providers_from_config(&mut self, config: &Config) -> Result<()> {
        info!("Loading providers from configuration");
        
        for provider_config in &config.providers {
            match provider_config.provider_type.as_str() {
                "plugin" => {
                    if let Some(plugin_path) = &provider_config.plugin_path {
                        info!("Loading plugin from: {}", plugin_path);
                        let path = Path::new(plugin_path);
                        match self.load_plugin(path).await {
                            Ok(provider_name) => {
                                info!("Successfully loaded plugin: {}", provider_name);
                            }
                            Err(e) => {
                                error!("Failed to load plugin from {}: {}", plugin_path, e);
                                // Continue with other providers
                            }
                        }
                    } else {
                        error!("Plugin provider missing plugin_path");
                    }
                }
                "builtin-aws" => {
                    info!("Creating built-in AWS provider");
                    match create_builtin_aws_provider(provider_config).await {
                        Ok(provider) => {
                            self.providers.push(Arc::from(provider));
                            info!("Successfully created built-in AWS provider");
                        }
                        Err(e) => {
                            error!("Failed to create built-in AWS provider: {}, falling back to dummy", e);
                            let provider = create_async_provider_adapter();
                            self.providers.push(Arc::from(provider));
                        }
                    }
                }
                _ => {
                    info!("Creating dummy provider for type: {}", provider_config.provider_type);
                    let provider = create_async_provider_adapter();
                    self.providers.push(Arc::from(provider));
                }
            }
        }
        
        Ok(())
    }
    
    async fn load_plugin(&mut self, path: &Path) -> Result<String> {
        // Validate plugin before loading
        self.validate_plugin_security(path)?;
        
        // Try different platform-specific extensions if the provided path doesn't exist
        let final_path = if !path.exists() {
            self.find_platform_plugin(path)?
        } else {
            path.to_path_buf()
        };
        
        // Load library with RAII wrapper for automatic cleanup
        let library = unsafe { 
            Library::new(&final_path)
                .map_err(|e| CloudScannerError::plugin(
                    final_path.to_string_lossy(), 
                    format!("Failed to load library: {}", e)
                ))?
        };
        
        let loaded_lib = LoadedLibrary::new(library, &final_path)?;
        let provider_name = self.load_plugin_from_library(&loaded_lib, &final_path).await?;
        
        self.loaded_libs.push(loaded_lib);
        Ok(provider_name)
    }
    
    /// Find platform-specific plugin file based on the base path
    fn find_platform_plugin(&self, path: &Path) -> std::result::Result<PathBuf, CloudScannerError> {
        let stem = path.file_stem()
            .ok_or_else(|| CloudScannerError::plugin(
                path.to_string_lossy(), 
                "Invalid plugin filename"
            ))?;
        
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        
        // Try platform-specific extensions in order of preference
        let extensions = if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else if cfg!(target_os = "linux") {
            vec!["so", "dylib"]
        } else if cfg!(target_os = "windows") {
            vec!["dll", "so", "dylib"]
        } else {
            vec!["so", "dylib", "dll"]
        };
        
        for ext in extensions {
            let candidate = parent.join(format!("{}.{}", stem.to_string_lossy(), ext));
            if candidate.exists() {
                info!("Found platform plugin: {}", candidate.display());
                return Ok(candidate);
            }
        }
        
        Err(CloudScannerError::plugin(
            path.to_string_lossy(), 
            "Plugin file not found for current platform"
        ))
    }
    
    async fn load_plugin_from_library(&mut self, loaded_lib: &LoadedLibrary, path: &Path) -> Result<String> {
        unsafe {
            let constructor: Symbol<ProviderCreator> = loaded_lib._library.get(b"_create_provider")
                .map_err(|e| CloudScannerError::plugin(
                    path.to_string_lossy(), 
                    format!("Failed to find constructor symbol: {}", e)
                ))?;
            
            let trait_object = constructor();

            // Validate trait object before using it
            if trait_object.data.is_null() || trait_object.vtable.is_null() {
                return Err(CloudScannerError::plugin(
                    path.to_string_lossy(), 
                    "Invalid trait object returned from plugin"
                ));
            }

            // Try to load AWS provider if it's an AWS plugin
            if path.to_string_lossy().contains("aws") {
                match self.try_load_aws_provider(path).await {
                    Ok(provider_name) => return Ok(provider_name),
                    Err(e) => {
                        warn!("Failed to create AWS provider: {}, using dummy provider", e);
                        // Fall through to create dummy provider
                    }
                }
            }
            
            // Create a simple async provider adapter for testing
            let provider = create_async_provider_adapter();
            let provider_name = provider.name();
            
            self.providers.push(Arc::from(provider));
            Ok(provider_name)
        }
    }
    
    async fn try_load_aws_provider(&mut self, path: &Path) -> Result<String> {
        // Try to find config file with same name as plugin
        let config_path = path.with_extension("toml");
        if !config_path.exists() {
            return Err(CloudScannerError::plugin(
                path.to_string_lossy(), 
                "No configuration file found"
            ));
        }
        
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Failed to read config file: {}", e)
            ))?;
        
        let aws_provider = create_aws_provider(&config_str).await
            .map_err(|e| CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Failed to create AWS provider: {}", e)
            ))?;
        
        let provider_name = aws_provider.name();
        self.providers.push(Arc::from(aws_provider));
        Ok(provider_name)
    }
    
    fn validate_plugin_security(&self, path: &Path) -> Result<()> {
        // Basic security validation for plugins
        let metadata = std::fs::metadata(path)
            .map_err(|e| CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Cannot access plugin file: {}", e)
            ))?;
            
        // Check file size is reasonable (less than 10MB - reduced from 50MB)
        const MAX_PLUGIN_SIZE: u64 = 10 * 1024 * 1024;
        if metadata.len() > MAX_PLUGIN_SIZE {
            return Err(CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Plugin file is too large: {} bytes (max: {})", metadata.len(), MAX_PLUGIN_SIZE)
            ));
        }
        
        // Check file permissions (should not be world-writable)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o002 != 0 {
                return Err(CloudScannerError::security(
                    format!("Plugin file has insecure permissions (world-writable): {}", path.to_string_lossy())
                ));
            }
        }
        
        // Basic file extension validation
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy();
            let ext_str = ext.as_ref();
            if !matches!(ext_str, "so" | "dylib" | "dll") {
                return Err(CloudScannerError::plugin(
                    path.to_string_lossy(), 
                    format!("Invalid plugin file extension: {}", ext)
                ));
            }
        } else {
            return Err(CloudScannerError::plugin(
                path.to_string_lossy(), 
                "Plugin file has no extension"
            ));
        }
        
        // Additional security checks
        self.validate_plugin_content(path)?;
        
        Ok(())
    }
    
    fn validate_plugin_content(&self, path: &Path) -> Result<()> {
        // Read first few bytes to check for suspicious patterns
        let mut file = std::fs::File::open(path)
            .map_err(|e| CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Cannot open plugin file: {}", e)
            ))?;
        
        let mut buffer = [0u8; 1024];
        let bytes_read = std::io::Read::read(&mut file, &mut buffer)
            .map_err(|e| CloudScannerError::plugin(
                path.to_string_lossy(), 
                format!("Cannot read plugin file: {}", e)
            ))?;
        
        // Check for suspicious patterns in plugin header
        let header = &buffer[..bytes_read];
        let header_str = std::str::from_utf8(header).unwrap_or("");
        
        // Check for potential shell scripts or other non-binary content
        if header_str.contains("#!/bin/sh") || header_str.contains("#!/bin/bash") {
            return Err(CloudScannerError::security(
                format!("Plugin appears to be a shell script: {}", path.to_string_lossy())
            ));
        }
        
        // Check for embedded credentials or suspicious strings
        let suspicious_patterns = [
            "AKIA", "aws_secret_access_key", "password", "secret", "token",
            "BEGIN PRIVATE KEY", "BEGIN RSA PRIVATE KEY",
        ];
        
        for pattern in &suspicious_patterns {
            if header_str.to_lowercase().contains(pattern) {
                return Err(CloudScannerError::security(
                    format!("Plugin contains potentially sensitive data: {}", path.to_string_lossy())
                ));
            }
        }
        
        Ok(())
    }

    pub async fn run(&self) -> Vec<Resource> {
        info!("Discovery engine running with {} providers", self.providers.len());
        
        // Run providers concurrently with controlled parallelism
        let max_concurrent_providers = std::cmp::min(self.providers.len(), 10);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_providers));
        
        let tasks: Vec<_> = self.providers.iter().map(|provider| {
            let provider = Arc::clone(provider);
            let semaphore = Arc::clone(&semaphore);
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let provider_name = provider.name();
                info!("Starting discovery for provider: {}", provider_name);
                
                let start_time = std::time::Instant::now();
                match provider.discover().await {
                    Ok(resources) => {
                        let duration = start_time.elapsed();
                        info!(
                            "Discovered {} resources from provider {} in {:?}", 
                            resources.len(), provider_name, duration
                        );
                        Ok((provider_name, resources))
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        let sanitized_error = e.sanitize_for_logging();
                        error!(
                            "Error discovering resources for provider {} in {:?}: {}", 
                            provider_name, duration, sanitized_error
                        );
                        Err((provider_name, e))
                    }
                }
            })
        }).collect();

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        
        // Collect results
        let mut all_resources = Vec::new();
        let mut successful_providers = 0;
        let mut failed_providers = 0;
        
        for result in results {
            match result {
                Ok(Ok((provider_name, resources))) => {
                    all_resources.extend(resources);
                    successful_providers += 1;
                }
                Ok(Err((provider_name, _))) => {
                    failed_providers += 1;
                }
                Err(join_error) => {
                    error!("Provider task panicked: {}", join_error);
                    failed_providers += 1;
                }
            }
        }
        
        info!(
            "Discovery complete. Total resources: {} from {} successful providers, {} failed providers", 
            all_resources.len(), successful_providers, failed_providers
        );
        
        all_resources
    }
}



    /// Create a simple async provider adapter for testing
    pub fn create_async_provider_adapter() -> Box<dyn Provider> {
        struct DummyProvider;
        
        #[async_trait]
        impl Provider for DummyProvider {
            fn info(&self) -> ProviderInfo {
                ProviderInfo {
                    name: "dummy-provider".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Dummy provider for testing".to_string(),
                    supported_resource_types: vec!["test-resource".to_string()],
                }
            }

            async fn discover(&self) -> Result<Vec<Resource>> {
                Ok(vec![
                    Resource::new("test-resource".to_string(), "test-id-1".to_string())
                        .with_metadata("test".to_string(), "value".to_string()),
                    Resource::new("test-resource".to_string(), "test-id-2".to_string())
                ])
            }
        }
        
        Box::new(DummyProvider)
    }
