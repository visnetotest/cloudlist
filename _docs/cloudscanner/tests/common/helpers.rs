use std::collections::HashMap;
use cloudscanner::models::provider::Resource;
use tempfile::NamedTempFile;
use std::io::Write;

/// Create a test resource with sample data
pub fn create_test_resource(asset_type: &str, id: &str) -> Resource {
    Resource::new(asset_type.to_string(), id.to_string())
        .with_metadata("test".to_string(), "value".to_string())
}

/// Create multiple test resources
pub fn create_test_resources(count: usize) -> Vec<Resource> {
    (0..count)
        .map(|i| create_test_resource("test-resource", &format!("test-id-{}", i)))
        .collect()
}

/// Create a temporary config file with content
pub fn create_temp_config(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file
}

/// Create a temporary policy file with content  
pub fn create_temp_policy(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file
}

/// Create test metadata
pub fn create_test_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    metadata
}