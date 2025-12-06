#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::provider::Resource;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;

    fn create_test_resource(asset_type: &str, id: &str) -> Resource {
        let mut resource = Resource::new(asset_type.to_string(), id.to_string());
        resource.metadata.insert("test".to_string(), "value".to_string());
        resource
    }

    fn create_temp_policy(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file
    }

    #[test]
    fn test_policy_engine_new() {
        let engine = PolicyEngine::new();
        assert_eq!(engine.get_violations_count(), 0);
        assert_eq!(engine.get_violations().len(), 0);
    }

    #[test]
    fn test_load_valid_policy() {
        let policy_content = r#"
name: "Test Policy"
asset_type: "test-resource"
rules:
  - key: "environment"
    value: "production"
  - key: "encrypted"
    value: "true"
"#;
        
        let temp_file = create_temp_policy(policy_content);
        let mut engine = PolicyEngine::new();
        let result = engine.load_policies(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(engine.get_violations_count(), 0);
    }

    #[test]
    fn test_load_empty_policy_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = PolicyEngine::new();
        let result = engine.load_policies(temp_dir.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(engine.get_violations_count(), 0);
    }

    #[test]
    fn test_load_nonexistent_policy_directory() {
        let mut engine = PolicyEngine::new();
        let result = engine.load_policies("/nonexistent/directory");
        
        assert!(result.is_ok()); // Should not fail, just warn
    }

    #[test]
    fn test_policy_evaluation_match() {
        let mut engine = PolicyEngine::new();
        
        // Create a test resource that matches policy
        let mut resource = create_test_resource("test-resource", "test-id-1");
        resource.metadata.insert("environment".to_string(), "production".to_string());
        resource.metadata.insert("encrypted".to_string(), "true".to_string());
        
        let resources = vec![resource];
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 0);
    }

    #[test]
    fn test_policy_evaluation_no_match() {
        let mut engine = PolicyEngine::new();
        
        // Create a test resource that doesn't match policy
        let mut resource = create_test_resource("test-resource", "test-id-1");
        resource.metadata.insert("environment".to_string(), "development".to_string());
        resource.metadata.insert("encrypted".to_string(), "false".to_string());
        
        let resources = vec![resource];
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 1);
        let violations = engine.get_violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].resource_id, "test-id-1");
        assert_eq!(violations[0].resource_type, "test-resource");
        assert_eq!(violations[0].violated_rules.len(), 2);
        assert!(violations[0].violated_rules[0].contains("environment"));
        assert!(violations[0].violated_rules[1].contains("encrypted"));
    }

    #[test]
    fn test_policy_evaluation_missing_metadata() {
        let mut engine = PolicyEngine::new();
        
        // Create a test resource with missing metadata
        let resource = create_test_resource("test-resource", "test-id-1");
        // No metadata added
        
        let resources = vec![resource];
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 1);
        let violations = engine.get_violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].resource_id, "test-id-1");
        assert_eq!(violations[0].violated_rules.len(), 2);
        assert!(violations[0].violated_rules[0].contains("missing required metadata"));
        assert!(violations[0].violated_rules[1].contains("missing required metadata"));
    }

    #[test]
    fn test_policy_evaluation_wrong_asset_type() {
        let mut engine = PolicyEngine::new();
        
        // Create a test resource with wrong asset type
        let resource = create_test_resource("different-resource", "test-id-1");
        
        let resources = vec![resource];
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 0); // Should not match policy
    }

    #[test]
    fn test_multiple_policy_evaluation() {
        let mut engine = PolicyEngine::new();
        
        // Create test resources
        let mut resource1 = create_test_resource("test-resource", "test-id-1");
        resource1.metadata.insert("environment".to_string(), "production".to_string());
        
        let mut resource2 = create_test_resource("test-resource", "test-id-2");
        resource2.metadata.insert("environment".to_string(), "development".to_string());
        
        let resources = vec![resource1, resource2];
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 1);
        let violations = engine.get_violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].resource_id, "test-id-2");
    }

    #[test]
    fn test_violation_reporting() {
        let mut engine = PolicyEngine::new();
        
        // Create a resource with multiple violations
        let mut resource = create_test_resource("test-resource", "test-id-1");
        resource.metadata.insert("environment".to_string(), "development".to_string());
        resource.metadata.insert("encrypted".to_string(), "false".to_string());
        
        let resources = vec![resource];
        engine.evaluate(&resources);
        
        let violations = engine.get_violations();
        assert_eq!(violations.len(), 1);
        
        let violation = &violations[0];
        assert_eq!(violation.resource_id, "test-id-1");
        assert_eq!(violation.resource_type, "test-resource");
        assert_eq!(violation.violated_rules.len(), 2);
        
        // Check violation messages
        let env_violation = violation.violated_rules.iter()
            .find(|rule| rule.contains("environment"))
            .unwrap_or(&"");
        assert!(env_violation.contains("expected 'production'"));
        assert!(env_violation.contains("found 'development'"));
        
        let enc_violation = violation.violated_rules.iter()
            .find(|rule| rule.contains("encrypted"))
            .unwrap_or(&"");
        assert!(enc_violation.contains("expected 'true'"));
        assert!(enc_violation.contains("found 'false'"));
    }

    #[test]
    fn test_policy_evaluation_with_empty_resources() {
        let mut engine = PolicyEngine::new();
        let resources: Vec<Resource> = vec![];
        
        engine.evaluate(&resources);
        
        assert_eq!(engine.get_violations_count(), 0);
        assert_eq!(engine.get_violations().len(), 0);
    }

    #[test]
    fn test_policy_violation_structure() {
        let violation = PolicyViolation {
            resource_id: "test-id".to_string(),
            resource_type: "test-resource".to_string(),
            policy_name: "Test Policy".to_string(),
            violated_rules: vec!["rule 1".to_string(), "rule 2".to_string()],
        };
        
        assert_eq!(violation.resource_id, "test-id");
        assert_eq!(violation.resource_type, "test-resource");
        assert_eq!(violation.policy_name, "Test Policy");
        assert_eq!(violation.violated_rules.len(), 2);
        assert_eq!(violation.violated_rules[0], "rule 1");
        assert_eq!(violation.violated_rules[1], "rule 2");
    }
}