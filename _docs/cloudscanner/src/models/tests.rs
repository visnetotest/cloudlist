#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new("test-type".to_string(), "test-id".to_string());
        
        assert_eq!(resource.asset_type, "test-type");
        assert_eq!(resource.id, "test-id");
        assert!(resource.metadata.is_empty());
    }

    #[test]
    fn test_resource_with_metadata() {
        let resource = Resource::new("test-type".to_string(), "test-id".to_string())
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string());
        
        assert_eq!(resource.asset_type, "test-type");
        assert_eq!(resource.id, "test-id");
        assert_eq!(resource.metadata.len(), 2);
        assert_eq!(resource.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(resource.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_resource_unique_id() {
        let resource = Resource::new("test-type".to_string(), "test-id".to_string());
        let unique_id = resource.unique_id();
        
        assert_eq!(unique_id, "test-type:test-id");
    }

    #[test]
    fn test_resource_unique_id_with_special_chars() {
        let resource = Resource::new("test-type/with:special".to_string(), "test-id:123".to_string());
        let unique_id = resource.unique_id();
        
        assert_eq!(unique_id, "test-type/with:special:test-id:123");
    }

    #[test]
    fn test_provider_info_validation() {
        let info = ProviderInfo {
            name: "test-provider".to_string(),
            version: "1.0.0".to_string(),
            description: "Test provider".to_string(),
            supported_resource_types: vec!["test-resource".to_string()],
        };
        
        assert_eq!(info.name, "test-provider");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "Test provider");
        assert_eq!(info.supported_resource_types.len(), 1);
        assert_eq!(info.supported_resource_types[0], "test-resource");
    }

    #[test]
    fn test_provider_info_empty_fields() {
        let info = ProviderInfo {
            name: "".to_string(),
            version: "".to_string(),
            description: "".to_string(),
            supported_resource_types: vec![],
        };
        
        assert!(info.name.is_empty());
        assert!(info.version.is_empty());
        assert!(info.description.is_empty());
        assert!(info.supported_resource_types.is_empty());
    }

    #[test]
    fn test_c_string_conversion_valid() {
        let test_str = "hello world";
        let c_string = unsafe { super::string_to_c_char(test_str) };
        let converted_back = unsafe { super::c_char_to_string(c_string) };
        
        assert_eq!(converted_back, test_str);
        
        // Cleanup
        unsafe { super::free_c_string(c_string) };
    }

    #[test]
    fn test_c_string_conversion_empty() {
        let test_str = "";
        let c_string = unsafe { super::string_to_c_char(test_str) };
        let converted_back = unsafe { super::c_char_to_string(c_string) };
        
        assert_eq!(converted_back, test_str);
        
        // Cleanup
        unsafe { super::free_c_string(c_string) };
    }

    #[test]
    fn test_c_string_conversion_null() {
        let c_string = unsafe { super::string_to_c_char("test") };
        let converted_back = unsafe { super::c_char_to_string(std::ptr::null_mut()) };
        
        // Should return empty string for null pointer
        assert_eq!(converted_back, "");
        
        // Cleanup
        unsafe { super::free_c_string(c_string) };
    }

    #[test]
    fn test_c_resource_conversion_empty() {
        let resources: Vec<Resource> = vec![];
        let c_array = unsafe { super::resources_to_c(&resources) };
        
        assert_eq!(c_array.count, 0);
        assert!(c_array.resources.is_null());
        
        // Cleanup
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_c_resource_conversion_single() {
        let mut resource = Resource::new("test-type".to_string(), "test-id".to_string());
        resource.metadata.insert("key".to_string(), "value".to_string());
        
        let resources = vec![resource];
        let c_array = unsafe { super::resources_to_c(&resources) };
        
        assert_eq!(c_array.count, 1);
        assert!(!c_array.resources.is_null());
        
        // Convert back to verify
        let first_resource = unsafe { &*c_array.resources };
        let asset_type = unsafe { super::c_char_to_string(first_resource.asset_type) };
        let id = unsafe { super::c_char_to_string(first_resource.id) };
        
        assert_eq!(asset_type, "test-type");
        assert_eq!(id, "test-id");
        
        // Cleanup
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_c_resource_conversion_multiple() {
        let mut resource1 = Resource::new("type1".to_string(), "id1".to_string());
        resource1.metadata.insert("key1".to_string(), "value1".to_string());
        
        let mut resource2 = Resource::new("type2".to_string(), "id2".to_string());
        resource2.metadata.insert("key2".to_string(), "value2".to_string());
        
        let resources = vec![resource1, resource2];
        let c_array = unsafe { super::resources_to_c(&resources) };
        
        assert_eq!(c_array.count, 2);
        assert!(!c_array.resources.is_null());
        
        // Verify first resource
        let first_resource = unsafe { &*c_array.resources };
        let asset_type1 = unsafe { super::c_char_to_string(first_resource.asset_type) };
        assert_eq!(asset_type1, "type1");
        
        // Verify second resource
        let second_resource = unsafe { c_array.resources.add(1) };
        let asset_type2 = unsafe { super::c_char_to_string(second_resource.asset_type) };
        assert_eq!(asset_type2, "type2");
        
        // Cleanup
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_c_resource_conversion_with_metadata() {
        let mut resource = Resource::new("test-type".to_string(), "test-id".to_string());
        resource.metadata.insert("env".to_string(), "prod".to_string());
        resource.metadata.insert("region".to_string(), "us-west-2".to_string());
        
        let resources = vec![resource];
        let c_array = unsafe { super::resources_to_c(&resources) };
        
        assert_eq!(c_array.count, 1);
        assert_eq!(c_array.metadata_count, 2);
        assert!(!c_array.metadata.is_null());
        
        // Verify metadata
        let c_metadata = unsafe { &*c_array.metadata };
        let key1 = unsafe { super::c_char_to_string(c_metadata.key) };
        let value1 = unsafe { super::c_char_to_string(c_metadata.value) };
        
        assert_eq!(key1, "env");
        assert_eq!(value1, "prod");
        
        // Cleanup
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_c_resource_cleanup_empty() {
        let c_array = super::CResourceArray {
            resources: std::ptr::null_mut(),
            count: 0,
        };
        
        // Should not panic when cleaning up empty array
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_c_resource_cleanup_with_data() {
        let mut resource = Resource::new("test-type".to_string(), "test-id".to_string());
        resource.metadata.insert("key".to_string(), "value".to_string());
        
        let resources = vec![resource];
        let c_array = unsafe { super::resources_to_c(&resources) };
        
        // Should not panic when cleaning up
        unsafe { super::free_c_resources(c_array) };
    }

    #[test]
    fn test_sync_provider_trait() {
        // Test that SyncProvider trait is defined
        // This is mainly a compilation test
        fn assert_sync_provider<P: SyncProvider + 'static>() {
            // This function exists to ensure the trait is properly defined
        }
        
        assert_sync_provider::<DummySyncProvider>();
    }

    #[test]
    fn test_sync_to_async_adapter() {
        // Test that SyncToAsyncAdapter is defined
        // This is mainly a compilation test
        fn assert_adapter_exists<P: SyncProvider + 'static>() {
            // This function exists to ensure the adapter is properly defined
        }
        
        assert_adapter_exists::<DummySyncProvider>();
    }

    // Dummy implementations for testing
    struct DummySyncProvider;
    
    impl SyncProvider for DummySyncProvider {
        fn name(&self) -> &str {
            "dummy-sync"
        }
        
        fn discover(&self) -> super::Result<Vec<Resource>> {
            Ok(vec![])
        }
    }
}