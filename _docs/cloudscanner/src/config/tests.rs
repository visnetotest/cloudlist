#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// Create a temporary config file with content
    fn create_temp_config(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file
    }

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
[[provider]]
id = "test-provider-1"
type = "aws"

[[provider]]
id = "test-provider-2"
type = "gcp"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.providers[0].id, "test-provider-1");
        assert_eq!(config.providers[0].provider_type, "aws");
        assert_eq!(config.providers[1].id, "test-provider-2");
        assert_eq!(config.providers[1].provider_type, "gcp");
    }

    #[test]
    fn test_load_empty_config() {
        let config_content = r#""#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.providers.len(), 0);
    }

    #[test]
    fn test_load_invalid_toml() {
        let config_content = r#"
[[provider]]
id = "test-provider"
type = "aws"
invalid_syntax_here
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        let error = config.unwrap_err();
        assert!(error.to_string().contains("Failed to parse config TOML"));
    }

    #[test]
    fn test_missing_config_file() {
        let config = super::load_config("/nonexistent/config.toml");
        
        assert!(config.is_err());
        let error = config.unwrap_err();
        assert!(error.to_string().contains("Config file not found"));
    }

    #[test]
    fn test_empty_provider_id() {
        let config_content = r#"
[[provider]]
id = ""
type = "aws"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        let error = config.unwrap_err();
        assert!(error.to_string().contains("empty ID"));
    }

    #[test]
    fn test_empty_provider_type() {
        let config_content = r#"
[[provider]]
id = "test-provider"
type = ""
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        let error = config.unwrap_err();
        assert!(error.to_string().contains("empty type"));
    }

    #[test]
    fn test_duplicate_provider_ids() {
        let config_content = r#"
[[provider]]
id = "duplicate-id"
type = "aws"

[[provider]]
id = "duplicate-id"
type = "gcp"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        // This should load successfully (validation doesn't check for duplicates)
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.providers.len(), 2);
        assert_eq!(config.providers[0].id, "duplicate-id");
        assert_eq!(config.providers[1].id, "duplicate-id");
    }

    #[test]
    fn test_config_with_whitespace() {
        let config_content = r#"
[[provider]]
id = "  test-provider  "
type = "  aws  "
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.providers[0].id, "  test-provider  ");
        assert_eq!(config.providers[0].provider_type, "  aws  ");
    }

    #[test]
    fn test_config_with_comments() {
        let config_content = r#"
# This is a comment
[[provider]]
id = "test-provider"  # Inline comment
type = "aws"

# Another comment
[[provider]]
id = "test-provider-2"
type = "gcp"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.providers.len(), 2);
    }

    #[test]
    fn test_config_error_messages() {
        let test_cases = vec![
            ("id = \"\"", "empty ID"),
            ("type = \"\"", "empty type"),
            ("invalid", "Failed to parse config TOML"),
        ];

        for (content, expected_error) in test_cases {
            let config_content = format!(r#"
 [[provider]]
 {}
 "#, content);
            
            let temp_file = create_temp_config(&config_content);
            let config = super::load_config(temp_file.path().to_str().unwrap());
            
            assert!(config.is_err(), "Should fail for content: {}", content);
            let error = config.unwrap_err();
            assert!(error.to_string().contains(expected_error), 
                "Error '{}' should contain '{}'", error.to_string(), expected_error);
        }
    }

    #[test]
    fn test_oversized_config_file() {
        let config_content = "[[provider]]\nid = \"test\"\ntype = \"aws\"\nconfig = ".to_string();
        let large_content = config_content + &"x".repeat(11 * 1024 * 1024); // 11MB
        
        let temp_file = create_temp_config(&large_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("exceeds maximum allowed"));
    }

    #[test]
    fn test_too_many_providers() {
        let mut config_content = String::new();
        for i in 0..101 { // Exceeds MAX_PROVIDERS (100)
            config_content.push_str(&format!(
                "[[provider]]\nid = \"provider-{}\"\ntype = \"aws\"\n\n", i
            ));
        }
        
        let temp_file = create_temp_config(&config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Number of providers"));
    }

    #[test]
    fn test_provider_id_too_long() {
        let long_id = "x".repeat(257); // Exceeds MAX_ID_LENGTH (256)
        let config_content = format!(r#"
[[provider]]
id = "{}"
type = "aws"
"#, long_id);
        
        let temp_file = create_temp_config(&config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Provider ID exceeds maximum length"));
    }

    #[test]
    fn test_provider_type_too_long() {
        let long_type = "x".repeat(101); // Exceeds MAX_TYPE_LENGTH (100)
        let config_content = format!(r#"
[[provider]]
id = "test-provider"
type = "{}"
"#, long_type);
        
        let temp_file = create_temp_config(&config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Provider type exceeds maximum length"));
    }

    #[test]
    fn test_dangerous_characters_in_id() {
        let dangerous_chars = ['\0', '\r', '\n', '\t', '<', '>', '|', '&', ';', '`', '$', '\\'];
        
        for dangerous_char in dangerous_chars {
            let config_content = format!(r#"
[[provider]]
id = "test{}provider"
type = "aws"
"#, dangerous_char);
            
            let temp_file = create_temp_config(&config_content);
            let config = super::load_config(temp_file.path().to_str().unwrap());
            
            assert!(config.is_err());
            assert!(config.unwrap_err().to_string().contains("dangerous character"));
        }
    }

    #[test]
    fn test_path_traversal_in_id() {
        let config_content = r#"
[[provider]]
id = "../../../etc/passwd"
type = "aws"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("dangerous path patterns"));
    }

    #[test]
    fn test_script_injection_patterns() {
        let script_patterns = ["javascript:", "data:", "eval(", "exec(", "system("];
        
        for pattern in script_patterns {
            let config_content = format!(r#"
[[provider]]
id = "test-provider"
type = "{}aws"
"#, pattern);
            
            let temp_file = create_temp_config(&config_content);
            let config = super::load_config(temp_file.path().to_str().unwrap());
            
            assert!(config.is_err());
            assert!(config.unwrap_err().to_string().contains("script pattern"));
        }
    }

    #[test]
    fn test_absolute_plugin_path() {
        let config_content = r#"
[[provider]]
id = "test-provider"
type = "aws"
plugin_path = "/etc/passwd"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("cannot be absolute path"));
    }

    #[test]
    fn test_dangerous_plugin_extensions() {
        let dangerous_extensions = [".exe", ".bat", ".cmd", ".com", ".pif", ".scr", ".vbs", ".js", ".jar", ".sh", ".ps1"];
        
        for ext in dangerous_extensions {
            let config_content = format!(r#"
[[provider]]
id = "test-provider"
type = "aws"
plugin_path = "plugin{}"
"#, ext);
            
            let temp_file = create_temp_config(&config_content);
            let config = super::load_config(temp_file.path().to_str().unwrap());
            
            assert!(config.is_err());
            assert!(config.unwrap_err().to_string().contains("dangerous extension"));
        }
    }

    #[test]
    fn test_oversized_config_data() {
        let large_config = "x".repeat(11 * 1024); // Exceeds MAX_CONFIG_SIZE (10KB)
        let config_content = format!(r#"
[[provider]]
id = "test-provider"
type = "aws"
config = {{ large_data = "{}" }}
"#, large_config);
        
        let temp_file = create_temp_config(&config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap());
        
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Config data exceeds maximum size"));
    }

    #[test]
    fn test_empty_path_input() {
        let config = super::load_config("");
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Configuration path cannot be empty"));
    }

    #[test]
    fn test_path_too_long() {
        let long_path = "/".to_string() + &"x".repeat(4097); // Exceeds MAX_PATH_LENGTH
        let config = super::load_config(&long_path);
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("Configuration path exceeds maximum length"));
    }

    #[test]
    fn test_path_traversal_in_path() {
        let config = super::load_config("../../../etc/passwd");
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("directory traversal"));
    }

    #[test]
    fn test_invalid_characters_in_path() {
        let invalid_chars = ['\0', '\r', '\n'];
        
        for invalid_char in invalid_chars {
            let path = format!("test{}config.toml", invalid_char);
            let config = super::load_config(&path);
            assert!(config.is_err());
            assert!(config.unwrap_err().to_string().contains("invalid characters"));
        }
    }

    #[test]
    fn test_valid_plugin_extensions() {
        let valid_extensions = [".so", ".dylib", ".dll"];
        
        for ext in valid_extensions {
            let config_content = format!(r#"
[[provider]]
id = "test-provider"
type = "aws"
plugin_path = "plugin{}"
"#, ext);
            
            let temp_file = create_temp_config(&config_content);
            let config = super::load_config(temp_file.path().to_str().unwrap());
            assert!(config.is_ok());
        }
    }

    #[test]
    fn test_minimal_valid_config() {
        let config_content = r#"
[[provider]]
id = "test"
type = "aws"
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert_eq!(config.providers[0].id, "test");
        assert_eq!(config.providers[0].provider_type, "aws");
        assert!(config.providers[0].plugin_path.is_none());
        assert!(config.providers[0].config.is_none());
    }

    #[test]
    fn test_whitespace_in_path() {
        let config_content = r#"
[[provider]]
id = "test"
type = "aws"
"#;
        
        let temp_file = create_temp_config(config_content);
        
        // Test with leading/trailing whitespace
        let path_with_whitespace = format!("  {}  ", temp_file.path().to_str().unwrap());
        let config = super::load_config(&path_with_whitespace);
        assert!(config.is_ok());
    }

    #[test]
    fn test_complex_yaml_config() {
        let config_content = r#"
[[provider]]
id = "complex-provider"
type = "aws"
plugin_path = "plugins/aws.so"
config = """
region: us-east-1
access_key: AKIAIOSFODNN7EXAMPLE
secret_key: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
services:
  - name: ec2
    enabled: true
    regions:
      - us-east-1
      - us-west-2
  - name: s3
    enabled: false
tags:
  environment: production
  team: security
"""
"#;
        
        let temp_file = create_temp_config(config_content);
        let config = super::load_config(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.providers.len(), 1);
        assert_eq!(config.providers[0].id, "complex-provider");
        assert!(config.providers[0].config.is_some());
    }
}