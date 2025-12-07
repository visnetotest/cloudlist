use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use super::AwsProviderConfig as AwsConfig;

/// ECS cluster and service discovery trait
#[async_trait]
pub trait EcsDiscovery: Send + Sync {
    async fn discover_clusters(&self) -> Result<Vec<Resource>>;
    async fn discover_services(&self) -> Result<Vec<Resource>>;
    async fn discover_tasks(&self) -> Result<Vec<Resource>>;
}

/// Mock ECS discovery for testing
pub struct MockEcsDiscovery;

impl Default for MockEcsDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl EcsDiscovery for MockEcsDiscovery {
    async fn discover_clusters(&self) -> Result<Vec<Resource>> {
        info!("Using mock ECS cluster discovery");
        
        Ok(vec![
            Resource::new("ecs-cluster".to_string(), "test-cluster-1".to_string())
                .with_metadata("cluster_name".to_string(), "test-cluster-1".to_string())
                .with_metadata("status".to_string(), "ACTIVE".to_string())
                .with_metadata("running_tasks_count".to_string(), "5".to_string())
                .with_metadata("pending_tasks_count".to_string(), "0".to_string())
                .with_metadata("active_services_count".to_string(), "3".to_string())
                .with_metadata("registered_container_instances_count".to_string(), "2".to_string()),
                
            Resource::new("ecs-cluster".to_string(), "prod-cluster-main".to_string())
                .with_metadata("cluster_name".to_string(), "prod-cluster-main".to_string())
                .with_metadata("status".to_string(), "ACTIVE".to_string())
                .with_metadata("running_tasks_count".to_string(), "25".to_string())
                .with_metadata("pending_tasks_count".to_string(), "2".to_string())
                .with_metadata("active_services_count".to_string(), "8".to_string())
                .with_metadata("registered_container_instances_count".to_string(), "6".to_string()),
        ])
    }
    
    async fn discover_services(&self) -> Result<Vec<Resource>> {
        info!("Using mock ECS service discovery");
        
        Ok(vec![
            Resource::new("ecs-service".to_string(), "web-app-service".to_string())
                .with_metadata("service_name".to_string(), "web-app-service".to_string())
                .with_metadata("cluster".to_string(), "test-cluster-1".to_string())
                .with_metadata("task_definition".to_string(), "web-app-task:1".to_string())
                .with_metadata("desired_count".to_string(), "2".to_string())
                .with_metadata("running_count".to_string(), "2".to_string())
                .with_metadata("pending_count".to_string(), "0".to_string())
                .with_metadata("status".to_string(), "ACTIVE".to_string())
                .with_metadata("launch_type".to_string(), "FARGATE".to_string())
                .with_metadata("network_mode".to_string(), "awsvpc".to_string()),
                
            Resource::new("ecs-service".to_string(), "api-service".to_string())
                .with_metadata("service_name".to_string(), "api-service".to_string())
                .with_metadata("cluster".to_string(), "test-cluster-1".to_string())
                .with_metadata("task_definition".to_string(), "api-task:3".to_string())
                .with_metadata("desired_count".to_string(), "3".to_string())
                .with_metadata("running_count".to_string(), "3".to_string())
                .with_metadata("pending_count".to_string(), "0".to_string())
                .with_metadata("status".to_string(), "ACTIVE".to_string())
                .with_metadata("launch_type".to_string(), "EC2".to_string())
                .with_metadata("network_mode".to_string(), "bridge".to_string()),
        ])
    }
    
    async fn discover_tasks(&self) -> Result<Vec<Resource>> {
        info!("Using mock ECS task discovery");
        
        Ok(vec![
            Resource::new("ecs-task".to_string(), "web-app-task-abc123".to_string())
                .with_metadata("task_id".to_string(), "web-app-task-abc123".to_string())
                .with_metadata("cluster".to_string(), "test-cluster-1".to_string())
                .with_metadata("task_definition_arn".to_string(), "arn:aws:ecs:us-east-1:123456789012:task-definition/web-app-task:1".to_string())
                .with_metadata("container_instance_arn".to_string(), "arn:aws:ecs:us-east-1:123456789012:container-instance/i-1234567890abcdef0".to_string())
                .with_metadata("last_status".to_string(), "RUNNING".to_string())
                .with_metadata("desired_status".to_string(), "RUNNING".to_string())
                .with_metadata("launch_type".to_string(), "FARGATE".to_string()),
        ])
    }
}

/// Real AWS ECS discovery implementation
pub struct EcsDiscoveryImpl {
    client: aws_sdk_ecs::Client,
    config: AwsConfig,
}

impl EcsDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS ECS client for region: {}", config.region);
        
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_sdk_ecs::config::Region::new(config.region.clone()))
            .load()
            .await;
            
        // Set custom endpoint if provided (for LocalStack) - note: this would need custom config builder
        // For now, we'll use the basic config
        
        let client = aws_sdk_ecs::Client::new(&aws_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl EcsDiscovery for EcsDiscoveryImpl {
    async fn discover_clusters(&self) -> Result<Vec<Resource>> {
        info!("Discovering ECS clusters in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_clusters();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(clusters) = response.clusters() {
                        for cluster in clusters {
                            let resource = Resource::new("ecs-cluster".to_string(), 
                                                        cluster.cluster_name().unwrap_or("unknown").to_string())
                                .with_metadata("cluster_name".to_string(), 
                                             cluster.cluster_name().unwrap_or("").to_string())
                                .with_metadata("status".to_string(), 
                                             cluster.status().unwrap_or("").to_string())
                                .with_metadata("running_tasks_count".to_string(), 
                                             cluster.running_tasks_count().unwrap_or(0).to_string())
                                .with_metadata("pending_tasks_count".to_string(), 
                                             cluster.pending_tasks_count().unwrap_or(0).to_string())
                                .with_metadata("active_services_count".to_string(), 
                                             cluster.active_services_count().unwrap_or(0).to_string())
                                .with_metadata("registered_container_instances_count".to_string(), 
                                             cluster.registered_container_instances_count().unwrap_or(0).to_string());
                            
                            // Add cluster ARN
                            if let Some(cluster_arn) = cluster.cluster_arn() {
                                resource.metadata.insert("cluster_arn".to_string(), 
                                                      cluster_arn.to_string());
                            }
                            
                            // Add capacity providers if present
                            if let Some(capacity_providers) = cluster.capacity_providers() {
                                let provider_names: Vec<String> = capacity_providers
                                    .iter()
                                    .filter_map(|cp| cp.name())
                                    .map(|name| name.to_string())
                                    .collect();
                                if !provider_names.is_empty() {
                                    resource.metadata.insert("capacity_providers".to_string(), 
                                                          format!("{:?}", provider_names));
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
                    error!("Failed to describe ECS clusters: {}", e);
                    return Err(anyhow::anyhow!("ECS cluster discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} ECS clusters", resources.len());
        Ok(resources)
    }
    
    async fn discover_services(&self) -> Result<Vec<Resource>> {
        info!("Discovering ECS services in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.list_services();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(service_arns) = response.service_arns() {
                        // Get detailed service information
                        match self.client.describe_services()
                            .set_services(service_arns.to_vec())
                            .send()
                            .await {
                            Ok(detail_response) => {
                                if let Some(services) = detail_response.services() {
                                    for service in services {
                                        let resource = Resource::new("ecs-service".to_string(), 
                                                                    service.service_name().unwrap_or("unknown").to_string())
                                                .with_metadata("service_name".to_string(), 
                                                             service.service_name().unwrap_or("").to_string())
                                                .with_metadata("cluster_arn".to_string(), 
                                                             service.cluster_arn().unwrap_or("").to_string())
                                                .with_metadata("task_definition".to_string(), 
                                                             service.task_definition().unwrap_or("").to_string())
                                                .with_metadata("desired_count".to_string(), 
                                                             service.desired_count().unwrap_or(0).to_string())
                                                .with_metadata("running_count".to_string(), 
                                                             service.running_count().unwrap_or(0).to_string())
                                                .with_metadata("pending_count".to_string(), 
                                                             service.pending_count().unwrap_or(0).to_string())
                                                .with_metadata("status".to_string(), 
                                                             service.status().unwrap_or("").to_string())
                                                .with_metadata("launch_type".to_string(), 
                                                             service.launch_type().unwrap_or("").to_string())
                                                .with_metadata("network_mode".to_string(), 
                                                             service.network_configuration()
                                                                .and_then(|nc| nc.awsvpc_configuration())
                                                                .and_then(|awsvpc| awsvpc.subnets())
                                                                .map(|_| "awsvpc".to_string())
                                                                .unwrap_or_else(|| "bridge".to_string()));
                                        
                                        // Add service ARN
                                        if let Some(service_arn) = service.service_arn() {
                                            resource.metadata.insert("service_arn".to_string(), 
                                                                  service_arn.to_string());
                                        }
                                        
                                        // Add load balancer information if present
                                        if let Some(load_balancers) = service.load_balancers() {
                                            let lb_names: Vec<String> = load_balancers
                                                .iter()
                                                .filter_map(|lb| lb.load_balancer_name())
                                                .map(|name| name.to_string())
                                                .collect();
                                            if !lb_names.is_empty() {
                                                resource.metadata.insert("load_balancers".to_string(), 
                                                                      format!("{:?}", lb_names));
                                            }
                                        }
                                        
                                        // Add deployment information
                                        if let Some(deployments) = service.deployments() {
                                            if let Some(latest_deployment) = deployments.first() {
                                                resource.metadata.insert("task_revision".to_string(), 
                                                                      latest_deployment.task_definition().unwrap_or("").to_string());
                                                resource.metadata.insert("updated_at".to_string(), 
                                                                      latest_deployment.updated_at().unwrap_or("").to_string());
                                            }
                                        }
                                        
                                        resources.push(resource);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to describe ECS services: {}", e);
                            }
                        }
                    }
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list ECS services: {}", e);
                    return Err(anyhow::anyhow!("ECS service discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} ECS services", resources.len());
        Ok(resources)
    }
    
    async fn discover_tasks(&self) -> Result<Vec<Resource>> {
        info!("Discovering ECS tasks in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.list_tasks();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(task_arns) = response.task_arns() {
                        if !task_arns.is_empty() {
                            // Get detailed task information
                            match self.client.describe_tasks()
                                .set_tasks(task_arns.to_vec())
                                .send()
                                .await {
                                Ok(detail_response) => {
                                    if let Some(tasks) = detail_response.tasks() {
                                        for task in tasks {
                                            let resource = Resource::new("ecs-task".to_string(), 
                                                                        task.task_arn()
                                                                            .and_then(|arn| arn.split('/').last())
                                                                            .unwrap_or("unknown")
                                                                            .to_string())
                                                        .with_metadata("task_arn".to_string(), 
                                                                     task.task_arn().unwrap_or("").to_string())
                                                        .with_metadata("cluster_arn".to_string(), 
                                                                     task.cluster_arn().unwrap_or("").to_string())
                                                        .with_metadata("task_definition_arn".to_string(), 
                                                                     task.task_definition_arn().unwrap_or("").to_string())
                                                        .with_metadata("last_status".to_string(), 
                                                                     task.last_status().unwrap_or("").to_string())
                                                        .with_metadata("desired_status".to_string(), 
                                                                     task.desired_status().unwrap_or("").to_string())
                                                        .with_metadata("launch_type".to_string(), 
                                                                     task.launch_type().unwrap_or("").to_string());
                                            
                                            // Add container instance ARN if present
                                            if let Some(container_instance_arn) = task.container_instance_arn() {
                                                resource.metadata.insert("container_instance_arn".to_string(), 
                                                                      container_instance_arn.to_string());
                                            }
                                            
                                            // Add started by information
                                            if let Some(started_by) = task.started_by() {
                                                resource.metadata.insert("started_by".to_string(), 
                                                                      started_by.to_string());
                                            }
                                            
                                            // Add stop code if present
                                            if let Some(stop_code) = task.stop_code() {
                                                resource.metadata.insert("stop_code".to_string(), 
                                                                      stop_code.to_string());
                                            }
                                            
                                            resources.push(resource);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to describe ECS tasks: {}", e);
                                }
                            }
                        }
                    }
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list ECS tasks: {}", e);
                    return Err(anyhow::anyhow!("ECS task discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} ECS tasks", resources.len());
        Ok(resources)
    }
}