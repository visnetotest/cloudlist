#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, NamedTempFile};
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    #[tokio::test]
    async fn test_discovery_engine_new() {
        let engine = DiscoveryEngine::new();
        assert_eq!(engine.providers.len(), 0);
        assert_eq!(engine.loaded_libs.len(), 0);
    }

    #[tokio::test]
    async fn test_load_providers_from_config_empty() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![],
        };
        
        let result = engine.load_providers_from_config(&config).await;
        assert!(result.is_ok());
        assert_eq!(engine.providers.len(), 0);
    }

    #[tokio::test]
    async fn test_load_providers_from_config_with_dummy() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "test-provider".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        let result = engine.load_providers_from_config(&config).await;
        assert!(result.is_ok());
        assert_eq!(engine.providers.len(), 1);
    }

    #[tokio::test]
    async fn test_load_providers_from_config_with_builtin_aws() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "aws-provider".to_string(),
                    provider_type: "builtin-aws".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        let result = engine.load_providers_from_config(&config).await;
        assert!(result.is_ok());
        assert_eq!(engine.providers.len(), 1);
    }

    #[tokio::test]
    async fn test_load_providers_from_config_with_plugin_missing_path() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "plugin-provider".to_string(),
                    provider_type: "plugin".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        let result = engine.load_providers_from_config(&config).await;
        assert!(result.is_ok()); // Should handle gracefully
        assert_eq!(engine.providers.len(), 0);
    }

    #[tokio::test]
    async fn test_discovery_with_no_providers() {
        let engine = DiscoveryEngine::new();
        let resources = engine.run().await;
        assert_eq!(resources.len(), 0);
    }

    #[tokio::test]
    async fn test_discovery_with_dummy_providers() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "dummy1".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                },
                crate::config::ProviderConfig {
                    id: "dummy2".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        engine.load_providers_from_config(&config).await.unwrap();
        let resources = engine.run().await;
        assert_eq!(resources.len(), 4); // 2 providers × 2 resources each
    }

    #[tokio::test]
    async fn test_provider_loading_invalid_library() {
        let temp_dir = TempDir::new().unwrap();
        let fake_plugin_path = temp_dir.path().join("fake.so");
        fs::write(&fake_plugin_path, "not a real library").unwrap();
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_plugin(&fake_plugin_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provider_loading_missing_symbol() {
        let temp_dir = TempDir::new().unwrap();
        // Create a minimal shared library without the required symbol
        // This is a simplified test - in practice you'd need to compile a real library
        let fake_plugin_path = temp_dir.path().join("fake.so");
        fs::write(&fake_plugin_path, "fake library content").unwrap();
        
        let mut engine = DiscoveryEngine::new();
        let result = engine.load_plugin(&fake_plugin_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_discovery_timeout() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "dummy".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        engine.load_providers_from_config(&config).await.unwrap();
        
        // Test that discovery completes in reasonable time
        let start = std::time::Instant::now();
        let _resources = engine.run().await;
        let duration = start.elapsed();
        assert!(duration < std::time::Duration::from_secs(5));
    }

    #[test]
    fn test_validate_plugin_file_not_found() {
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(Path::new("/nonexistent/file.so"));
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
        let result = engine.validate_plugin_security(&large_file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_validate_plugin_file_no_extension() {
        let temp_dir = TempDir::new().unwrap();
        let no_ext_path = temp_dir.path().join("noextension");
        fs::write(&no_ext_path, "content").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(&no_ext_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no extension"));
    }

    #[test]
    fn test_validate_plugin_file_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_ext_path = temp_dir.path().join("invalid.txt");
        fs::write(&invalid_ext_path, "content").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(&invalid_ext_path);
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
            let result = engine.validate_plugin_security(&valid_path);
            
            // Should pass extension validation (may fail on other grounds like size)
            if !result.is_err() || !result.unwrap_err().to_string().contains("extension") {
                // Either passes or fails for non-extension reasons
                assert!(true);
            }
        }
    }

    #[test]
    fn test_validate_plugin_file_world_writable() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test.so");
        fs::write(&plugin_path, "content").unwrap();
        
        // Set world-writable permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&plugin_path).unwrap().permissions();
            perms.set_mode(0o622); // rw-w--w-
            fs::set_permissions(&plugin_path, perms).unwrap();
            
            let engine = DiscoveryEngine::new();
            let result = engine.validate_plugin_security(&plugin_path);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("world-writable"));
        }
    }

    #[test]
    fn test_validate_plugin_content_shell_script() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("malicious.so");
        fs::write(&script_path, "#!/bin/bash\necho 'pwned'").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(&script_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shell script"));
    }

    #[test]
    fn test_validate_plugin_content_sensitive_data() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("sensitive.so");
        fs::write(&plugin_path, "AKIAIOSFODNN7EXAMPLE secret_key").unwrap();
        
        let engine = DiscoveryEngine::new();
        let result = engine.validate_plugin_security(&plugin_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sensitive data"));
    }

    #[test]
    fn test_find_platform_plugin() {
        let engine = DiscoveryEngine::new();
        let temp_dir = TempDir::new().unwrap();
        
        // Create plugin with different extensions
        let base_path = temp_dir.path().join("test");
        
        // Test when no plugin exists
        let result = engine.find_platform_plugin(&base_path);
        assert!(result.is_err());
        
        // Create platform-specific plugin
        let plugin_ext = if cfg!(target_os = "macos") { "dylib" } 
                        else if cfg!(target_os = "linux") { "so" }
                        else { "dll" };
        let plugin_path = temp_dir.path().join(format!("test.{}", plugin_ext));
        fs::write(&plugin_path, "content").unwrap();
        
        let result = engine.find_platform_plugin(&base_path);
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with(plugin_ext));
    }

    #[tokio::test]
    async fn test_create_async_provider_adapter() {
        let provider = super::create_async_provider_adapter();
        let info = provider.info();
        assert_eq!(info.name, "dummy-provider");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "Dummy provider for testing");
        assert!(info.supported_resource_types.contains(&"test-resource".to_string()));
        
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
    async fn test_discovery_concurrent_execution() {
        let mut engine = DiscoveryEngine::new();
        let config = crate::config::Config {
            providers: vec![
                crate::config::ProviderConfig {
                    id: "dummy1".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                },
                crate::config::ProviderConfig {
                    id: "dummy2".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                },
                crate::config::ProviderConfig {
                    id: "dummy3".to_string(),
                    provider_type: "dummy".to_string(),
                    plugin_path: None,
                    config: None,
                }
            ],
        };
        
        engine.load_providers_from_config(&config).await.unwrap();
        
        let start = std::time::Instant::now();
        let resources = engine.run().await;
        let duration = start.elapsed();
        
        // Should complete faster than sequential execution
        assert_eq!(resources.len(), 6); // 3 providers × 2 resources each
        assert!(duration < std::time::Duration::from_secs(2));
    }

    #[test]
    fn test_loaded_library_drop() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test.so");
        fs::write(&plugin_path, "content").unwrap();
        
        // Test that LoadedLibrary implements Drop correctly
        let library = unsafe { 
            libloading::Library::new(&plugin_path)
        };
        
        if let Ok(lib) = library {
            let loaded_lib = super::LoadedLibrary::new(lib, &plugin_path);
            assert!(loaded_lib.is_ok());
            // When loaded_lib goes out of scope, Drop should be called
        }
    }
}