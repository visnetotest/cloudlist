use cloudscanner::config;
use cloudscanner::engine;
use cloudscanner::policy;
use cloudscanner::reporter;
use std::path::Path;

#[tokio::test]
async fn test_full_pipeline_smoke_test() {
    // Test that basic pipeline components can be created
    let config_path = Path::new("test_config.toml");
    let plugins_path = Path::new("test_plugins");
    let policies_path = Path::new("test_policies");
    
    // Test component creation
    let engine = engine::DiscoveryEngine::new();
    let policy_engine = policy::PolicyEngine::new();
    let reporter = reporter::Reporter::new();
    
    // Test that components are created successfully
    assert_eq!(engine.provider_count(), 0);
    assert_eq!(policy_engine.get_violations_count(), 0);
    
    // Test basic functionality
    let resources = engine.run().await;
    assert_eq!(resources.len(), 0); // No providers loaded
    
    // Test report generation
    let result = reporter.report(&resources);
    assert!(result.is_ok());
}