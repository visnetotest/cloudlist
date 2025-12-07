// cloudscanner/src/config/mod.rs

use serde::Deserialize;
use tracing::{info, error};
use std::path::Path;
use crate::error::{CloudScannerError, Result};

#[cfg(test)]
pub mod tests;

#[derive(Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub plugin_path: Option<String>,
    pub config: Option<serde_yaml::Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "provider")]
    pub providers: Vec<ProviderConfig>,
}

/// Alias for CloudScanner configuration to match expected interface
pub type CloudScannerConfig = Config;

pub fn load_config(path: &str) -> Result<Config> {
    info!("Loading configuration from: {}", path);
    
    // Validate and sanitize input path
    let sanitized_path = sanitize_path_input(path)?;
    
    let config_path = Path::new(&sanitized_path);
    if !config_path.exists() {
        return Err(CloudScannerError::config(format!("Configuration file not found: {}", sanitized_path)));
    }
    
    // Validate file size to prevent excessive memory usage
    let metadata = std::fs::metadata(config_path)?;
    const MAX_CONFIG_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > MAX_CONFIG_FILE_SIZE {
        return Err(CloudScannerError::config(
            format!("Configuration file size ({} bytes) exceeds maximum allowed ({} bytes)", 
                    metadata.len(), MAX_CONFIG_FILE_SIZE)
        ));
    }

    let content = std::fs::read_to_string(config_path)?;
    
    // Validate content size again after reading
    if content.len() > MAX_CONFIG_FILE_SIZE as usize {
        return Err(CloudScannerError::config(
            "Configuration content size exceeds maximum allowed"
        ));
    }
    
    let config: Config = toml::from_str(&content)?;
    
    // Validate configuration
    validate_config(&config)?;
    
    info!("Successfully loaded configuration with {} providers", config.providers.len());
    Ok(config)
}

/// Sanitize and validate path input
fn sanitize_path_input(path: &str) -> Result<String> {
    const MAX_PATH_LENGTH: usize = 4096;
    
    if path.is_empty() {
        return Err(CloudScannerError::validation("path", "Configuration path cannot be empty"));
    }
    
    if path.len() > MAX_PATH_LENGTH {
        return Err(CloudScannerError::validation(
            "path", 
            &format!("Configuration path exceeds maximum length of {} characters", MAX_PATH_LENGTH)
        ));
    }
    
    // Remove any surrounding whitespace
    let sanitized = path.trim();
    
    // Check for dangerous path patterns
    if sanitized.contains("..") {
        return Err(CloudScannerError::security(
            "Configuration path contains potentially dangerous directory traversal"
        ));
    }
    
    // Check for null bytes and other dangerous characters
    if sanitized.contains('\0') || sanitized.contains('\r') || sanitized.contains('\n') {
        return Err(CloudScannerError::security(
            "Configuration path contains invalid characters"
        ));
    }
    
    Ok(sanitized.to_string())
}

fn validate_config(config: &Config) -> Result<()> {
    const MAX_PROVIDERS: usize = 100;
    const MAX_ID_LENGTH: usize = 256;
    const MAX_TYPE_LENGTH: usize = 100;
    const MAX_CONFIG_SIZE: usize = 1024 * 1024; // 1MB
    
    // Validate overall configuration size
    if config.providers.len() > MAX_PROVIDERS {
        return Err(CloudScannerError::config(
            format!("Number of providers ({}) exceeds maximum allowed ({})", 
                    config.providers.len(), MAX_PROVIDERS)
        ));
    }
    
    for (i, provider) in config.providers.iter().enumerate() {
        // Validate provider ID
        if provider.id.is_empty() {
            return Err(CloudScannerError::validation(
                format!("provider[{}].id", i), 
                "Provider ID cannot be empty"
            ));
        }
        
        if provider.id.len() > MAX_ID_LENGTH {
            return Err(CloudScannerError::validation(
                format!("provider[{}].id", i), 
                &format!("Provider ID exceeds maximum length of {} characters", MAX_ID_LENGTH)
            ));
        }
        
        // Validate provider type
        if provider.provider_type.is_empty() {
            return Err(CloudScannerError::validation(
                format!("provider[{}].type", i), 
                "Provider type cannot be empty"
            ));
        }
        
        if provider.provider_type.len() > MAX_TYPE_LENGTH {
            return Err(CloudScannerError::validation(
                format!("provider[{}].type", i), 
                &format!("Provider type exceeds maximum length of {} characters", MAX_TYPE_LENGTH)
            ));
        }
        
        // Security validation for ID and type
        validate_string_security(&provider.id, "provider ID", &provider.id)?;
        validate_string_security(&provider.provider_type, "provider type", &provider.id)?;
        
        // Validate plugin path if present
        if let Some(ref plugin_path) = provider.plugin_path {
            validate_plugin_path(plugin_path, &provider.id)?;
        }
        
        // Validate configuration data if present
        if let Some(ref config_data) = provider.config {
            validate_config_data(config_data, &provider.id)?;
        }
    }
    
    Ok(())
}

/// Validate string for security issues
fn validate_string_security(value: &str, field_name: &str, provider_id: &str) -> Result<()> {
    // Check for dangerous characters that could lead to injection attacks
    const DANGEROUS_CHARS: &[char] = &['\0', '\r', '\n', '\t', '<', '>', '|', '&', ';', '`', '$', '\\'];
    
    for dangerous_char in DANGEROUS_CHARS {
        if value.contains(*dangerous_char) {
            return Err(CloudScannerError::security(
                format!("Provider '{}' {} contains dangerous character '{}'", 
                        provider_id, field_name, dangerous_char)
            ));
        }
    }
    
    // Check for path traversal patterns
    if value.contains("..") || value.contains("~") {
        return Err(CloudScannerError::security(
            format!("Provider '{}' {} contains potentially dangerous path patterns", 
                    provider_id, field_name)
        ));
    }
    
    // Check for script injection patterns
    const SCRIPT_PATTERNS: &[&str] = &[
        "javascript:", "data:", "vbscript:", "file:", "ftp:", "http:", "https:",
        "<script", "</script", "eval(", "exec(", "system("
    ];
    
    let value_lower = value.to_lowercase();
    for pattern in SCRIPT_PATTERNS {
        if value_lower.contains(pattern) {
            return Err(CloudScannerError::security(
                format!("Provider '{}' {} contains potentially dangerous script pattern", 
                        provider_id, field_name)
            ));
        }
    }
    
    Ok(())
}

/// Validate plugin path for security
fn validate_plugin_path(plugin_path: &str, provider_id: &str) -> Result<()> {
    const MAX_PATH_LENGTH: usize = 4096;
    
    if plugin_path.is_empty() {
        return Err(CloudScannerError::validation(
            format!("provider[{}].plugin_path", provider_id), 
            "Plugin path cannot be empty"
        ));
    }
    
    if plugin_path.len() > MAX_PATH_LENGTH {
        return Err(CloudScannerError::validation(
            format!("provider[{}].plugin_path", provider_id), 
            &format!("Plugin path exceeds maximum length of {} characters", MAX_PATH_LENGTH)
        ));
    }
    
    // Security validation for path
    validate_string_security(plugin_path, "plugin path", provider_id)?;
    
    // Check for absolute path attempts that could be dangerous
    if plugin_path.starts_with('/') || plugin_path.starts_with('\\') {
        return Err(CloudScannerError::security(
            format!("Provider '{}' plugin path cannot be absolute path", provider_id)
        ));
    }
    
    // Check for suspicious file extensions
    const DANGEROUS_EXTENSIONS: &[&str] = &[
        ".exe", ".bat", ".cmd", ".com", ".pif", ".scr", ".vbs", ".js", ".jar", ".sh", ".ps1"
    ];
    
    let path_lower = plugin_path.to_lowercase();
    for ext in DANGEROUS_EXTENSIONS {
        if path_lower.ends_with(ext) {
            return Err(CloudScannerError::security(
                format!("Provider '{}' plugin path has potentially dangerous extension: {}", 
                        provider_id, ext)
            ));
        }
    }
    
    Ok(())
}

/// Validate configuration data for security and size
fn validate_config_data(config_data: &serde_yaml::Value, provider_id: &str) -> Result<()> {
    const MAX_CONFIG_SIZE: usize = 10 * 1024; // 10KB per provider config
    
    // Serialize to string to check size
    let config_str = serde_yaml::to_string(config_data)?;
    
    if config_str.len() > MAX_CONFIG_SIZE {
        return Err(CloudScannerError::validation(
            format!("provider[{}].config", provider_id), 
            &format!("Config data exceeds maximum size of {} bytes", MAX_CONFIG_SIZE)
        ));
    }
    
    // Recursively validate YAML structure for security issues
    validate_yaml_value(config_data, provider_id, "config")?;
    
    Ok(())
}

/// Recursively validate YAML values for security issues
fn validate_yaml_value(value: &serde_yaml::Value, provider_id: &str, path: &str) -> Result<()> {
    match value {
        serde_yaml::Value::String(s) => {
            validate_string_security(s, path, provider_id)?;
        },
        serde_yaml::Value::Mapping(mapping) => {
            for (key, val) in mapping {
                if let serde_yaml::Value::String(key_str) = key {
                    validate_string_security(key_str, &format!("{} key", path), provider_id)?;
                    validate_yaml_value(val, provider_id, &format!("{}.{}", path, key_str))?;
                } else {
                    return Err(CloudScannerError::validation(
                        format!("provider[{}].{}", provider_id, path), 
                        "Mapping keys must be strings"
                    ));
                }
            }
        },
        serde_yaml::Value::Sequence(sequence) => {
            for (i, val) in sequence.iter().enumerate() {
                validate_yaml_value(val, provider_id, &format!("{}[{}]", path, i))?;
            }
        },
        _ => {
            // Numbers, booleans, null are generally safe
        }
    }
    
    Ok(())
}
