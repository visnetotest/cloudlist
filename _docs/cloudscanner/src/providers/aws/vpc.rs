use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use super::config::AwsConfig;

/// VPC and networking resources discovery trait
#[async_trait]
pub trait VpcDiscovery: Send + Sync {
    async fn discover_vpcs(&self) -> Result<Vec<Resource>>;
    async fn discover_subnets(&self) -> Result<Vec<Resource>>;
    async fn discover_security_groups(&self) -> Result<Vec<Resource>>;
    async fn discover_route_tables(&self) -> Result<Vec<Resource>>;
    async fn discover_internet_gateways(&self) -> Result<Vec<Resource>>;
    async fn discover_nat_gateways(&self) -> Result<Vec<Resource>>;
}

/// Mock VPC discovery for testing
pub struct MockVpcDiscovery;

#[async_trait]
impl VpcDiscovery for MockVpcDiscovery {
    async fn discover_vpcs(&self) -> Result<Vec<Resource>> {
        info!("Using mock VPC discovery");
        
        Ok(vec![
            Resource::new("vpc".to_string(), "vpc-12345678".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("cidr_block".to_string(), "10.0.0.0/16".to_string())
                .with_metadata("state".to_string(), "available".to_string())
                .with_metadata("is_default".to_string(), "false".to_string())
                .with_metadata("instance_tenancy".to_string(), "default".to_string())
                .with_metadata("dns_hostnames".to_string(), "enabled".to_string())
                .with_metadata("dns_resolution".to_string(), "enabled".to_string()),
                
            Resource::new("vpc".to_string(), "vpc-87654321".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-87654321".to_string())
                .with_metadata("cidr_block".to_string(), "172.16.0.0/12".to_string())
                .with_metadata("state".to_string(), "available".to_string())
                .with_metadata("is_default".to_string(), "true".to_string())
                .with_metadata("instance_tenancy".to_string(), "default".to_string())
                .with_metadata("dns_hostnames".to_string(), "enabled".to_string())
                .with_metadata("dns_resolution".to_string(), "enabled".to_string()),
        ])
    }
    
    async fn discover_subnets(&self) -> Result<Vec<Resource>> {
        info!("Using mock subnet discovery");
        
        Ok(vec![
            Resource::new("subnet".to_string(), "subnet-11111111".to_string())
                .with_metadata("subnet_id".to_string(), "subnet-11111111".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("cidr_block".to_string(), "10.0.1.0/24".to_string())
                .with_metadata("availability_zone".to_string(), "us-east-1a".to_string())
                .with_metadata("available_ip_count".to_string(), "251".to_string())
                .with_metadata("map_public_ip".to_string(), "false".to_string())
                .with_metadata("state".to_string(), "available".to_string()),
                
            Resource::new("subnet".to_string(), "subnet-22222222".to_string())
                .with_metadata("subnet_id".to_string(), "subnet-22222222".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("cidr_block".to_string(), "10.0.2.0/24".to_string())
                .with_metadata("availability_zone".to_string(), "us-east-1b".to_string())
                .with_metadata("available_ip_count".to_string(), "251".to_string())
                .with_metadata("map_public_ip".to_string(), "true".to_string())
                .with_metadata("state".to_string(), "available".to_string()),
        ])
    }
    
    async fn discover_security_groups(&self) -> Result<Vec<Resource>> {
        info!("Using mock security group discovery");
        
        Ok(vec![
            Resource::new("security-group".to_string(), "sg-aaaaaaaa".to_string())
                .with_metadata("group_id".to_string(), "sg-aaaaaaaa".to_string())
                .with_metadata("group_name".to_string(), "web-servers".to_string())
                .with_metadata("description".to_string(), "Security group for web servers".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("ingress_rules_count".to_string(), "2".to_string())
                .with_metadata("egress_rules_count".to_string(), "1".to_string()),
                
            Resource::new("security-group".to_string(), "sg-bbbbbbbb".to_string())
                .with_metadata("group_id".to_string(), "sg-bbbbbbbb".to_string())
                .with_metadata("group_name".to_string(), "database-servers".to_string())
                .with_metadata("description".to_string(), "Security group for database servers".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("ingress_rules_count".to_string(), "1".to_string())
                .with_metadata("egress_rules_count".to_string(), "1".to_string()),
        ])
    }
    
    async fn discover_route_tables(&self) -> Result<Vec<Resource>> {
        info!("Using mock route table discovery");
        
        Ok(vec![
            Resource::new("route-table".to_string(), "rtb-cccccccc".to_string())
                .with_metadata("route_table_id".to_string(), "rtb-cccccccc".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("routes_count".to_string(), "3".to_string())
                .with_metadata("associations_count".to_string(), "2".to_string())
                .with_metadata("propagating_vgws_count".to_string(), "0".to_string())
                .with_metadata("main".to_string(), "false".to_string()),
        ])
    }
    
    async fn discover_internet_gateways(&self) -> Result<Vec<Resource>> {
        info!("Using mock internet gateway discovery");
        
        Ok(vec![
            Resource::new("internet-gateway".to_string(), "igw-dddddddd".to_string())
                .with_metadata("gateway_id".to_string(), "igw-dddddddd".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("state".to_string(), "available".to_string())
                .with_metadata("attachments_count".to_string(), "1".to_string()),
        ])
    }
    
    async fn discover_nat_gateways(&self) -> Result<Vec<Resource>> {
        info!("Using mock NAT gateway discovery");
        
        Ok(vec![
            Resource::new("nat-gateway".to_string(), "nat-eeeeeeee".to_string())
                .with_metadata("nat_gateway_id".to_string(), "nat-eeeeeeee".to_string())
                .with_metadata("vpc_id".to_string(), "vpc-12345678".to_string())
                .with_metadata("subnet_id".to_string(), "subnet-22222222".to_string())
                .with_metadata("state".to_string(), "available".to_string())
                .with_metadata("allocation_id".to_string(), "eipalloc-ffffffff".to_string())
                .with_metadata("connectivity_type".to_string(), "public".to_string()),
        ])
    }
}

/// Real AWS VPC discovery implementation
pub struct VpcDiscoveryImpl {
    client: aws_sdk_ec2::Client,
    config: AwsConfig,
}

impl VpcDiscoveryImpl {
    pub async fn new(config: AwsConfig) -> Result<Self> {
        info!("Creating AWS EC2 client for VPC discovery in region: {}", config.region);
        
        let mut aws_config = aws_config::from_env()
            .region(aws_sdk_ec2::config::Region::new(config.region.clone()));
            
        // Set custom endpoint if provided (for LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            aws_config = aws_config.endpoint_url(endpoint_url);
        }
        
        let client = aws_sdk_ec2::Client::new(&aws_config);
        
        Ok(Self { client, config })
    }
}

#[async_trait]
impl VpcDiscovery for VpcDiscoveryImpl {
    async fn discover_vpcs(&self) -> Result<Vec<Resource>> {
        info!("Discovering VPCs in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_vpcs();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(vpcs) = response.vpcs() {
                        for vpc in vpcs {
                            let resource = Resource::new("vpc".to_string(), 
                                                        vpc.vpc_id().unwrap_or("unknown").to_string())
                                .with_metadata("vpc_id".to_string(), 
                                             vpc.vpc_id().unwrap_or("").to_string())
                                .with_metadata("cidr_block".to_string(), 
                                             vpc.cidr_block().unwrap_or("").to_string())
                                .with_metadata("state".to_string(), 
                                             vpc.state().unwrap_or("").to_string())
                                .with_metadata("is_default".to_string(), 
                                             vpc.is_default().unwrap_or(false).to_string())
                                .with_metadata("instance_tenancy".to_string(), 
                                             vpc.instance_tenancy().unwrap_or("").to_string())
                                .with_metadata("dhcp_options_id".to_string(), 
                                             vpc.dhcp_options_id().unwrap_or("").to_string());
                            
                            // Add DNS configuration
                            if let Some(dhcp_options) = vpc.dhcp_options_id() {
                                resource.metadata.insert("dhcp_options_id".to_string(), 
                                                      dhcp_options.to_string());
                            }
                            
                            // Add tags
                            if let Some(tags) = vpc.tags() {
                                let tag_map: std::collections::HashMap<String, String> = tags
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
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to describe VPCs: {}", e);
                    return Err(anyhow::anyhow!("VPC discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} VPCs", resources.len());
        Ok(resources)
    }
    
    async fn discover_subnets(&self) -> Result<Vec<Resource>> {
        info!("Discovering subnets in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_subnets();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(subnets) = response.subnets() {
                        for subnet in subnets {
                            let resource = Resource::new("subnet".to_string(), 
                                                        subnet.subnet_id().unwrap_or("unknown").to_string())
                                .with_metadata("subnet_id".to_string(), 
                                             subnet.subnet_id().unwrap_or("").to_string())
                                .with_metadata("vpc_id".to_string(), 
                                             subnet.vpc_id().unwrap_or("").to_string())
                                .with_metadata("cidr_block".to_string(), 
                                             subnet.cidr_block().unwrap_or("").to_string())
                                .with_metadata("availability_zone".to_string(), 
                                             subnet.availability_zone().unwrap_or("").to_string())
                                .with_metadata("availability_zone_id".to_string(), 
                                             subnet.availability_zone_id().unwrap_or("").to_string())
                                .with_metadata("available_ip_count".to_string(), 
                                             subnet.available_ip_address_count().unwrap_or(0).to_string())
                                .with_metadata("map_public_ip".to_string(), 
                                             subnet.map_public_ip_on_launch().unwrap_or(false).to_string())
                                .with_metadata("assign_ipv6".to_string(), 
                                             subnet.assign_ipv6_address_on_creation().unwrap_or(false).to_string())
                                .with_metadata("state".to_string(), 
                                             subnet.state().unwrap_or("").to_string());
                            
                            // Add subnet ARN
                            if let Some(subnet_arn) = subnet.subnet_arn() {
                                resource.metadata.insert("subnet_arn".to_string(), 
                                                      subnet_arn.to_string());
                            }
                            
                            // Add IPv6 CIDR if present
                            if let Some(ipv6_cidr) = subnet.ipv6_cidr_block_association_set() {
                                if let Some(first_assoc) = ipv6_cidr.first() {
                                    if let Some(cidr) = first_assoc.ipv6_cidr_block() {
                                        resource.metadata.insert("ipv6_cidr_block".to_string(), 
                                                              cidr.to_string());
                                    }
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
                    error!("Failed to describe subnets: {}", e);
                    return Err(anyhow::anyhow!("Subnet discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} subnets", resources.len());
        Ok(resources)
    }
    
    async fn discover_security_groups(&self) -> Result<Vec<Resource>> {
        info!("Discovering security groups in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_security_groups();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(sgs) = response.security_groups() {
                        for sg in sgs {
                            let resource = Resource::new("security-group".to_string(), 
                                                        sg.group_id().unwrap_or("unknown").to_string())
                                .with_metadata("group_id".to_string(), 
                                             sg.group_id().unwrap_or("").to_string())
                                .with_metadata("group_name".to_string(), 
                                             sg.group_name().unwrap_or("").to_string())
                                .with_metadata("description".to_string(), 
                                             sg.description().unwrap_or("").to_string())
                                .with_metadata("vpc_id".to_string(), 
                                             sg.vpc_id().unwrap_or("").to_string())
                                .with_metadata("owner_id".to_string(), 
                                             sg.owner_id().unwrap_or("").to_string());
                            
                            // Count rules
                            if let Some(ingress) = sg.ip_permissions() {
                                resource.metadata.insert("ingress_rules_count".to_string(), 
                                                      ingress.len().to_string());
                            }
                            
                            if let Some(egress) = sg.ip_permissions_egress() {
                                resource.metadata.insert("egress_rules_count".to_string(), 
                                                      egress.len().to_string());
                            }
                            
                            // Add tags
                            if let Some(tags) = sg.tags() {
                                let tag_map: std::collections::HashMap<String, String> = tags
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
                    
                    next_token = response.next_token().map(|s| s.to_string());
                    if next_token.is_none() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to describe security groups: {}", e);
                    return Err(anyhow::anyhow!("Security group discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} security groups", resources.len());
        Ok(resources)
    }
    
    async fn discover_route_tables(&self) -> Result<Vec<Resource>> {
        info!("Discovering route tables in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_route_tables();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(rts) = response.route_tables() {
                        for rt in rts {
                            let resource = Resource::new("route-table".to_string(), 
                                                        rt.route_table_id().unwrap_or("unknown").to_string())
                                .with_metadata("route_table_id".to_string(), 
                                             rt.route_table_id().unwrap_or("").to_string())
                                .with_metadata("vpc_id".to_string(), 
                                             rt.vpc_id().unwrap_or("").to_string())
                                .with_metadata("owner_id".to_string(), 
                                             rt.owner_id().unwrap_or("").to_string())
                                .with_metadata("propagating_vgws_count".to_string(), 
                                             rt.propagating_vgws().map(|v| v.len()).unwrap_or(0).to_string());
                            
                            // Count routes and associations
                            if let Some(routes) = rt.routes() {
                                resource.metadata.insert("routes_count".to_string(), 
                                                      routes.len().to_string());
                            }
                            
                            if let Some(associations) = rt.associations() {
                                resource.metadata.insert("associations_count".to_string(), 
                                                      associations.len().to_string());
                                
                                // Check if main route table
                                for assoc in associations {
                                    if assoc.main().unwrap_or(false) {
                                        resource.metadata.insert("main".to_string(), "true".to_string());
                                        break;
                                    }
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
                    error!("Failed to describe route tables: {}", e);
                    return Err(anyhow::anyhow!("Route table discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} route tables", resources.len());
        Ok(resources)
    }
    
    async fn discover_internet_gateways(&self) -> Result<Vec<Resource>> {
        info!("Discovering internet gateways in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_internet_gateways();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(igws) = response.internet_gateways() {
                        for igw in igws {
                            let resource = Resource::new("internet-gateway".to_string(), 
                                                        igw.internet_gateway_id().unwrap_or("unknown").to_string())
                                .with_metadata("gateway_id".to_string(), 
                                             igw.internet_gateway_id().unwrap_or("").to_string())
                                .with_metadata("owner_id".to_string(), 
                                             igw.owner_id().unwrap_or("").to_string())
                                .with_metadata("state".to_string(), 
                                             igw.state().unwrap_or("").to_string());
                            
                            // Count attachments
                            if let Some(attachments) = igw.attachments() {
                                resource.metadata.insert("attachments_count".to_string(), 
                                                      attachments.len().to_string());
                                
                                for attachment in attachments {
                                    if let Some(vpc_id) = attachment.vpc_id() {
                                        resource.metadata.insert("vpc_id".to_string(), 
                                                              vpc_id.to_string());
                                        resource.metadata.insert("attachment_state".to_string(), 
                                                              attachment.state().unwrap_or("").to_string());
                                    }
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
                    error!("Failed to describe internet gateways: {}", e);
                    return Err(anyhow::anyhow!("Internet gateway discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} internet gateways", resources.len());
        Ok(resources)
    }
    
    async fn discover_nat_gateways(&self) -> Result<Vec<Resource>> {
        info!("Discovering NAT gateways in region: {}", self.config.region);
        
        let mut resources = Vec::new();
        let mut next_token: Option<String> = None;
        
        loop {
            let mut request = self.client.describe_nat_gateways();
            
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if let Some(nat_gws) = response.nat_gateways() {
                        for nat_gw in nat_gws {
                            let resource = Resource::new("nat-gateway".to_string(), 
                                                        nat_gw.nat_gateway_id().unwrap_or("unknown").to_string())
                                .with_metadata("nat_gateway_id".to_string(), 
                                             nat_gw.nat_gateway_id().unwrap_or("").to_string())
                                .with_metadata("vpc_id".to_string(), 
                                             nat_gw.vpc_id().unwrap_or("").to_string())
                                .with_metadata("subnet_id".to_string(), 
                                             nat_gw.subnet_id().unwrap_or("").to_string())
                                .with_metadata("state".to_string(), 
                                             nat_gw.state().unwrap_or("").to_string())
                                .with_metadata("failure_code".to_string(), 
                                             nat_gw.failure_code().unwrap_or("").to_string())
                                .with_metadata("failure_message".to_string(), 
                                             nat_gw.failure_message().unwrap_or("").to_string());
                            
                            // Add NAT gateway details
                            if let Some(nat_gw_addr) = nat_gw.nat_gateway_address() {
                                if let Some(allocation_id) = nat_gw_addr.allocation_id() {
                                    resource.metadata.insert("allocation_id".to_string(), 
                                                          allocation_id.to_string());
                                }
                                if let Some(ip) = nat_gw_addr.public_ip() {
                                    resource.metadata.insert("public_ip".to_string(), 
                                                          ip.to_string());
                                }
                                if let Some(private_ip) = nat_gw_addr.private_ip() {
                                    resource.metadata.insert("private_ip".to_string(), 
                                                          private_ip.to_string());
                                }
                            }
                            
                            // Add connectivity type
                            if let Some(connectivity) = nat_gw.connectivity_type() {
                                resource.metadata.insert("connectivity_type".to_string(), 
                                                      connectivity.to_string());
                            }
                            
                            // Add create time
                            if let Some(create_time) = nat_gw.create_time() {
                                resource.metadata.insert("create_time".to_string(), 
                                                      create_time.to_string());
                            }
                            
                            // Add delete time
                            if let Some(delete_time) = nat_gw.delete_time() {
                                resource.metadata.insert("delete_time".to_string(), 
                                                      delete_time.to_string());
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
                    error!("Failed to describe NAT gateways: {}", e);
                    return Err(anyhow::anyhow!("NAT gateway discovery failed: {}", e));
                }
            }
        }
        
        info!("Discovered {} NAT gateways", resources.len());
        Ok(resources)
    }
}