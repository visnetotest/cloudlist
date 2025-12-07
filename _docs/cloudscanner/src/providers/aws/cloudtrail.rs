use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use super::config::AwsConfig;

/// CloudTrail and CloudWatch discovery trait
#[async_trait]
pub trait CloudTrailDiscovery: Send + Sync {
    async fn discover_trails(&self) -> Result<Vec<Resource>>;
    async fn discover_events(&self, trail_arn: &str) -> Result<Vec<Resource>>;
}

/// CloudWatch discovery trait
#[async_trait]
pub trait CloudWatchDiscovery: Send + Sync {
    async fn discover_log_groups(&self) -> Result<Vec<Resource>>;
    async fn discover_metrics(&self) -> Result<Vec<Resource>>;
    async fn discover_alarms(&self) -> Result<Vec<Resource>>;
}

/// Mock CloudTrail discovery for testing
pub struct MockCloudTrailDiscovery;

impl Default for MockCloudTrailDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl CloudTrailDiscovery for MockCloudTrailDiscovery {
    async fn discover_trails(&self) -> Result<Vec<Resource>> {
        info!("Using mock CloudTrail discovery");
        
        Ok(vec![
            Resource::new("cloudtrail".to_string(), "arn:aws:cloudtrail:us-east-1:123456789012:trail/DefaultTrail".to_string())
                .with_metadata("trail_name".to_string(), "DefaultTrail".to_string())
                .with_metadata("trail_arn".to_string(), "arn:aws:cloudtrail:us-east-1:123456789012:trail/DefaultTrail".to_string())
                .with_metadata("s3_bucket_name".to_string(), "my-cloudtrail-bucket".to_string())
                .with_metadata("s3_key_prefix".to_string(), "AWSLogs/".to_string())
                .with_metadata("is_multi_region".to_string(), "false".to_string())
                .with_metadata("home_region".to_string(), "us-east-1".to_string())
                .with_metadata("include_global_services".to_string(), "true".to_string())
                .with_metadata("is_logging".to_string(), "true".to_string())
                .with_metadata("status".to_string(), "IsLogging".to_string())
                .with_metadata("kms_key_id".to_string(), "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012".to_string()),
                
            Resource::new("cloudtrail".to_string(), "arn:aws:cloudtrail:us-east-1:123456789012:trail/OrganizationTrail".to_string())
                .with_metadata("trail_name".to_string(), "OrganizationTrail".to_string())
                .with_metadata("trail_arn".to_string(), "arn:aws:cloudtrail:us-east-1:123456789012:trail/OrganizationTrail".to_string())
                .with_metadata("s3_bucket_name".to_string(), "org-cloudtrail-bucket".to_string())
                .with_metadata("s3_key_prefix".to_string(), "organization/".to_string())
                .with_metadata("is_multi_region".to_string(), "true".to_string())
                .with_metadata("home_region".to_string(), "us-east-1".to_string())
                .with_metadata("include_global_services".to_string(), "true".to_string())
                .with_metadata("is_logging".to_string(), "true".to_string())
                .with_metadata("status".to_string(), "IsLogging".to_string())
                .with_metadata("organization_id".to_string(), "o-123456789012".to_string()),
        ])
    }
    
    async fn discover_events(&self, _trail_arn: &str) -> Result<Vec<Resource>> {
        info!("Using mock CloudTrail events discovery");
        
        Ok(vec![
            Resource::new("cloudtrail-event".to_string(), "event-12345678".to_string())
                .with_metadata("event_id".to_string(), "event-12345678".to_string())
                .with_metadata("event_name".to_string(), "RunInstances".to_string())
                .with_metadata("event_source".to_string(), "ec2.amazonaws.com".to_string())
                .with_metadata("event_time".to_string(), "2023-12-05T10:30:00Z".to_string())
                .with_metadata("user_name".to_string(), "admin-user".to_string())
                .with_metadata("aws_region".to_string(), "us-east-1".to_string())
                .with_metadata("source_ip_address".to_string(), "203.0.113.1".to_string())
                .with_metadata("user_agent".to_string(), "aws-cli/2.0.55".to_string())
                .with_metadata("error_code".to_string(), "".to_string())
                .with_metadata("read_only".to_string(), "false".to_string()),
                
            Resource::new("cloudtrail-event".to_string(), "event-87654321".to_string())
                .with_metadata("event_id".to_string(), "event-87654321".to_string())
                .with_metadata("event_name".to_string(), "CreateBucket".to_string())
                .with_metadata("event_source".to_string(), "s3.amazonaws.com".to_string())
                .with_metadata("event_time".to_string(), "2023-12-05T11:45:00Z".to_string())
                .with_metadata("user_name".to_string(), "service-account".to_string())
                .with_metadata("aws_region".to_string(), "us-east-1".to_string())
                .with_metadata("source_ip_address".to_string(), "203.0.113.2".to_string())
                .with_metadata("user_agent".to_string(), "console.aws.amazon.com".to_string())
                .with_metadata("error_code".to_string(), "".to_string())
                .with_metadata("read_only".to_string(), "false".to_string()),
        ])
    }
}

/// Mock CloudWatch discovery for testing
pub struct MockCloudWatchDiscovery;

impl Default for MockCloudWatchDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl CloudWatchDiscovery for MockCloudWatchDiscovery {
    async fn discover_log_groups(&self) -> Result<Vec<Resource>> {
        info!("Using mock CloudWatch log groups discovery");
        
        Ok(vec![
            Resource::new("cloudwatch-log-group".to_string(), "arn:aws:logs:us-east-1:123456789012:log-group:/aws/lambda/us-east-1-123456789012".to_string())
                .with_metadata("log_group_name".to_string(), "/aws/lambda/us-east-1-123456789012".to_string())
                .with_metadata("log_group_arn".to_string(), "arn:aws:logs:us-east-1:123456789012:log-group:/aws/lambda/us-east-1-123456789012".to_string())
                .with_metadata("retention_in_days".to_string(), "30".to_string())
                .with_metadata("metric_filter_count".to_string(), "2".to_string())
                .with_metadata("stored_bytes".to_string(), "1048576".to_string())
                .with_metadata("creation_time".to_string(), "2023-11-01T00:00:00Z".to_string()),
                
            Resource::new("cloudwatch-log-group".to_string(), "arn:aws:logs:us-east-1:123456789012:log-group:/aws/rds/instance/mysql-db".to_string())
                .with_metadata("log_group_name".to_string(), "/aws/rds/instance/mysql-db".to_string())
                .with_metadata("log_group_arn".to_string(), "arn:aws:logs:us-east-1:123456789012:log-group:/aws/rds/instance/mysql-db".to_string())
                .with_metadata("retention_in_days".to_string(), "7".to_string())
                .with_metadata("metric_filter_count".to_string(), "1".to_string())
                .with_metadata("stored_bytes".to_string(), "5242880".to_string())
                .with_metadata("creation_time".to_string(), "2023-10-15T00:00:00Z".to_string()),
        ])
    }
    
    async fn discover_metrics(&self) -> Result<Vec<Resource>> {
        info!("Using mock CloudWatch metrics discovery");
        
        Ok(vec![
            Resource::new("cloudwatch-metric".to_string(), "CPUUtilization-mysql-db".to_string())
                .with_metadata("metric_name".to_string(), "CPUUtilization".to_string())
                .with_metadata("namespace".to_string(), "AWS/RDS".to_string())
                .with_metadata("dimensions".to_string(), "DBInstanceIdentifier=mysql-db".to_string())
                .with_metadata("unit".to_string(), "Percent".to_string())
                .with_metadata("statistic".to_string(), "Average".to_string())
                .with_metadata("period".to_string(), "300".to_string())
                .with_metadata("region".to_string(), "us-east-1".to_string()),
                
            Resource::new("cloudwatch-metric".to_string(), "NetworkOut-web-app".to_string())
                .with_metadata("metric_name".to_string(), "NetworkOut".to_string())
                .with_metadata("namespace".to_string(), "AWS/EC2".to_string())
                .with_metadata("dimensions".to_string(), "InstanceId=i-1234567890abcdef0".to_string())
                .with_metadata("unit".to_string(), "Bytes".to_string())
                .with_metadata("statistic".to_string(), "Sum".to_string())
                .with_metadata("period".to_string(), "60".to_string())
                .with_metadata("region".to_string(), "us-east-1".to_string()),
                
            Resource::new("cloudwatch-metric".to_string(), "Invocations-api-function".to_string())
                .with_metadata("metric_name".to_string(), "Invocations".to_string())
                .with_metadata("namespace".to_string(), "AWS/Lambda".to_string())
                .with_metadata("dimensions".to_string(), "FunctionName=api-function".to_string())
                .with_metadata("unit".to_string(), "Count".to_string())
                .with_metadata("statistic".to_string(), "Sum".to_string())
                .with_metadata("period".to_string(), "60".to_string())
                .with_metadata("region".to_string(), "us-east-1".to_string()),
        ])
    }
    
    async fn discover_alarms(&self) -> Result<Vec<Resource>> {
        info!("Using mock CloudWatch alarms discovery");
        
        Ok(vec![
            Resource::new("cloudwatch-alarm".to_string(), "HighCPUUtilization-mysql-db".to_string())
                .with_metadata("alarm_name".to_string(), "HighCPUUtilization-mysql-db".to_string())
                .with_metadata("alarm_arn".to_string(), "arn:aws:cloudwatch:us-east-1:123456789012:alarm:HighCPUUtilization-mysql-db".to_string())
                .with_metadata("metric_name".to_string(), "CPUUtilization".to_string())
                .with_metadata("namespace".to_string(), "AWS/RDS".to_string())
                .with_metadata("statistic".to_string(), "Average".to_string())
                .with_metadata("period".to_string(), "300".to_string())
                .with_metadata("evaluation_periods".to_string(), "2".to_string())
                .with_metadata("threshold".to_string(), "80.0".to_string())
                .with_metadata("comparison_operator".to_string(), "GreaterThanThreshold".to_string())
                .with_metadata("state_value".to_string(), "ALARM".to_string())
                .with_metadata("state_reason".to_string(), "Threshold Crossed: 1 datapoint (85.0) was greater than or equal to the threshold (80.0).".to_string())
                .with_metadata("actions_enabled".to_string(), "true".to_string())
                .with_metadata("alarm_actions_count".to_string(), "1".to_string())
                .with_metadata("ok_actions_count".to_string(), "1".to_string())
                .with_metadata("insufficient_data_actions_count".to_string(), "0".to_string()),
                
            Resource::new("cloudwatch-alarm".to_string(), "LambdaErrors-api-function".to_string())
                .with_metadata("alarm_name".to_string(), "LambdaErrors-api-function".to_string())
                .with_metadata("alarm_arn".to_string(), "arn:aws:cloudwatch:us-east-1:123456789012:alarm:LambdaErrors-api-function".to_string())
                .with_metadata("metric_name".to_string(), "Errors".to_string())
                .with_metadata("namespace".to_string(), "AWS/Lambda".to_string())
                .with_metadata("statistic".to_string(), "Sum".to_string())
                .with_metadata("period".to_string(), "300".to_string())
                .with_metadata("evaluation_periods".to_string(), "1".to_string())
                .with_metadata("threshold".to_string(), "5.0".to_string())
                .with_metadata("comparison_operator".to_string(), "GreaterThanThreshold".to_string())
                .with_metadata("state_value".to_string(), "OK".to_string())
                .with_metadata("state_reason".to_string(), "Threshold Crossed: 1 datapoint (2.0) was greater than or equal to the threshold (5.0).".to_string())
                .with_metadata("actions_enabled".to_string(), "true".to_string())
                .with_metadata("alarm_actions_count".to_string(), "2".to_string())
                .with_metadata("ok_actions_count".to_string(), "0".to_string())
                .with_metadata("insufficient_data_actions_count".to_string(), "0".to_string()),
        ])
    }
}

/// Real AWS CloudTrail discovery implementation
pub struct CloudTrailDiscoveryImpl {
    client: aws_sdk_cloudtrail::Client,
    config: AwsConfig,
}

impl CloudTrailDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS CloudTrail client for region: {}", config.region);
        
        let mut aws_config = aws_config::from_env()
            .region(aws_sdk_cloudtrail::config::Region::new(config.region.clone()));
            
        // Set custom endpoint if provided (for LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url);
        }
        
        let client = aws_sdk_cloudtrail::Client::new(&aws_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl CloudTrailDiscovery for CloudTrailDiscoveryImpl {
    async fn discover_trails(&self) -> Result<Vec<Resource>> {
        info!("Discovering CloudTrail trails in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        
        match self.client.describe_trails().send().await {
            Ok(response) => {
                if let Some(trails) = response.trail_list() {
                    for trail in trails {
                        let resource = Resource::new("cloudtrail".to_string(), 
                                                    trail.trail_arn().unwrap_or("unknown").to_string())
                                .with_metadata("trail_name".to_string(), 
                                             trail.name().unwrap_or("").to_string())
                                .with_metadata("trail_arn".to_string(), 
                                             trail.trail_arn().unwrap_or("").to_string())
                                .with_metadata("s3_bucket_name".to_string(), 
                                             trail.s3_bucket_name().unwrap_or("").to_string())
                                .with_metadata("s3_key_prefix".to_string(), 
                                             trail.s3_key_prefix().unwrap_or("").to_string())
                                .with_metadata("is_multi_region".to_string(), 
                                             trail.is_multi_region_trail().unwrap_or(false).to_string())
                                .with_metadata("home_region".to_string(), 
                                             trail.home_region().unwrap_or("").to_string())
                                .with_metadata("include_global_services".to_string(), 
                                             trail.include_global_service_events().unwrap_or(false).to_string())
                                .with_metadata("is_logging".to_string(), 
                                             trail.is_logging().unwrap_or(false).to_string())
                                .with_metadata("has_custom_event_selectors".to_string(), 
                                             trail.has_custom_event_selectors().unwrap_or(false).to_string())
                                .with_metadata("has_insight_selectors".to_string(), 
                                             trail.has_insight_selectors().unwrap_or(false).to_string());
                        
                        // Add S3 details
                        if let Some(s3) = trail.s3_bucket_name() {
                            resource.metadata.insert("s3_bucket_arn".to_string(), 
                                                  format!("arn:aws:s3:::{}", s3));
                        }
                        
                        // Add KMS details
                        if let Some(kms) = trail.kms_key_id() {
                            resource.metadata.insert("kms_key_id".to_string(), 
                                                  kms.to_string());
                        }
                        
                        // Add CloudWatch log group
                        if let Some(log_group) = trail.cloud_watch_logs_log_group_arn() {
                            resource.metadata.insert("cloudwatch_log_group_arn".to_string(), 
                                                  log_group.to_string());
                        }
                        
                        // Add tags
                        if let Some(tags_list) = trail.tags_list() {
                            let tag_map: std::collections::HashMap<String, String> = tags_list
                                .iter()
                                .filter_map(|tag| {
                                    tag.key().map(|key| {
                                        let value = tag.value().unwrap_or("").to_string();
                                        (key.to_string(), value)
                                    })
                                })
                                .collect();
                            if !tag_map.is_empty() {
                                resource.metadata.insert("tags".to_string(), 
                                                      format!("{:?}", tag_map));
                            }
                        }
                        
                        resources.push(resource);
                    }
                }
            }
            Err(e) => {
                error!("Failed to describe CloudTrail trails: {}", e);
                return Err(anyhow::anyhow!("CloudTrail discovery failed: {}", e));
            }
        }
        
        info!("Discovered {} CloudTrail trails", resources.len());
        Ok(resources)
    }
    
    async fn discover_events(&self, trail_arn: &str) -> Result<Vec<Resource>> {
        info!("Discovering CloudTrail events for trail: {}", trail_arn);
        
        let mut resources = Vec::new();
        let end_time = chrono::Utc::now();
        let start_time = end_time - chrono::Duration::hours(24); // Last 24 hours
        
        let mut request = self.client.lookup_events()
            .set_lookup_attributes(aws_sdk_cloudtrail::types::LookupAttributes::All)
            .set_start_time(aws_smithy_types::DateTime::from(start_time))
            .set_end_time(aws_smithy_types::DateTime::from(end_time));
        
        match request.send().await {
            Ok(response) => {
                if let Some(events) = response.events() {
                    for event in events {
                        let resource = Resource::new("cloudtrail-event".to_string(), 
                                                            event.event_id().unwrap_or("unknown").to_string())
                                .with_metadata("event_id".to_string(), 
                                             event.event_id().unwrap_or("").to_string())
                                .with_metadata("event_name".to_string(), 
                                             event.event_name().unwrap_or("").to_string())
                                .with_metadata("event_source".to_string(), 
                                             event.event_source().unwrap_or("").to_string())
                                .with_metadata("event_time".to_string(), 
                                             event.event_time().unwrap_or("").to_string())
                                .with_metadata("user_name".to_string(), 
                                             event.username().unwrap_or("").to_string())
                                .with_metadata("aws_region".to_string(), 
                                             event.aws_region().unwrap_or("").to_string())
                                .with_metadata("source_ip_address".to_string(), 
                                             event.source_ip_address().unwrap_or("").to_string())
                                .with_metadata("user_agent".to_string(), 
                                             event.user_agent().unwrap_or("").to_string())
                                .with_metadata("error_code".to_string(), 
                                             event.error_code().unwrap_or("").to_string())
                                .with_metadata("read_only".to_string(), 
                                             event.read_only().unwrap_or(false).to_string());
                        
                        // Add request parameters
                        if let Some(params) = event.request_parameters() {
                            resource.metadata.insert("request_parameters".to_string(), 
                                                  format!("{:?}", params));
                        }
                        
                        // Add response elements
                        if let Some(response) = event.response_elements() {
                            resource.metadata.insert("response_elements".to_string(), 
                                                  format!("{:?}", response));
                        }
                        
                        // Add API call details
                        if let Some(resources) = event.resources() {
                            let resource_names: Vec<String> = resources
                                .iter()
                                .filter_map(|r| r.resource_name())
                                .map(|name| name.to_string())
                                .collect();
                            if !resource_names.is_empty() {
                                resource.metadata.insert("affected_resources".to_string(), 
                                                      format!("{:?}", resource_names));
                            }
                        }
                        
                        resources.push(resource);
                    }
                }
            }
            Err(e) => {
                error!("Failed to lookup CloudTrail events: {}", e);
                return Err(anyhow::anyhow!("CloudTrail events discovery failed: {}", e));
            }
        }
        
        info!("Discovered {} CloudTrail events", resources.len());
        Ok(resources)
    }
}

/// Real AWS CloudWatch discovery implementation
pub struct CloudWatchDiscoveryImpl {
    client: aws_sdk_cloudwatchlogs::Client,
    metrics_client: aws_sdk_cloudwatch::Client,
    config: AwsConfig,
}

impl CloudWatchDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS CloudWatch clients for region: {}", config.region);
        
        let mut aws_config = aws_config::from_env()
            .region(aws_sdk_cloudwatchlogs::config::Region::new(config.region.clone()));
            
        // Set custom endpoint if provided (for LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url);
        }
        
        let logs_client = aws_sdk_cloudwatchlogs::Client::new(&aws_config);
        let metrics_client = aws_sdk_cloudwatch::Client::new(&aws_config);
        
        Ok(Self { 
            client: logs_client, 
            metrics_client, 
            config 
        })
    }
}

#[async_trait]
impl CloudWatchDiscovery for CloudWatchDiscoveryImpl {
    async fn discover_log_groups(&self) -> Result<Vec<Resource>> {
        info!("Discovering CloudWatch log groups in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_log_groups();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(log_groups) = response.log_groups() {
                        for log_group in log_groups {
                            let resource = Resource::new("cloudwatch-log-group".to_string(), 
                                                                log_group.arn().unwrap_or("unknown").to_string())
                                    .with_metadata("log_group_name".to_string(), 
                                                 log_group.log_group_name().unwrap_or("").to_string())
                                    .with_metadata("log_group_arn".to_string(), 
                                                 log_group.arn().unwrap_or("").to_string())
                                    .with_metadata("retention_in_days".to_string(), 
                                                 log_group.retention_in_days().map(|d| d.to_string()).unwrap_or_else(|| "Never".to_string()))
                                    .with_metadata("stored_bytes".to_string(), 
                                                 log_group.stored_bytes().map(|b| b.to_string()).unwrap_or_else(|| "0".to_string()));
                            
                            // Get metric filters
                            match self.client.describe_metric_filters()
                                .set_log_group_name(log_group.log_group_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(filter_response) => {
                                    if let Some(filters) = filter_response.metric_filters() {
                                        resource.metadata.insert("metric_filter_count".to_string(), 
                                                              filters.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to describe metric filters: {}", e);
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to describe CloudWatch log groups: {}", e);
                    return Err(anyhow::anyhow!("CloudWatch log groups discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} CloudWatch log groups", resources.len());
        Ok(resources)
    }
    
    async fn discover_metrics(&self) -> Result<Vec<Resource>> {
        info!("Discovering CloudWatch metrics in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.metrics_client.list_metrics();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(metrics) = response.metrics() {
                        for metric in metrics {
                            let resource = Resource::new("cloudwatch-metric".to_string(), 
                                                                format!("{}-{}", 
                                                                        metric.namespace().unwrap_or("unknown"), 
                                                                        metric.metric_name().unwrap_or("unknown")).to_string())
                                    .with_metadata("metric_name".to_string(), 
                                                 metric.metric_name().unwrap_or("").to_string())
                                    .with_metadata("namespace".to_string(), 
                                                 metric.namespace().unwrap_or("").to_string())
                                    .with_metadata("unit".to_string(), 
                                                 metric.unit().unwrap_or("").to_string());
                            
                            // Format dimensions
                            if let Some(dimensions) = metric.dimensions() {
                                let dim_strs: Vec<String> = dimensions
                                    .iter()
                                    .filter_map(|d| {
                                        d.name().map(|name| {
                                            let value = d.value().unwrap_or("").to_string();
                                            format!("{}={}", name, value)
                                        })
                                    })
                                    .collect();
                                if !dim_strs.is_empty() {
                                    resource.metadata.insert("dimensions".to_string(), 
                                                          dim_strs.join(","));
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list CloudWatch metrics: {}", e);
                    return Err(anyhow::anyhow!("CloudWatch metrics discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} CloudWatch metrics", resources.len());
        Ok(resources)
    }
    
    async fn discover_alarms(&self) -> Result<Vec<Resource>> {
        info!("Discovering CloudWatch alarms in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.metrics_client.describe_alarms();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(alarms) = response.metric_alarms() {
                        for alarm in alarms {
                            let resource = Resource::new("cloudwatch-alarm".to_string(), 
                                                                alarm.alarm_name().unwrap_or("unknown").to_string())
                                    .with_metadata("alarm_name".to_string(), 
                                                 alarm.alarm_name().unwrap_or("").to_string())
                                    .with_metadata("alarm_arn".to_string(), 
                                                 alarm.alarm_arn().unwrap_or("").to_string())
                                    .with_metadata("metric_name".to_string(), 
                                                 alarm.metric_name().unwrap_or("").to_string())
                                    .with_metadata("namespace".to_string(), 
                                                 alarm.namespace().unwrap_or("").to_string())
                                    .with_metadata("statistic".to_string(), 
                                                 alarm.statistic().unwrap_or("").to_string())
                                    .with_metadata("period".to_string(), 
                                                 alarm.period().map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()))
                                    .with_metadata("evaluation_periods".to_string(), 
                                                 alarm.evaluation_periods().map(|e| e.to_string()).unwrap_or_else(|| "0".to_string()))
                                    .with_metadata("threshold".to_string(), 
                                                 alarm.threshold().map(|t| t.to_string()).unwrap_or_else(|| "0".to_string()))
                                    .with_metadata("comparison_operator".to_string(), 
                                                 alarm.comparison_operator().unwrap_or("").to_string())
                                    .with_metadata("state_value".to_string(), 
                                                 alarm.state_value().unwrap_or("").to_string())
                                    .with_metadata("state_reason".to_string(), 
                                                 alarm.state_reason().unwrap_or("").to_string())
                                    .with_metadata("actions_enabled".to_string(), 
                                                 alarm.actions_enabled().unwrap_or(false).to_string())
                                    .with_metadata("ok_actions_count".to_string(), 
                                                 alarm.ok_actions().map(|a| a.len()).map(|l| l.to_string()).unwrap_or_else(|| "0".to_string()))
                                    .with_metadata("alarm_actions_count".to_string(), 
                                                 alarm.alarm_actions().map(|a| a.len()).map(|l| l.to_string()).unwrap_or_else(|| "0".to_string()))
                                    .with_metadata("insufficient_data_actions_count".to_string(), 
                                                 alarm.insufficient_data_actions().map(|a| a.len()).map(|l| l.to_string()).unwrap_or_else(|| "0".to_string()));
                            
                            resources.push(resource);
                        }
                    }
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to describe CloudWatch alarms: {}", e);
                    return Err(anyhow::anyhow!("CloudWatch alarms discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} CloudWatch alarms", resources.len());
        Ok(resources)
    }
}