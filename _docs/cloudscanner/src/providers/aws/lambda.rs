use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, error};

use crate::models::provider::Resource;
use super::AwsProviderConfig as AwsConfig;

/// Lambda function discovery trait
#[async_trait]
pub trait LambdaDiscovery: Send + Sync {
    async fn discover_functions(&self) -> Result<Vec<Resource>>;
}

/// Mock Lambda discovery for testing
pub struct MockLambdaDiscovery;

impl Default for MockLambdaDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl LambdaDiscovery for MockLambdaDiscovery {
    async fn discover_functions(&self) -> Result<Vec<Resource>> {
        info!("Using mock Lambda discovery");
        
        Ok(vec![
            Resource::new("lambda-function".to_string(), "test-function-1".to_string())
                .with_metadata("function_name".to_string(), "test-function-1".to_string())
                .with_metadata("runtime".to_string(), "python3.9".to_string())
                .with_metadata("handler".to_string(), "lambda_function.lambda_handler".to_string())
                .with_metadata("code_size".to_string(), "1024".to_string())
                .with_metadata("timeout".to_string(), "300".to_string())
                .with_metadata("memory_size".to_string(), "128".to_string())
                .with_metadata("last_modified".to_string(), "2023-12-01T12:00:00Z".to_string()),
                
            Resource::new("lambda-function".to_string(), "test-function-2".to_string())
                .with_metadata("function_name".to_string(), "test-function-2".to_string())
                .with_metadata("runtime".to_string(), "nodejs18.x".to_string())
                .with_metadata("handler".to_string(), "index.handler".to_string())
                .with_metadata("code_size".to_string(), "2048".to_string())
                .with_metadata("timeout".to_string(), "60".to_string())
                .with_metadata("memory_size".to_string(), "256".to_string())
                .with_metadata("last_modified".to_string(), "2023-12-02T15:30:00Z".to_string()),
                
            Resource::new("lambda-function".to_string(), "api-gateway-function".to_string())
                .with_metadata("function_name".to_string(), "api-gateway-function".to_string())
                .with_metadata("runtime".to_string(), "python3.9".to_string())
                .with_metadata("handler".to_string(), "app.lambda_handler".to_string())
                .with_metadata("code_size".to_string(), "4096".to_string())
                .with_metadata("timeout".to_string(), "30".to_string())
                .with_metadata("memory_size".to_string(), "512".to_string())
                .with_metadata("last_modified".to_string(), "2023-12-03T09:15:00Z".to_string())
                .with_metadata("trigger".to_string(), "api-gateway".to_string()),
        ])
    }
}

/// Real AWS Lambda discovery implementation
pub struct LambdaDiscoveryImpl {
    client: aws_sdk_lambda::Client,
    config: AwsConfig,
}

impl LambdaDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS Lambda client for region: {}", config.region);
        
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_sdk_lambda::config::Region::new(config.region.clone()))
            .load()
            .await;
            
        let client = aws_sdk_lambda::Client::new(&aws_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl LambdaDiscovery for LambdaDiscoveryImpl {
    async fn discover_functions(&self) -> Result<Vec<Resource>> {
        info!("Discovering Lambda functions in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.list_functions();
            
            if let Some(token) = &next_token {
                request = request.marker(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    let functions = response.functions();
                    for function in functions {
                            let resource = Resource::new("lambda-function".to_string(), 
                                                        function.function_name().unwrap_or("unknown").to_string())
                                .with_metadata("function_name".to_string(), 
                                             function.function_name().unwrap_or("").to_string())
                                .with_metadata("runtime".to_string(), 
                                             function.runtime().map(|r| r.as_str()).unwrap_or("").to_string())
                                .with_metadata("handler".to_string(), 
                                             function.handler().unwrap_or("").to_string())
                                .with_metadata("code_size".to_string(), 
                                             function.code_size().to_string())
                                .with_metadata("timeout".to_string(), 
                                             function.timeout().to_string())
                                .with_metadata("memory_size".to_string(), 
                                             function.memory_size().to_string())
                                .with_metadata("last_modified".to_string(), 
                                             function.last_modified().unwrap_or("").to_string())
                                .with_metadata("state".to_string(), 
                                             function.state().map(|s| s.as_str()).unwrap_or("").to_string());
                                
                            // Add VPC configuration if present
                            if let Some(vpc_config) = function.vpc_config() {
                                if vpc_config.vpc_id().is_some() {
                                    resource.metadata.insert("vpc_id".to_string(), 
                                                          vpc_config.vpc_id().unwrap_or("").to_string());
                                }
                                let subnet_ids = vpc_config.subnet_ids(); if !subnet_ids.is_empty() {
                                    resource.metadata.insert("subnet_ids".to_string(), 
                                                          format!("{:?}", subnet_ids));
                                }
                                let security_group_ids = vpc_config.security_group_ids(); if !security_group_ids.is_empty() {
                                    resource.metadata.insert("security_group_ids".to_string(), 
                                                          format!("{:?}", security_group_ids));
                                }
                            }
                            
                            // Add environment variables count
                            if let Some(env) = function.environment() {
                                if let Some(variables) = env.variables() {
                                    resource.metadata.insert("environment_variables_count".to_string(), 
                                                          variables.len().to_string());
                                }
                            }
                            
                            resources.push(resource);
                    }
                    
                    next_token = response.next_marker().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list Lambda functions: {}", e);
                    return Err(anyhow::anyhow!("Lambda discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} Lambda functions", resources.len());
        Ok(resources)
    }
}