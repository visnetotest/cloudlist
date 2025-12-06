use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use super::config::AwsConfig;

/// IAM roles and policies discovery trait
#[async_trait]
pub trait IamDiscovery: Send + Sync {
    async fn discover_roles(&self) -> Result<Vec<Resource>>;
    async fn discover_policies(&self) -> Result<Vec<Resource>>;
    async fn discover_users(&self) -> Result<Vec<Resource>>;
    async fn discover_groups(&self) -> Result<Vec<Resource>>;
}

/// Mock IAM discovery for testing
pub struct MockIamDiscovery;

#[async_trait]
impl IamDiscovery for MockIamDiscovery {
    async fn discover_roles(&self) -> Result<Vec<Resource>> {
        info!("Using mock IAM role discovery");
        
        Ok(vec![
            Resource::new("iam-role".to_string(), "arn:aws:iam::123456789012:role/EC2InstanceRole".to_string())
                .with_metadata("role_name".to_string(), "EC2InstanceRole".to_string())
                .with_metadata("role_id".to_string(), "AROAEXAMPLEROLE".to_string())
                .with_metadata("path".to_string(), "/".to_string())
                .with_metadata("description".to_string(), "Role for EC2 instances".to_string())
                .with_metadata("max_session_duration".to_string(), "3600".to_string())
                .with_metadata("create_date".to_string(), "2023-01-15T10:30:00Z".to_string())
                .with_metadata("assume_role_policy_count".to_string(), "2".to_string())
                .with_metadata("inline_policy_count".to_string(), "1".to_string())
                .with_metadata("attached_policy_count".to_string(), "3".to_string()),
                
            Resource::new("iam-role".to_string(), "arn:aws:iam::123456789012:role/LambdaExecutionRole".to_string())
                .with_metadata("role_name".to_string(), "LambdaExecutionRole".to_string())
                .with_metadata("role_id".to_string(), "AROAXAMPLEROLE".to_string())
                .with_metadata("path".to_string(), "/service-role/".to_string())
                .with_metadata("description".to_string(), "Role for Lambda function execution".to_string())
                .with_metadata("max_session_duration".to_string(), "3600".to_string())
                .with_metadata("create_date".to_string(), "2023-02-20T14:15:00Z".to_string())
                .with_metadata("assume_role_policy_count".to_string(), "1".to_string())
                .with_metadata("inline_policy_count".to_string(), "0".to_string())
                .with_metadata("attached_policy_count".to_string(), "2".to_string()),
        ])
    }
    
    async fn discover_policies(&self) -> Result<Vec<Resource>> {
        info!("Using mock IAM policy discovery");
        
        Ok(vec![
            Resource::new("iam-policy".to_string(), "arn:aws:iam::123456789012:policy/EC2AccessPolicy".to_string())
                .with_metadata("policy_name".to_string(), "EC2AccessPolicy".to_string())
                .with_metadata("policy_id".to_string(), "ANPAEXAMPLEPOLICY".to_string())
                .with_metadata("path".to_string(), "/".to_string())
                .with_metadata("description".to_string(), "Policy for EC2 access".to_string())
                .with_metadata("create_date".to_string(), "2023-01-10T09:00:00Z".to_string())
                .with_metadata("update_date".to_string(), "2023-06-15T11:30:00Z".to_string())
                .with_metadata("default_version_id".to_string(), "v1".to_string())
                .with_metadata("attachment_count".to_string(), "5".to_string())
                .with_metadata("permissions_count".to_string(), "12".to_string()),
                
            Resource::new("iam-policy".to_string(), "arn:aws:iam::123456789012:policy/S3ReadOnlyPolicy".to_string())
                .with_metadata("policy_name".to_string(), "S3ReadOnlyPolicy".to_string())
                .with_metadata("policy_id".to_string(), "ANPBEXAMPLEPOLICY".to_string())
                .with_metadata("path".to_string(), "/application/".to_string())
                .with_metadata("description".to_string(), "Read-only access to S3 buckets".to_string())
                .with_metadata("create_date".to_string(), "2023-02-05T16:45:00Z".to_string())
                .with_metadata("update_date".to_string(), "2023-07-20T13:20:00Z".to_string())
                .with_metadata("default_version_id".to_string(), "v2".to_string())
                .with_metadata("attachment_count".to_string(), "3".to_string())
                .with_metadata("permissions_count".to_string(), "8".to_string()),
        ])
    }
    
    async fn discover_users(&self) -> Result<Vec<Resource>> {
        info!("Using mock IAM user discovery");
        
        Ok(vec![
            Resource::new("iam-user".to_string(), "arn:aws:iam::123456789012:user/admin-user".to_string())
                .with_metadata("user_name".to_string(), "admin-user".to_string())
                .with_metadata("user_id".to_string(), "AIDAEXAMPLEUSER".to_string())
                .with_metadata("path".to_string(), "/".to_string())
                .with_metadata("create_date".to_string(), "2023-01-01T00:00:00Z".to_string())
                .with_metadata("password_last_used".to_string(), "2023-12-01T10:00:00Z".to_string())
                .with_metadata("mfa_active".to_string(), "true".to_string())
                .with_metadata("access_keys_count".to_string(), "2".to_string())
                .with_metadata("attached_policies_count".to_string(), "4".to_string())
                .with_metadata("groups_count".to_string(), "2".to_string()),
                
            Resource::new("iam-user".to_string(), "arn:aws:iam::123456789012:user/service-account".to_string())
                .with_metadata("user_name".to_string(), "service-account".to_string())
                .with_metadata("user_id".to_string(), "AIDBEXAMPLEUSER".to_string())
                .with_metadata("path".to_string(), "/service/".to_string())
                .with_metadata("create_date".to_string(), "2023-03-15T12:30:00Z".to_string())
                .with_metadata("password_last_used".to_string(), "never".to_string())
                .with_metadata("mfa_active".to_string(), "false".to_string())
                .with_metadata("access_keys_count".to_string(), "1".to_string())
                .with_metadata("attached_policies_count".to_string(), "2".to_string())
                .with_metadata("groups_count".to_string(), "1".to_string()),
        ])
    }
    
    async fn discover_groups(&self) -> Result<Vec<Resource>> {
        info!("Using mock IAM group discovery");
        
        Ok(vec![
            Resource::new("iam-group".to_string(), "arn:aws:iam::123456789012:group/administrators".to_string())
                .with_metadata("group_name".to_string(), "administrators".to_string())
                .with_metadata("group_id".to_string(), "AGPAEXAMPLEGROUP".to_string())
                .with_metadata("path".to_string(), "/".to_string())
                .with_metadata("create_date".to_string(), "2023-01-01T00:00:00Z".to_string())
                .with_metadata("users_count".to_string(), "3".to_string())
                .with_metadata("attached_policies_count".to_string(), "5".to_string())
                .with_metadata("inline_policies_count".to_string(), "1".to_string()),
                
            Resource::new("iam-group".to_string(), "arn:aws:iam::123456789012:group/developers".to_string())
                .with_metadata("group_name".to_string(), "developers".to_string())
                .with_metadata("group_id".to_string(), "AGPBEXAMPLEGROUP".to_string())
                .with_metadata("path".to_string(), "/teams/".to_string())
                .with_metadata("create_date".to_string(), "2023-02-01T00:00:00Z".to_string())
                .with_metadata("users_count".to_string(), "8".to_string())
                .with_metadata("attached_policies_count".to_string(), "3".to_string())
                .with_metadata("inline_policies_count".to_string(), "0".to_string()),
        ])
    }
}

/// Real AWS IAM discovery implementation
pub struct IamDiscoveryImpl {
    client: aws_sdk_iam::Client,
    config: AwsConfig,
}

impl IamDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS IAM client for region: {}", config.region);
        
        let mut aws_config = aws_config::from_env()
            .region(aws_sdk_iam::config::Region::new(config.region.clone()));
            
        // Set custom endpoint if provided (for LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url);
        }
        
        let client = aws_sdk_iam::Client::new(&aws_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl IamDiscovery for IamDiscoveryImpl {
    async fn discover_roles(&self) -> Result<Vec<Resource>> {
        info!("Discovering IAM roles");
        
        let mut resources = Vec::new();
        let mut marker: Option<String> = None;
        
        loop {
            let mut request = self.client.list_roles();
            
            if let Some(m) = &marker {
                request = request.marker(m);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(roles) = response.roles() {
                        for role in roles {
                            let resource = Resource::new("iam-role".to_string(), 
                                                        role.arn().unwrap_or("unknown").to_string())
                                .with_metadata("role_name".to_string(), 
                                             role.role_name().unwrap_or("").to_string())
                                .with_metadata("role_id".to_string(), 
                                             role.role_id().unwrap_or("").to_string())
                                .with_metadata("path".to_string(), 
                                             role.path().unwrap_or("").to_string())
                                .with_metadata("description".to_string(), 
                                             role.description().unwrap_or("").to_string())
                                .with_metadata("max_session_duration".to_string(), 
                                             role.max_session_duration().unwrap_or(0).to_string())
                                .with_metadata("create_date".to_string(), 
                                             role.create_date().unwrap_or("").to_string());
                            
                            // Count assume role policies
                            if let Some(assume_policy) = role.assume_role_policy_document() {
                                resource.metadata.insert("assume_role_policy_size".to_string(), 
                                                      assume_policy.len().to_string());
                            }
                            
                            // Get role policies
                            match self.client.list_attached_role_policies()
                                .set_role_name(role.role_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(policy_response) => {
                                    if let Some(policies) = policy_response.attached_policies() {
                                        resource.metadata.insert("attached_policy_count".to_string(), 
                                                              policies.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list attached role policies: {}", e);
                                }
                            }
                            
                            // Get inline policies
                            match self.client.list_role_policies()
                                .set_role_name(role.role_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(inline_response) => {
                                    if let Some(policies) = inline_response.policy_names() {
                                        resource.metadata.insert("inline_policy_count".to_string(), 
                                                              policies.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list role inline policies: {}", e);
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    marker = response.marker().map(|s| s.to_string());
                    if marker.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list IAM roles: {}", e);
                    return Err(anyhow::anyhow!("IAM role discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} IAM roles", resources.len());
        Ok(resources)
    }
    
    async fn discover_policies(&self) -> Result<Vec<Resource>> {
        info!("Discovering IAM policies");
        
        let mut resources = Vec::new();
        let mut marker: Option<String> = None;
        let mut only_attached = false;
        
        loop {
            let mut request = self.client.list_policies();
            
            if let Some(m) = &marker {
                request = request.marker(m);
            }
            
            if only_attached {
                request = request.only_attached(true);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(policies) = response.policies() {
                        for policy in policies {
                            let resource = Resource::new("iam-policy".to_string(), 
                                                        policy.arn().unwrap_or("unknown").to_string())
                                .with_metadata("policy_name".to_string(), 
                                             policy.policy_name().unwrap_or("").to_string())
                                .with_metadata("policy_id".to_string(), 
                                             policy.policy_id().unwrap_or("").to_string())
                                .with_metadata("path".to_string(), 
                                             policy.path().unwrap_or("").to_string())
                                .with_metadata("description".to_string(), 
                                             policy.description().unwrap_or("").to_string())
                                .with_metadata("create_date".to_string(), 
                                             policy.create_date().unwrap_or("").to_string())
                                .with_metadata("update_date".to_string(), 
                                             policy.update_date().unwrap_or("").to_string())
                                .with_metadata("default_version_id".to_string(), 
                                             policy.default_version_id().unwrap_or("").to_string())
                                .with_metadata("attachment_count".to_string(), 
                                             policy.attachment_count().unwrap_or(0).to_string())
                                .with_metadata("permissions_boundary_usage_count".to_string(), 
                                             policy.permissions_boundary_usage_count().unwrap_or(0).to_string())
                                .with_metadata("is_attachable".to_string(), 
                                             policy.is_attachable().unwrap_or(false).to_string());
                            
                            // Get policy versions
                            match self.client.list_policy_versions()
                                .set_policy_arn(policy.arn().unwrap_or(""))
                                .send()
                                .await {
                                Ok(versions_response) => {
                                    if let Some(versions) = versions_response.versions() {
                                        resource.metadata.insert("versions_count".to_string(), 
                                                              versions.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list policy versions: {}", e);
                                }
                            }
                            
                            // Get policy document for default version
                            if let Some(default_version) = policy.default_version_id() {
                                match self.client.get_policy()
                                    .set_policy_arn(policy.arn().unwrap_or(""))
                                    .set_version_id(default_version)
                                    .send()
                                    .await {
                                    Ok(doc_response) => {
                                        if let Some(policy_version) = doc_response.policy_version() {
                                            if let Some(document) = policy_version.document() {
                                                resource.metadata.insert("document_size".to_string(), 
                                                                      document.len().to_string());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to get policy document: {}", e);
                                    }
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    marker = response.marker().map(|s| s.to_string());
                    if marker.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list IAM policies: {}", e);
                    return Err(anyhow::anyhow!("IAM policy discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} IAM policies", resources.len());
        Ok(resources)
    }
    
    async fn discover_users(&self) -> Result<Vec<Resource>> {
        info!("Discovering IAM users");
        
        let mut resources = Vec::new();
        let mut marker: Option<String> = None;
        
        loop {
            let mut request = self.client.list_users();
            
            if let Some(m) = &marker {
                request = request.marker(m);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(users) = response.users() {
                        for user in users {
                            let resource = Resource::new("iam-user".to_string(), 
                                                        user.arn().unwrap_or("unknown").to_string())
                                .with_metadata("user_name".to_string(), 
                                             user.user_name().unwrap_or("").to_string())
                                .with_metadata("user_id".to_string(), 
                                             user.user_id().unwrap_or("").to_string())
                                .with_metadata("path".to_string(), 
                                             user.path().unwrap_or("").to_string())
                                .with_metadata("create_date".to_string(), 
                                             user.create_date().unwrap_or("").to_string())
                                .with_metadata("password_last_used".to_string(), 
                                             user.password_last_used().unwrap_or("").to_string());
                            
                            // Get access keys
                            match self.client.list_access_keys()
                                .set_user_name(user.user_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(keys_response) => {
                                    if let Some(keys) = keys_response.access_key_metadata() {
                                        resource.metadata.insert("access_keys_count".to_string(), 
                                                              keys.len().to_string());
                                        
                                        // Check for active keys
                                        let active_keys = keys.iter()
                                            .filter(|key| key.status().unwrap_or("") == "Active")
                                            .count();
                                        resource.metadata.insert("active_access_keys_count".to_string(), 
                                                              active_keys.to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list access keys: {}", e);
                                }
                            }
                            
                            // Get attached policies
                            match self.client.list_attached_user_policies()
                                .set_user_name(user.user_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(policies_response) => {
                                    if let Some(policies) = policies_response.attached_policies() {
                                        resource.metadata.insert("attached_policies_count".to_string(), 
                                                              policies.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list attached user policies: {}", e);
                                }
                            }
                            
                            // Get group memberships
                            match self.client.list_groups_for_user()
                                .set_user_name(user.user_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(groups_response) => {
                                    if let Some(groups) = groups_response.groups() {
                                        resource.metadata.insert("groups_count".to_string(), 
                                                              groups.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list user groups: {}", e);
                                }
                            }
                            
                            // Get MFA devices
                            match self.client.list_mfa_devices()
                                .set_user_name(user.user_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(mfa_response) => {
                                    if let Some(mfa_devices) = mfa_response.mfa_devices() {
                                        resource.metadata.insert("mfa_devices_count".to_string(), 
                                                              mfa_devices.len().to_string());
                                        
                                        let enabled_mfa = mfa_devices.iter()
                                            .any(|device| device.enable_date().is_some());
                                        resource.metadata.insert("mfa_enabled".to_string(), 
                                                              enabled_mfa.to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list MFA devices: {}", e);
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    marker = response.marker().map(|s| s.to_string());
                    if marker.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list IAM users: {}", e);
                    return Err(anyhow::anyhow!("IAM user discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} IAM users", resources.len());
        Ok(resources)
    }
    
    async fn discover_groups(&self) -> Result<Vec<Resource>> {
        info!("Discovering IAM groups");
        
        let mut resources = Vec::new();
        let mut marker: Option<String> = None;
        
        loop {
            let mut request = self.client.list_groups();
            
            if let Some(m) = &marker {
                request = request.marker(m);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(groups) = response.groups() {
                        for group in groups {
                            let resource = Resource::new("iam-group".to_string(), 
                                                        group.arn().unwrap_or("unknown").to_string())
                                .with_metadata("group_name".to_string(), 
                                             group.group_name().unwrap_or("").to_string())
                                .with_metadata("group_id".to_string(), 
                                             group.group_id().unwrap_or("").to_string())
                                .with_metadata("path".to_string(), 
                                             group.path().unwrap_or("").to_string())
                                .with_metadata("create_date".to_string(), 
                                             group.create_date().unwrap_or("").to_string());
                            
                            // Get group policies
                            match self.client.list_attached_group_policies()
                                .set_group_name(group.group_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(policies_response) => {
                                    if let Some(policies) = policies_response.attached_policies() {
                                        resource.metadata.insert("attached_policies_count".to_string(), 
                                                              policies.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list attached group policies: {}", e);
                                }
                            }
                            
                            // Get inline policies
                            match self.client.list_group_policies()
                                .set_group_name(group.group_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(inline_response) => {
                                    if let Some(policies) = inline_response.policy_names() {
                                        resource.metadata.insert("inline_policies_count".to_string(), 
                                                              policies.len().to_string());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to list group inline policies: {}", e);
                                }
                            }
                            
                            // Get group users
                            match self.client.get_group()
                                .set_group_name(group.group_name().unwrap_or(""))
                                .send()
                                .await {
                                Ok(group_response) => {
                                    if let Some(group_detail) = group_response.group() {
                                        if let Some(users) = group_detail.users() {
                                            resource.metadata.insert("users_count".to_string(), 
                                                                  users.len().to_string());
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to get group details: {}", e);
                                }
                            }
                            
                            resources.push(resource);
                        }
                    }
                    
                    marker = response.marker().map(|s| s.to_string());
                    if marker.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list IAM groups: {}", e);
                    return Err(anyhow::anyhow!("IAM group discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} IAM groups", resources.len());
        Ok(resources)
    }
}