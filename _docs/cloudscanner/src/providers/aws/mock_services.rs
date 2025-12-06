use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::models::provider::Resource;
use super::config::AwsConfig;
use super::ec2::Ec2Discovery;
use super::s3::S3Discovery;

/// Mock EC2 Client for testing
pub struct MockEc2Client {
    instances: Vec<aws_sdk_ec2::types::Instance>,
}

impl MockEc2Client {
    pub fn new() -> Self {
        Self {
            instances: vec![
                // Create mock instances for testing
                aws_sdk_ec2::types::Instance::builder()
                    .instance_id("i-1234567890abcdef0".to_string())
                    .instance_type(aws_sdk_ec2::types::InstanceType::T3Micro)
                    .state(
                        aws_sdk_ec2::types::InstanceState::builder()
                            .name(aws_sdk_ec2::types::InstanceStateName::Running)
                            .build()
                    )
                    .image_id("ami-1234567890abcdef".to_string())
                    .key_name("test-key".to_string())
                    .subnet_id("subnet-12345678".to_string())
                    .vpc_id("vpc-12345678".to_string())
                    .security_groups(
                        aws_sdk_ec2::types::GroupIdentifier::builder()
                            .group_id("sg-12345678".to_string())
                            .group_name("test-sg".to_string())
                            .build()
                    )
                    .tags(
                        aws_sdk_ec2::types::Tag::builder()
                            .key("Environment".to_string())
                            .value("test".to_string())
                            .build()
                    )
                    .tags(
                        aws_sdk_ec2::types::Tag::builder()
                            .key("Owner".to_string())
                            .value("test-team".to_string())
                            .build()
                    )
                    .public_ip_address("203.0.113.1".to_string())
                    .private_ip_address("10.0.0.1".to_string())
                    .launch_time(aws_sdk_ec2::primitives::DateTime::from_secs(1609459200i64))
                    .build(),
                aws_sdk_ec2::types::Instance::builder()
                    .instance_id("i-0987654321fedcbae".to_string())
                    .instance_type(aws_sdk_ec2::types::InstanceType::T2Small)
                    .state(
                        aws_sdk_ec2::types::InstanceState::builder()
                            .name(aws_sdk_ec2::types::InstanceStateName::Running)
                            .build()
                    )
                    .image_id("ami-0987654321fedcba".to_string())
                    .key_name("test-key-2".to_string())
                    .subnet_id("subnet-87654321".to_string())
                    .vpc_id("vpc-87654321".to_string())
                    .tags(
                        aws_sdk_ec2::types::Tag::builder()
                            .key("Environment".to_string())
                            .value("production".to_string())
                            .build()
                    )
                    .public_ip_address("203.0.113.2".to_string())
                    .private_ip_address("10.0.0.2".to_string())
                    .launch_time(aws_sdk_ec2::primitives::DateTime::from_secs(1609459300i64))
                    .build(),
            ],
        }
    }
}

#[async_trait]
impl Ec2Discovery for MockEc2Client {
    async fn discover_instances(&self) -> Result<Vec<Resource>> {
        info!("Mock EC2 discovery returning {} instances", self.instances.len());
        
        let mut resources = Vec::new();
        for instance in &self.instances {
            if let Some(resource) = self.instance_to_resource(instance) {
                resources.push(resource);
            }
        }
        
        Ok(resources)
    }
}

impl MockEc2Client {
    fn instance_to_resource(&self, instance: &aws_sdk_ec2::types::Instance) -> Option<Resource> {
        let instance_id = instance.instance_id.as_ref()?;
        let instance_type = instance.instance_type.as_ref()?;
        let state = instance.state.as_ref()?;
        
        // Only include running instances
        if state.name() != Some(&aws_sdk_ec2::types::InstanceStateName::Running) {
            return None;
        }
        
        let mut metadata = HashMap::new();
        
        // Basic instance information
        metadata.insert("instance_type".to_string(), 
            instance_type.as_str().to_string());
        metadata.insert("state".to_string(), 
            format!("{:?}", state.name()));
        metadata.insert("region".to_string(), "us-east-1".to_string());
        metadata.insert("provider".to_string(), "aws".to_string());
        metadata.insert("service".to_string(), "ec2".to_string());
        metadata.insert("mock".to_string(), "true".to_string());
        
        // Instance details
        if let Some(ami_id) = &instance.image_id {
            metadata.insert("ami_id".to_string(), ami_id.to_string());
        }
        
        if let Some(key_name) = &instance.key_name {
            metadata.insert("key_name".to_string(), key_name.to_string());
        }
        
        if let Some(subnet_id) = &instance.subnet_id {
            metadata.insert("subnet_id".to_string(), subnet_id.to_string());
        }
        
        if let Some(vpc_id) = &instance.vpc_id {
            metadata.insert("vpc_id".to_string(), vpc_id.to_string());
        }
        
        // Security groups
        if let Some(security_groups) = &instance.security_groups {
            let sg_ids: Vec<String> = security_groups
                .iter()
                .filter_map(|sg| sg.group_id.as_ref())
                .map(|id| id.to_string())
                .collect();
            metadata.insert("security_groups".to_string(), sg_ids.join(","));
        }
        
        // Tags
        if let Some(tags) = &instance.tags {
            for tag in tags {
                if let (Some(key), Some(value)) = (tag.key.as_ref(), tag.value.as_ref()) {
                    metadata.insert(format!("tag_{}", key), value.to_string());
                }
            }
        }
        
        // Public IP
        if let Some(public_ip) = &instance.public_ip_address {
            metadata.insert("public_ip".to_string(), public_ip.to_string());
        }
        
        // Private IP
        if let Some(private_ip) = &instance.private_ip_address {
            metadata.insert("private_ip".to_string(), private_ip.to_string());
        }
        
        // Launch time
        if let Some(launch_time) = &instance.launch_time {
            metadata.insert("launch_time".to_string(), 
                format!("{}", launch_time.secs()));
        }
        
        let mut resource = Resource::new(
            "ec2-instance".to_string(),
            instance_id.to_string(),
        );
        
        for (key, value) in metadata {
            resource.metadata.insert(key, value);
        }
        
        Some(resource)
    }
}

/// Mock S3 Client for testing
pub struct MockS3Client {
    buckets: Vec<aws_sdk_s3::types::Bucket>,
    objects: HashMap<String, Vec<aws_sdk_s3::types::Object>>,
}

impl MockS3Client {
    pub fn new() -> Self {
        let mut objects = HashMap::new();
        
        // Add objects to test bucket
        objects.insert("test-bucket".to_string(), vec![
            aws_sdk_s3::types::Object::builder()
                .key("test-object.txt".to_string())
                .size(1024)
                .last_modified(aws_sdk_s3::primitives::DateTime::from_secs(1609459200i64))
                .storage_class(aws_sdk_s3::types::ObjectStorageClass::Standard)
                .e_tag("test-etag".to_string())
                .build(),
            aws_sdk_s3::types::Object::builder()
                .key("large-object.bin".to_string())
                .size(100 * 1024 * 1024) // 100MB
                .last_modified(aws_sdk_s3::primitives::DateTime::from_secs(16094592100i64))
                .storage_class(aws_sdk_s3::types::ObjectStorageClass::Glacier)
                .e_tag("large-etag".to_string())
                .build(),
        ]);
        
        Self {
            buckets: vec![
                aws_sdk_s3::types::Bucket::builder()
                    .name("test-bucket".to_string())
                    .creation_date(aws_sdk_s3::primitives::DateTime::from_secs(1609459200i64))
                    .build(),
                aws_sdk_s3::types::Bucket::builder()
                    .name("empty-bucket".to_string())
                    .creation_date(aws_sdk_s3::primitives::DateTime::from_secs(1609459100i64))
                    .build(),
            ],
            objects,
        }
    }
}

#[async_trait]
impl S3Discovery for MockS3Client {
    async fn discover_buckets(&self) -> Result<Vec<Resource>> {
        info!("Mock S3 discovery returning {} buckets", self.buckets.len());
        
        let mut resources = Vec::new();
        for bucket in &self.buckets {
            if let Some(resource) = self.bucket_to_resource(bucket) {
                resources.push(resource);
            }
        }
        
        Ok(resources)
    }
}

impl MockS3Client {
    async fn discover_bucket_objects(&self, bucket_name: &str) -> Result<Vec<Resource>> {
        info!("Mock S3 object discovery for bucket: {}", bucket_name);
        
        let mut resources = Vec::new();
        if let Some(objects) = self.objects.get(bucket_name) {
            for object in objects {
                if let Some(resource) = self.object_to_resource(bucket_name, object) {
                    resources.push(resource);
                }
            }
        }
        
        Ok(resources)
    }
    
    fn bucket_to_resource(&self, bucket: &aws_sdk_s3::types::Bucket) -> Option<Resource> {
        let bucket_name = bucket.name.as_ref()?;
        
        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), "aws".to_string());
        metadata.insert("service".to_string(), "s3".to_string());
        metadata.insert("resource_type".to_string(), "bucket".to_string());
        metadata.insert("region".to_string(), "us-east-1".to_string());
        metadata.insert("mock".to_string(), "true".to_string());
        
        // Bucket creation date
        if let Some(creation_date) = &bucket.creation_date {
            metadata.insert("creation_date".to_string(), 
                format!("{}", creation_date.secs()));
        }
        
        let mut resource = Resource::new(
            "s3-bucket".to_string(),
            bucket_name.to_string(),
        );
        
        for (key, value) in metadata {
            resource.metadata.insert(key, value);
        }
        
        Some(resource)
    }
    
    fn object_to_resource(&self, bucket_name: &str, object: &aws_sdk_s3::types::Object) -> Option<Resource> {
        let key = object.key.as_ref()?;
        let size = object.size;
        
        let mut metadata = HashMap::new();
        metadata.insert("provider".to_string(), "aws".to_string());
        metadata.insert("service".to_string(), "s3".to_string());
        metadata.insert("region".to_string(), "us-east-1".to_string());
        metadata.insert("resource_type".to_string(), "object".to_string());
        metadata.insert("bucket_name".to_string(), bucket_name.to_string());
        metadata.insert("size".to_string(), size.to_string());
        metadata.insert("mock".to_string(), "true".to_string());
        
        // Last modified date
        if let Some(last_modified) = &object.last_modified {
            metadata.insert("last_modified".to_string(), 
                format!("{}", last_modified.secs()));
        }
        
        // Storage class
        if let Some(storage_class) = &object.storage_class {
            metadata.insert("storage_class".to_string(), 
                format!("{:?}", storage_class));
        }
        
        // ETag
        if let Some(etag) = &object.e_tag {
            metadata.insert("etag".to_string(), etag.to_string());
        }
        
        // Object key (path)
        metadata.insert("object_key".to_string(), key.to_string());
        
        // Calculate if object is likely large (> 100MB)
        let is_large = size > 100 * 1024 * 1024;
        metadata.insert("is_large".to_string(), is_large.to_string());
        
        let mut resource = Resource::new(
            "s3-object".to_string(),
            format!("{}:{}", bucket_name, key),
        );
        
        for (key, value) in metadata {
            resource.metadata.insert(key, value);
        }
        
        Some(resource)
    }
}

/// Mock AWS client factory
pub struct MockAwsClientFactory;

impl MockAwsClientFactory {
    pub fn create_ec2_client(&self, _config: &AwsConfig) -> Arc<dyn Ec2Discovery> {
        Arc::new(MockEc2Client::new())
    }
    
    pub fn create_s3_client(&self, _config: &AwsConfig) -> Arc<dyn S3Discovery> {
        Arc::new(MockS3Client::new())
    }
}

/// Replace the real AWS client factory with mock for testing
pub struct MockClientFactory;

impl MockClientFactory {
    pub fn create_ec2_client(&self, _config: &AwsConfig) -> Arc<dyn Ec2Discovery> {
        Arc::new(MockEc2Client::new())
    }
    
    pub fn create_s3_client(&self, _config: &AwsConfig) -> Arc<dyn S3Discovery> {
        Arc::new(MockS3Client::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_ec2_client_creation() {
        let client = MockEc2Client::new();
        assert_eq!(client.instances.len(), 2);
        assert_eq!(client.instances[0].instance_id.as_ref().unwrap(), "i-1234567890abcdef0");
        assert_eq!(client.instances[1].instance_id.as_ref().unwrap(), "i-0987654321fedcbae");
    }
    
    #[test]
    fn test_mock_s3_client_creation() {
        let client = MockS3Client::new();
        assert_eq!(client.buckets.len(), 2);
        assert_eq!(client.buckets[0].name.as_ref().unwrap(), "test-bucket");
        assert_eq!(client.buckets[1].name.as_ref().unwrap(), "empty-bucket");
        assert!(client.objects.get("test-bucket").unwrap().len(), 2);
    }
    
    #[test]
    fn test_mock_factory() {
        let factory = MockClientFactory;
        let config = super::super::config::AwsConfig::default();
        
        let ec2_client = factory.create_ec2_client(&config);
        let s3_client = factory.create_s3_client(&config);
        
        // Should create mock clients
        assert_eq!(ec2_client.discover_instances().await.unwrap().len(), 2);
        assert_eq!(s3_client.discover_buckets().await.unwrap().len(), 2);
    }
}