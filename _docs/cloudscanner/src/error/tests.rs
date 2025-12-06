#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CloudScannerError;

    #[test]
    fn test_error_creation() {
        let error = CloudScannerError::config("Invalid configuration");
        assert!(matches!(error, CloudScannerError::Config { .. }));
        
        let error = CloudScannerError::provider("aws", "Connection failed");
        assert!(matches!(error, CloudScannerError::Provider { .. }));
        
        let error = CloudScannerError::security("Invalid input");
        assert!(matches!(error, CloudScannerError::Security { .. }));
    }

    #[test]
    fn test_error_sanitization() {
        let error = CloudScannerError::provider("aws", "AKIA1234567890123456");
        let sanitized = error.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("AKIA"));
        
        let error = CloudScannerError::provider("aws", "password=secret123");
        let sanitized = error.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("secret123"));
    }

    #[test]
    fn test_retryable_errors() {
        let network_error = CloudScannerError::network("ec2", "Connection timeout");
        assert!(network_error.is_retryable());

        let aws_error = CloudScannerError::aws("s3", "Service unavailable");
        assert!(aws_error.is_retryable());

        let auth_error = CloudScannerError::authentication("ec2", "Invalid credentials");
        assert!(!auth_error.is_retryable());

        let security_error = CloudScannerError::security("Invalid input");
        assert!(!security_error.is_retryable());
    }

    #[test]
    fn test_warning_level_errors() {
        let provider_error = CloudScannerError::provider("aws", "Connection failed");
        assert!(provider_error.is_warning_level());

        let plugin_error = CloudScannerError::plugin("test", "Load failed");
        assert!(plugin_error.is_warning_level());

        let security_error = CloudScannerError::security("Invalid input");
        assert!(!security_error.is_warning_level());

        let auth_error = CloudScannerError::authentication("ec2", "Invalid credentials");
        assert!(!auth_error.is_warning_level());
    }

    #[test]
    fn test_base64_detection() {
        // This should be detected as potential credential data
        let base64_like = "SGVsbG8gV29ybGQ="; // "Hello World" in base64
        let error = CloudScannerError::provider("test", base64_like);
        let sanitized = error.sanitize_for_logging();
        assert!(sanitized.contains("[REDACTED: Potential credential data]"));
    }

    #[test]
    fn test_context_error() {
        let original_error = CloudScannerError::aws("ec2", "Instance not found");
        let context_error = crate::error::utils::context_error(
            original_error,
            "discover",
            "ec2-provider"
        );
        
        match context_error {
            CloudScannerError::Aws { service, message } => {
                assert_eq!(service, "ec2-provider");
                assert!(message.contains("discover"));
                assert!(message.contains("Instance not found"));
            }
            _ => panic!("Expected Aws error"),
        }
    }
}