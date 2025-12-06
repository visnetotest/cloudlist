// cloudscanner/src/error/mod.rs

use thiserror::Error;

/// Common error types for cloudscanner
#[derive(Error, Debug)]
pub enum CloudScannerError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },

    #[error("Plugin error: {plugin} - {message}")]
    Plugin { plugin: String, message: String },

    #[error("Security error: {message}")]
    Security { message: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("I/O error: {message}")]
    Io { 
        #[from]
        source: std::io::Error 
    },

    #[error("Serialization error: {message}")]
    Serialization { 
        #[from]
        source: serde_yaml::Error 
    },

    #[error("TOML parsing error: {message}")]
    TomlParsing { 
        #[from]
        source: toml::de::Error 
    },

    #[error("AWS SDK error: {service} - {message}")]
    Aws { service: String, message: String },

    #[error("Memory allocation error: {message}")]
    Memory { message: String },

    #[error("Resource not found: {resource_type} - {id}")]
    ResourceNotFound { resource_type: String, id: String },

    #[error("Authentication error: {service} - {message}")]
    Authentication { service: String, message: String },

    #[error("Network error: {service} - {message}")]
    Network { service: String, message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl CloudScannerError {
    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config { message: message.into() }
    }

    /// Create a provider error
    pub fn provider(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider { 
            provider: provider.into(), 
            message: message.into() 
        }
    }

    /// Create a plugin error
    pub fn plugin(plugin: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Plugin { 
            plugin: plugin.into(), 
            message: message.into() 
        }
    }

    /// Create a security error
    pub fn security(message: impl Into<String>) -> Self {
        Self::Security { message: message.into() }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation { 
            field: field.into(), 
            message: message.into() 
        }
    }

    /// Create an AWS error
    pub fn aws(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Aws { 
            service: service.into(), 
            message: message.into() 
        }
    }

    /// Create a memory error
    pub fn memory(message: impl Into<String>) -> Self {
        Self::Memory { message: message.into() }
    }

    /// Create a resource not found error
    pub fn resource_not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::ResourceNotFound { 
            resource_type: resource_type.into(), 
            id: id.into() 
        }
    }

    /// Create an authentication error
    pub fn authentication(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Authentication { 
            service: service.into(), 
            message: message.into() 
        }
    }

    /// Create a network error
    pub fn network(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Network { 
            service: service.into(), 
            message: message.into() 
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. } | Self::Aws { .. } => true,
            Self::Authentication { .. } | Self::Security { .. } => false,
            _ => false,
        }
    }

    /// Check if this error should be logged at WARN level instead of ERROR
    pub fn is_warning_level(&self) -> bool {
        match self {
            Self::Provider { .. } | Self::Plugin { .. } => true,
            Self::Security { .. } | Self::Authentication { .. } => false,
            _ => false,
        }
    }

    /// Sanitize error message for logging (remove sensitive data)
    pub fn sanitize_for_logging(&self) -> String {
        let message = self.to_string();
        
        // Remove potential sensitive patterns
        let sanitized = message
            .replace("AKIA", "[REDACTED]")
            .replace("aws_secret_access_key", "[REDACTED]")
            .replace("AWS_ACCESS_KEY_ID", "[REDACTED]")
            .replace("AWS_SECRET_ACCESS_KEY", "[REDACTED]")
            .replace("password", "[REDACTED]")
            .replace("token", "[REDACTED]")
            .replace("secret", "[REDACTED]");

        // Also check for base64-like patterns that might be credentials
        if sanitized.len() > 50 && sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
            return "[REDACTED: Potential credential data]".to_string();
        }

        sanitized
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, CloudScannerError>;

/// Error handling utilities
pub mod utils {
    use super::*;

    /// Convert any error to CloudScannerError
    pub fn convert_error<E: std::error::Error + Send + Sync + 'static>(
        error: E,
        context: &str,
    ) -> CloudScannerError {
        CloudScannerError::internal(format!("{}: {}", context, error))
    }

    /// Create a context-aware error
    pub fn context_error(
        error: CloudScannerError,
        operation: &str,
        resource: &str,
    ) -> CloudScannerError {
        match error {
            CloudScannerError::Provider { message, .. } => CloudScannerError::Provider {
                provider: resource.to_string(),
                message: format!("{}: {}", operation, message),
            },
            CloudScannerError::Aws { message, .. } => CloudScannerError::Aws {
                service: resource.to_string(),
                message: format!("{}: {}", operation, message),
            },
            _ => CloudScannerError::Internal {
                message: format!("{} on {}: {}", operation, resource, error),
            },
        }
    }

    /// Handle errors with appropriate logging level
    pub fn handle_error(error: &CloudScannerError, context: &str) {
        let sanitized_message = error.sanitize_for_logging();
        
        if error.is_warning_level() {
            tracing::warn!("{}: {}", context, sanitized_message);
        } else {
            tracing::error!("{}: {}", context, sanitized_message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CloudScannerError::config("Invalid configuration");
        assert!(matches!(error, CloudScannerError::Config { .. }));
    }

    #[test]
    fn test_error_sanitization() {
        let error = CloudScannerError::provider("aws", "AKIA1234567890123456");
        let sanitized = error.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("AKIA"));
    }

    #[test]
    fn test_retryable_errors() {
        let network_error = CloudScannerError::network("ec2", "Connection timeout");
        assert!(network_error.is_retryable());

        let auth_error = CloudScannerError::authentication("ec2", "Invalid credentials");
        assert!(!auth_error.is_retryable());
    }
}