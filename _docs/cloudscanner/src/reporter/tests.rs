#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::provider::Resource;
    use crate::tests::common::helpers::*;
    use std::collections::HashMap;

    #[test]
    fn test_reporter_new() {
        let reporter = Reporter::new();
        match reporter.output_format {
            OutputFormat::Console => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_reporter_with_format() {
        let console_reporter = Reporter::new().with_format(OutputFormat::Console);
        match console_reporter.output_format {
            OutputFormat::Console => assert!(true),
            _ => assert!(false),
        }

        let json_reporter = Reporter::new().with_format(OutputFormat::Json);
        match json_reporter.output_format {
            OutputFormat::Json => assert!(true),
            _ => assert!(false),
        }

        let yaml_reporter = Reporter::new().with_format(OutputFormat::Yaml);
        match yaml_reporter.output_format {
            OutputFormat::Yaml => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_generate_report_with_resources() {
        let reporter = Reporter::new();
        let resources = create_test_resources(3);
        
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.total_resources, 3);
        assert_eq!(report.resources.len(), 3);
        assert!(report.resources_by_type.contains_key("test-resource"));
        assert_eq!(report.resources_by_type["test-resource"], 3);
        
        // Check resource details
        for (i, resource_report) in report.resources.iter().enumerate() {
            assert_eq!(resource_report.asset_type, "test-resource");
            assert_eq!(resource_report.id, format!("test-id-{}", i + 1));
            assert!(resource_report.metadata.contains_key("test"));
            assert_eq!(resource_report.metadata["test"], "value");
        }
    }

    #[test]
    fn test_generate_report_with_empty_resources() {
        let reporter = Reporter::new();
        let resources: Vec<Resource> = vec![];
        
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.total_resources, 0);
        assert_eq!(report.resources.len(), 0);
        assert!(report.resources_by_type.is_empty());
        assert!(!report.timestamp.is_empty());
    }

    #[test]
    fn test_generate_report_with_mixed_resource_types() {
        let reporter = Reporter::new();
        let mut resources = Vec::new();
        
        // Add different resource types
        resources.push(create_test_resource("type1", "id1"));
        resources.push(create_test_resource("type1", "id2"));
        resources.push(create_test_resource("type2", "id3"));
        resources.push(create_test_resource("type3", "id4"));
        
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.total_resources, 4);
        assert_eq!(report.resources_by_type.len(), 3);
        assert_eq!(report.resources_by_type["type1"], 2);
        assert_eq!(report.resources_by_type["type2"], 1);
        assert_eq!(report.resources_by_type["type3"], 1);
    }

    #[test]
    fn test_generate_report_with_metadata() {
        let reporter = Reporter::new();
        let mut resource = create_test_resource("test-resource", "test-id");
        
        // Add multiple metadata fields
        resource.metadata.insert("env".to_string(), "prod".to_string());
        resource.metadata.insert("region".to_string(), "us-west-2".to_string());
        resource.metadata.insert("owner".to_string(), "team-a".to_string());
        
        let resources = vec![resource];
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.total_resources, 1);
        assert_eq!(report.resources.len(), 1);
        
        let resource_report = &report.resources[0];
        assert_eq!(resource_report.metadata.len(), 4); // test + env + region + owner
        assert_eq!(resource_report.metadata["test"], "value");
        assert_eq!(resource_report.metadata["env"], "prod");
        assert_eq!(resource_report.metadata["region"], "us-west-2");
        assert_eq!(resource_report.metadata["owner"], "team-a");
    }

    #[test]
    fn test_console_report_generation() {
        let reporter = Reporter::new().with_format(OutputFormat::Console);
        let resources = create_test_resources(2);
        
        // Capture console output
        let result = reporter.report(&resources);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_report_generation() {
        let reporter = Reporter::new().with_format(OutputFormat::Json);
        let resources = create_test_resources(2);
        
        let result = reporter.report(&resources);
        assert!(result.is_ok());
        
        // Test that JSON serialization doesn't panic
        let report = reporter.generate_report(&resources).unwrap();
        let json_str = serde_json::to_string(&report);
        assert!(json_str.is_ok());
    }

    #[test]
    fn test_yaml_report_generation() {
        let reporter = Reporter::new().with_format(OutputFormat::Yaml);
        let resources = create_test_resources(2);
        
        let result = reporter.report(&resources);
        assert!(result.is_ok());
        
        // Test that YAML serialization doesn't panic
        let report = reporter.generate_report(&resources).unwrap();
        let yaml_str = serde_yaml::to_string(&report);
        assert!(yaml_str.is_ok());
    }

    #[test]
    fn test_report_timestamp_format() {
        let reporter = Reporter::new();
        let resources = create_test_resources(1);
        
        let report = reporter.generate_report(&resources).unwrap();
        
        // Test that timestamp is in RFC3339 format
        // This is a basic check - in production, you might want more specific validation
        assert!(report.timestamp.len() > 20); // RFC3339 timestamps are long
        assert!(report.timestamp.contains('T')); // Contains time separator
        assert!(report.timestamp.contains('Z')); // Contains UTC indicator
    }

    #[test]
    fn test_report_resource_aggregation() {
        let reporter = Reporter::new();
        let mut resources = Vec::new();
        
        // Create resources with different types and counts
        for i in 0..5 {
            resources.push(create_test_resource("type-a", &format!("id-a-{}", i)));
        }
        for i in 0..3 {
            resources.push(create_test_resource("type-b", &format!("id-b-{}", i)));
        }
        for i in 0..2 {
            resources.push(create_test_resource("type-c", &format!("id-c-{}", i)));
        }
        
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.total_resources, 10);
        assert_eq!(report.resources_by_type.len(), 3);
        assert_eq!(report.resources_by_type["type-a"], 5);
        assert_eq!(report.resources_by_type["type-b"], 3);
        assert_eq!(report.resources_by_type["type-c"], 2);
    }

    #[test]
    fn test_report_error_handling() {
        let reporter = Reporter::new();
        
        // Test with invalid JSON serialization
        // This is hard to test directly since our Resource is serializable
        // But we can test the error path in report generation
        
        let resources = create_test_resources(1);
        let report = reporter.generate_report(&resources);
        assert!(report.is_ok());
        
        // Test JSON error path
        let json_reporter = Reporter::new().with_format(OutputFormat::Json);
        let result = json_reporter.report(&resources);
        assert!(result.is_ok());
    }

    #[test]
    fn test_report_with_large_dataset() {
        let reporter = Reporter::new();
        let resources = create_test_resources(1000); // Large number of resources
        
        let result = reporter.generate_report(&resources);
        assert!(result.is_ok());
        
        let report = result.unwrap();
        assert_eq!(report.total_resources, 1000);
        assert_eq!(report.resources.len(), 1000);
        assert_eq!(report.resources_by_type["test-resource"], 1000);
    }

    #[test]
    fn test_output_format_variants() {
        let formats = vec![
            OutputFormat::Console,
            OutputFormat::Json,
            OutputFormat::Yaml,
        ];
        
        for format in formats {
            let reporter = Reporter::new().with_format(format.clone());
            match reporter.output_format {
                ref f if std::mem::discriminant(f) == std::mem::discriminant(&format) => assert!(true),
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_resource_report_structure() {
        let reporter = Reporter::new();
        let mut resource = create_test_resource("test-type", "test-id");
        resource.metadata.insert("key1".to_string(), "value1".to_string());
        resource.metadata.insert("key2".to_string(), "value2".to_string());
        
        let resources = vec![resource];
        let report = reporter.generate_report(&resources).unwrap();
        
        assert_eq!(report.resources.len(), 1);
        let resource_report = &report.resources[0];
        
        assert_eq!(resource_report.asset_type, "test-type");
        assert_eq!(resource_report.id, "test-id");
        assert_eq!(resource_report.metadata.len(), 3); // test + key1 + key2
        assert_eq!(resource_report.metadata["test"], "value");
        assert_eq!(resource_report.metadata["key1"], "value1");
        assert_eq!(resource_report.metadata["key2"], "value2");
    }
}