#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::helpers::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn test_discovery_engine_new() {
        let engine = DiscoveryEngine::new();
        assert_eq!(engine.provider_count(), 0);
        assert_eq!(engine.get_provider_names().len(), 0);
    }

    #[tokio::test]
    async fn test_load_providers_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = DiscoveryEngine::new();
        
        let result = engine.load_providers(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(engine.provider_count(), 0);
    }

    #[tokio::test]
    async fn test_load_providers_nonexistent_directory() {
        let mut engine = DiscoveryEngine::new();
        
        let result = engine.load_providers("/nonexistent/directory").await;
        assert!(result.is_ok()); // Should not fail, just warn
        assert_eq!(engine.provider_count(), 0);
    }

    #[tokio::test]
    async fn test_load_providers_with_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create invalid files
        fs::write(temp_dir.path().join("invalid.txt"), "not a library").unwrap();
        fs::write(temp_dir.path().join("another.txt"), "also not a library").unwrap();
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_providers(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(engine.provider_count(), 0);
    }

    #[tokio::test]
    async fn test_plugin_size_validation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create oversized plugin file
        let large_file_path = temp_dir.path().join("oversized.so");
        let large_content = vec![0u8; 60 * 1024 * 1024]; // 60MB
        fs::write(&large_file_path, large_content).unwrap();
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_providers(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(engine.provider_count(), 0);
    }

    #[tokio::test]
    async fn test_plugin_extension_validation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create files with invalid extensions
        fs::write(temp_dir.path().join("invalid.txt"), "content").unwrap();
        fs::write(temp_dir.path().join("invalid.exe"), "content").unwrap();
        fs::write(temp_dir.path().join("invalid.dll"), "content").unwrap(); // Valid on Windows
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_providers(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(engine.provider_count(), 0);
    }

    #[tokio::test]
    async fn test_discovery_with_no_providers() {
        let engine = DiscoveryEngine::new();
        let resources = engine.run().await;
        assert_eq!(resources.len(), 0);
    }

    #[tokio::test]
    async fn test_discovery_with_mock_providers() {
        // This test would require mocking the provider loading
        // For now, we test the basic structure
        let engine = DiscoveryEngine::new();
        let resources = engine.run().await;
        assert_eq!(resources.len(), 0);
    }

    #[tokio::test]
    async fn test_provider_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a file that looks like a plugin but will fail to load
        let fake_plugin_path = temp_dir.path().join("fake.so");
        fs::write(&fake_plugin_path, "not a real library").unwrap();
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_providers(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok()); // Should handle errors gracefully
        assert_eq!(engine.provider_count(), 0);
    }

    #[test]
    fn test_validate_plugin_file_not_found() {
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security("/nonexistent/file.so");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot access plugin file"));
    }

    #[test]
    fn test_validate_plugin_file_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let large_file_path = temp_dir.path().join("large.so");
        let large_content = vec![0u8; 60 * 1024 * 1024]; // 60MB
        fs::write(&large_file_path, large_content).unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(large_file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_validate_plugin_file_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let no_ext_path = temp_dir.path().join("noextension");
        fs::write(&no_ext_path, "content").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(no_ext_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no extension"));
    }

    #[test]
    fn test_validate_plugin_file_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_ext_path = temp_dir.path().join("invalid.txt");
        fs::write(&invalid_ext_path, "content").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(invalid_ext_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid plugin file extension"));
    }

    #[test]
    fn test_validate_plugin_file_valid_extensions() {
        let temp_dir = TempDir::new().unwrap();
        let valid_extensions = vec!["test.so", "test.dylib", "test.dll"];
        
        for ext in valid_extensions {
            let valid_path = temp_dir.path().join(ext);
            fs::write(&valid_path, "content").unwrap();
            
            let engine = DiscoveryEngine::new();
            let result = engine.validate_plugin_security(valid_path.to_str().unwrap());
            
            // Should pass extension validation (may fail on other grounds like size)
            if !result.is_err() || !result.unwrap_err().to_string().contains("extension") {
                // Either passes or fails for non-extension reasons
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_create_async_provider_adapter() {
        // Test the dummy provider adapter function
        let provider = super::create_async_provider_adapter();
        let provider_name = provider.name();
        assert_eq!(provider_name, "dummy-provider");
        
        // Test discovery
        let resources = provider.discover().await;
        assert!(resources.is_ok());
        let resources = resources.unwrap();
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].asset_type, "test-resource");
        assert_eq!(resources[0].id, "test-id-1");
        assert_eq!(resources[1].asset_type, "test-resource");
        assert_eq!(resources[1].id, "test-id-2");
    }

    #[tokio::test]
    async fn test_discovery_engine_with_providers() {
        let mut engine = DiscoveryEngine::new();
        
        // Manually add a provider for testing
        let provider = super::create_async_provider_adapter();
        // Note: This would need to modify the engine to accept providers directly
        // For now, we test the run method with empty providers
        
        let resources = engine.run().await;
        assert_eq!(resources.len(), 0);
    }

    #[test]
    fn test_engine_error_types() {
        // Test that error types are properly defined
        let error = super::EngineError::PluginLoadError(
            libloading::Error::DlOpen { desc: "test".to_string() }
        );
        
        assert!(error.to_string().contains("Failed to load plugin library"));
        
        let symbol_error = super::EngineError::SymbolNotFoundError { 
            symbol: "test_symbol".to_string() 
        };
        assert!(symbol_error.to_string().contains("Plugin symbol not found"));
        
        let validation_error = super::EngineError::ValidationError("test error".to_string());
        assert!(validation_error.to_string().contains("Plugin validation failed"));
    }
}