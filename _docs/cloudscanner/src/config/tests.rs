#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::helpers::*;
    use tempfile::NamedTempFile;

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
}