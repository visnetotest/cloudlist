use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use super::AwsProviderConfig;

/// RDS instance discovery trait
#[async_trait]
pub trait RdsDiscovery: Send + Sync {
    async fn discover_instances(&self) -> Result<Vec<Resource>>;
}

/// Mock RDS discovery for testing
pub struct MockRdsDiscovery;

impl Default for MockRdsDiscovery {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl RdsDiscovery for MockRdsDiscovery {
    async fn discover_instances(&self) -> Result<Vec<Resource>> {
        info!("Using mock RDS discovery");
        
        Ok(vec![
            Resource::new("rds-instance".to_string(), "test-mysql-db-1".to_string())
                .with_metadata("db_name".to_string(), "test-mysql-db-1".to_string())
                .with_metadata("engine".to_string(), "mysql".to_string())
                .with_metadata("engine_version".to_string(), "8.0.35".to_string())
                .with_metadata("instance_class".to_string(), "db.t3.micro".to_string())
                .with_metadata("storage_type".to_string(), "gp2".to_string())
                .with_metadata("allocated_storage".to_string(), "20".to_string())
                .with_metadata("multi_az".to_string(), "false".to_string())
                .with_metadata("publicly_accessible".to_string(), "false".to_string())
                .with_metadata("status".to_string(), "available".to_string())
                .with_metadata("backup_retention_period".to_string(), "7".to_string()),
                
            Resource::new("rds-instance".to_string(), "prod-postgres-db-1".to_string())
                .with_metadata("db_name".to_string(), "prod-postgres-db-1".to_string())
                .with_metadata("engine".to_string(), "postgres".to_string())
                .with_metadata("engine_version".to_string(), "15.4".to_string())
                .with_metadata("instance_class".to_string(), "db.r5.large".to_string())
                .with_metadata("storage_type".to_string(), "io1".to_string())
                .with_metadata("allocated_storage".to_string(), "100".to_string())
                .with_metadata("multi_az".to_string(), "true".to_string())
                .with_metadata("publicly_accessible".to_string(), "false".to_string())
                .with_metadata("status".to_string(), "available".to_string())
                .with_metadata("backup_retention_period".to_string(), "30".to_string())
                .with_metadata("deletion_protection".to_string(), "true".to_string()),
                
            Resource::new("rds-instance".to_string(), "dev-mariadb-cluster-1".to_string())
                .with_metadata("db_name".to_string(), "dev-mariadb-cluster-1".to_string())
                .with_metadata("engine".to_string(), "mariadb".to_string())
                .with_metadata("engine_version".to_string(), "10.11.6".to_string())
                .with_metadata("instance_class".to_string(), "db.r6g.2xlarge".to_string())
                .with_metadata("storage_type".to_string(), "aurora".to_string())
                .with_metadata("allocated_storage".to_string(), "0".to_string()) // Aurora scales automatically
                .with_metadata("multi_az".to_string(), "true".to_string())
                .with_metadata("publicly_accessible".to_string(), "false".to_string())
                .with_metadata("status".to_string(), "available".to_string())
                .with_metadata("backup_retention_period".to_string(), "1".to_string())
                .with_metadata("cluster_id".to_string(), "dev-mariadb-cluster".to_string()),
        ])
    }
}

/// Real AWS RDS discovery implementation
pub struct RdsDiscoveryImpl {
    client: aws_sdk_rds::Client,
    config: AwsProviderConfig,
}

impl RdsDiscoveryImpl {
    pub async fn new(config: AwsProviderConfig) -> Result<Self> {
        info!("Creating AWS RDS client for region: {}", config.region);
        
        let mut aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_sdk_rds::config::Region::new(config.region.clone()));
            
        // Set custom endpoint if provided (for LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url);
        }
        
        let sdk_config = aws_config.load().await;
        let client = aws_sdk_rds::Client::new(&sdk_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl RdsDiscovery for RdsDiscoveryImpl {
    async fn discover_instances(&self) -> Result<Vec<Resource>> {
        info!("Discovering RDS instances in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_db_instances();
            
            if let Some(token) = &next_token {
                request = request.marker(token);
            }
            
                match request.send().await {
                Ok(response) => {
                    if let Some(instances) = response.db_instances() {
                        for instance in instances {
                            let resource = Resource::new("rds-instance".to_string(), 
                                                        instance.db_instance_identifier().unwrap_or("unknown").to_string())
                                .with_metadata("db_name".to_string(), 
                                             instance.db_instance_identifier().unwrap_or("").to_string())
                                .with_metadata("engine".to_string(), 
                                             instance.engine().unwrap_or("").to_string())
                                .with_metadata("engine_version".to_string(), 
                                             instance.engine_version().unwrap_or("").to_string())
                                .with_metadata("instance_class".to_string(), 
                                             instance.db_instance_class().unwrap_or("").to_string())
                                .with_metadata("allocated_storage".to_string(), 
                                             instance.allocated_storage().unwrap_or(0).to_string())
                                .with_metadata("storage_type".to_string(), 
                                             instance.storage_type().unwrap_or("").to_string())
                                .with_metadata("multi_az".to_string(), 
                                             instance.multi_az().unwrap_or(false).to_string())
                                .with_metadata("publicly_accessible".to_string(), 
                                             instance.publicly_accessible().unwrap_or(false).to_string())
                                .with_metadata("status".to_string(), 
                                             instance.db_instance_status().unwrap_or("").to_string())
                                .with_metadata("backup_retention_period".to_string(), 
                                             instance.backup_retention_period().unwrap_or(0).to_string())
                                .with_metadata("deletion_protection".to_string(), 
                                             instance.deletion_protection().unwrap_or(false).to_string());
                            
                            // Add VPC configuration if present
                            if let Some(vpc_security_groups) = instance.vpc_security_groups() {
                                let sg_ids: Vec<String> = vpc_security_groups
                                    .iter()
                                    .filter_map(|sg| sg.vpc_security_group_id())
                                    .map(|id| id.to_string())
                                    .collect();
                                if !sg_ids.is_empty() {
                                    resource.metadata.insert("security_group_ids".to_string(), 
                                                          format!("{:?}", sg_ids));
                                }
                            }
                            
                            if let Some(db_subnet_group) = instance.db_subnet_group() {
                                if let Some(subnet_name) = db_subnet_group.db_subnet_group_name() {
                                    resource.metadata.insert("subnet_group".to_string(), 
                                                          subnet_name.to_string());
                                }
                            }
                            
                            // Add endpoint information if present
                            if let Some(endpoint) = instance.endpoint() {
                                if let Some(address) = endpoint.address() {
                                    resource.metadata.insert("endpoint_address".to_string(), 
                                                          address.to_string());
                                }
                                if let Some(port) = endpoint.port() {
                                    resource.metadata.insert("endpoint_port".to_string(), 
                                                          port.to_string());
                                }
                            }
                            
                            // Add availability zone
                            if let Some(az) = instance.availability_zone() {
                                resource.metadata.insert("availability_zone".to_string(), 
                                                      az.to_string());
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    next_token = response.marker().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to describe RDS instances: {}", e);
                    return Err(anyhow::anyhow!("RDS discovery failed: {}", e));
                }
            }
        }
        
        // Also discover RDS clusters (Aurora)
        match self.client.describe_db_clusters().send().await {
            Ok(response) => {
                if let Some(clusters) = response.db_clusters() {
                    for cluster in clusters.iter() {
                        let resource = Resource::new("rds-cluster".to_string(), 
                                                    cluster.db_cluster_identifier().unwrap_or("unknown").to_string())
                                .with_metadata("cluster_name".to_string(), 
                                             cluster.db_cluster_identifier().unwrap_or("").to_string())
                                .with_metadata("engine".to_string(), 
                                             cluster.engine().unwrap_or("").to_string())
                                .with_metadata("engine_version".to_string(), 
                                             cluster.engine_version().unwrap_or("").to_string())
                                .with_metadata("status".to_string(), 
                                             cluster.status().unwrap_or("").to_string())
                                .with_metadata("backup_retention_period".to_string(), 
                                             cluster.backup_retention_period().unwrap_or(0).to_string())
                                .with_metadata("deletion_protection".to_string(), 
                                             cluster.deletion_protection().unwrap_or(false).to_string())
                                .with_metadata("multi_az".to_string(), 
                                             cluster.multi_az().unwrap_or(false).to_string())
                                .with_metadata("scalable".to_string(), "true".to_string());
                        
                        // Add cluster members
                        if let Some(members) = cluster.db_cluster_members() {
                            let member_names: Vec<String> = members
                                .iter()
                                .filter_map(|m| m.db_instance_identifier())
                                .map(|id| id.to_string())
                                .collect();
                            if !member_names.is_empty() {
                                resource.metadata.insert("cluster_members".to_string(), 
                                                      format!("{:?}", member_names));
                            }
                        }
                        
                        // Add endpoint information
                        if let Some(endpoint) = cluster.endpoint() {
                            resource.metadata.insert("endpoint_address".to_string(), 
                                                  endpoint.to_string());
                        }
                        
                        if let Some(reader_endpoint) = cluster.reader_endpoint() {
                            resource.metadata.insert("reader_endpoint".to_string(), 
                                                  reader_endpoint.to_string());
                        }
                        
                        resources.push(resource);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to describe RDS clusters: {}", e);
                // Continue with instances even if cluster discovery fails
            }
        }
        
        info!("Discovered {} RDS resources (instances + clusters)", resources.len());
        Ok(resources)
    }
}