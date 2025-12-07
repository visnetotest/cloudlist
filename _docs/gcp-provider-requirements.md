# Technical Requirements: GCP Provider Support

## Overview

This document outlines the technical requirements for implementing Google Cloud Platform (GCP) provider support in CloudScanner. The GCP provider will enable discovery and security assessment of GCP resources, following the same architectural patterns as the existing AWS provider.

## Requirements

### 1. Functional Requirements

#### 1.1 Resource Discovery

The GCP provider must support discovery of the following GCP resource types:

**Core Compute Resources:**
- **Compute Engine**: VM instances, instance templates, instance groups
- **Kubernetes Engine**: GKE clusters, node pools, workloads
- **Cloud Run**: Fully managed services and revisions
- **Cloud Functions**: Function deployments and configurations

**Storage Resources:**
- **Cloud Storage**: Buckets, objects, IAM policies
- **Filestore**: File instances and configurations
- **Persistent Disk**: Disk volumes and snapshots

**Network Resources:**
- **VPC Network**: Networks, subnets, firewall rules
- **Cloud Load Balancing**: Load balancers and backend services
- **Cloud CDN**: CDN configurations and policies

**Database Resources:**
- **Cloud SQL**: Database instances, backups, users
- **Cloud Spanner**: Instances, databases, configurations
- **BigQuery**: Datasets, tables, IAM policies

**Security & IAM Resources:**
- **IAM**: Service accounts, roles, policies
- **Cloud KMS**: Key rings, cryptographic keys
- **Secret Manager**: Secret versions and access policies

**Monitoring & Logging:**
- **Cloud Logging**: Log sinks and routing
- **Cloud Monitoring**: Alert policies and dashboards
- **Cloud Audit Logs**: Audit configuration and access

#### 1.2 Authentication Support

The provider must support multiple GCP authentication methods:

**Primary Authentication:**
- **Service Account Keys**: JSON key file authentication
- **Application Default Credentials**: Automatic credential discovery
- **Workload Identity**: GKE and Cloud Run identity-based auth

**Secondary Authentication:**
- **OAuth2 Flow**: User account authentication for testing
- **Impersonation**: Service account impersonation for privilege escalation

#### 1.3 Regional and Multi-Project Support

- **Multi-Region Discovery**: Support for resources across all GCP regions
- **Multi-Project Support**: Discovery across multiple GCP projects
- **Organization-Level**: Optional organization-wide resource discovery
- **Folder Hierarchy**: Support for GCP folder-based organization

### 2. Technical Requirements

#### 2.1 Provider Implementation

**Core Provider Structure:**
```rust
pub struct GcpProvider {
    config: GcpProviderConfig,
    clients: HashMap<String, GcpClient>,
    project_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GcpProviderConfig {
    pub credentials_path: Option<String>,
    pub project_ids: Vec<String>,
    pub regions: Vec<String>,
    pub organization_id: Option<String>,
    pub include_organizations: bool,
    pub service_account_email: Option<String>,
    pub impersonate_service_account: Option<String>,
}
```

**Required Trait Implementation:**
```rust
#[async_trait]
impl Provider for GcpProvider {
    fn info(&self) -> ProviderInfo;
    fn name(&self) -> String;
    async fn discover(&self) -> Result<Vec<Resource>>;
    async fn health_check(&self) -> Result<bool>;
    async fn cleanup(&self) -> Result<()>;
}
```

#### 2.2 Client Management

**GCP Client Structure:**
```rust
pub struct GcpClient {
    project_id: String,
    compute_client: compute::Client,
    storage_client: storage::Client,
    sql_client: sqladmin::Client,
    iam_client: iam::Client,
    // ... other service clients
}
```

**Client Factory Pattern:**
- Lazy initialization of service clients
- Connection pooling and reuse
- Automatic credential refresh
- Error handling and retry logic

#### 2.3 Resource Model Mapping

**GCP Resource Structure:**
```rust
impl Resource {
    pub fn gcp_compute_instance(project_id: &str, zone: &str, instance_name: &str) -> Self;
    pub fn gcp_storage_bucket(project_id: &str, bucket_name: &str) -> Self;
    pub fn gcp_kms_key(project_id: &str, location: &str, key_ring: &str, key_name: &str) -> Self;
    // ... other resource constructors
}
```

**Metadata Standards:**
```rust
// Standard metadata fields for all GCP resources
metadata.insert("project_id".to_string(), project_id.to_string());
metadata.insert("location".to_string(), location.to_string());
metadata.insert("resource_name".to_string(), resource_name.to_string());
metadata.insert("creation_time".to_string(), creation_time.to_string());
metadata.insert("labels".to_string(), serde_json::to_string(&labels)?);
```

#### 2.4 Configuration Integration

**Provider Configuration:**
```toml
[[provider]]
id = "gcp-provider"
type = "builtin-gcp"
config = {
    credentials_path = "/path/to/service-account.json",
    project_ids = ["project-1", "project-2"],
    regions = ["us-central1", "us-east1", "europe-west1"],
    include_organizations = false,
    impersonate_service_account = "admin@project.iam.gserviceaccount.com"
}
```

**Environment Variable Support:**
- `GOOGLE_APPLICATION_CREDENTIALS`
- `GOOGLE_CLOUD_PROJECT`
- `GOOGLE_CLOUD_QUOTA_PROJECT`
- `GOOGLE_IMPERSONATE_SERVICE_ACCOUNT`

### 3. Security Requirements

#### 3.1 Credential Security

**Secure Credential Handling:**
- No hardcoded credentials in source code
- Support for credential file encryption
- Automatic credential rotation support
- Secure temporary credential handling

**Permission Validation:**
- Minimum required permissions validation
- Credential scope verification
- Service account permission checks
- Organization-level access validation

#### 3.2 Data Protection

**Sensitive Data Handling:**
- Sanitization of sensitive metadata
- Encryption key discovery without key exposure
- IAM policy filtering for sensitive permissions
- Audit log access control

#### 3.3 Network Security

**API Security:**
- TLS 1.2+ for all API communications
- Certificate validation
- Request signing and authentication
- Rate limiting and quota management

### 4. Performance Requirements

#### 4.1 Discovery Performance

**Concurrency Requirements:**
- Parallel discovery across multiple projects
- Concurrent API calls within rate limits
- Async/await implementation throughout
- Configurable concurrency limits

**Rate Limiting:**
- GCP API quota management
- Exponential backoff for rate limit errors
- Request batching where supported
- Intelligent retry logic

#### 4.2 Resource Limits

**Scalability Requirements:**
- Support for projects with 10,000+ resources
- Memory-efficient resource collection
- Streaming response processing for large datasets
- Configurable resource type filtering

### 5. Integration Requirements

#### 5.1 Engine Integration

**Discovery Engine Integration:**
```rust
// In engine/mod.rs
match provider_config.provider_type.as_str() {
    "builtin-gcp" => {
        info!("Creating built-in GCP provider");
        match create_builtin_gcp_provider(provider_config).await {
            Ok(provider) => {
                self.providers.push(Arc::from(provider));
            }
            Err(e) => {
                error!("Failed to create built-in GCP provider: {}", e);
            }
        }
    }
    // ... other provider types
}
```

#### 5.2 Policy Integration

**GCP-Specific Policies:**
```yaml
# Example GCP policy
name: "public-cloud-storage-buckets"
asset_type: "gcp-storage-bucket"
rules:
  - key: "public_access_prevented"
    value: "true"
  - key: "uniform_bucket_level_access"
    value: "enabled"
  - key: "encryption_enabled"
    value: "true"

name: "compute-instance-os-login"
asset_type: "gcp-compute-instance"
rules:
  - key: "os_login_enabled"
    value: "true"
  - key: "shielded_vm_enabled"
    value: "true"
```

#### 5.3 Testing Integration

**Unit Testing:**
- Mock GCP client implementations
- Resource parsing tests
- Configuration validation tests
- Error handling tests

**Integration Testing:**
- Local GCP emulator support (Firestore emulator, etc.)
- Test project with sample resources
- End-to-end discovery tests
- Performance benchmarking

### 6. Dependencies and Libraries

#### 6.1 Required Dependencies

**Core GCP SDK:**
```toml
google-cloud-auth = "0.12"
google-cloud-compute = "0.12"
google-cloud-storage = "0.12"
google-cloud-iam = "0.12"
google-cloud-kms = "0.12"
google-cloud-sqladmin = "0.12"
google-cloud-bigquery = "0.12"
google-cloud-resourcemanager = "0.12"
google-cloud-logging = "0.12"
google-cloud-monitoring = "0.12"
```

**Authentication:**
```toml
tame-oauth = "0.9"
yup-oauth2 = "0.10"
```

#### 6.2 Optional Dependencies

**Organization Support:**
```toml
google-cloud-cloudresourcemanager = "0.12"
```

**Advanced Features:**
```toml
google-cloud-pubsub = "0.12"  # For async processing
google-cloud-scheduler = "0.12"  # For scheduled scans
```

### 7. Implementation Phases

#### Phase 1: Core Infrastructure (Weeks 1-2)
- Basic provider structure and configuration
- Authentication client setup
- Core service client factory
- Basic resource discovery for Compute Engine

#### Phase 2: Essential Services (Weeks 3-4)
- Cloud Storage integration
- IAM service integration
- Basic policy support
- Unit test coverage

#### Phase 3: Advanced Services (Weeks 5-6)
- Cloud SQL and database services
- Kubernetes Engine support
- Networking resources
- Performance optimization

#### Phase 4: Organization & Security (Weeks 7-8)
- Organization-level discovery
- Advanced security features
- Comprehensive policy support
- Integration testing

#### Phase 5: Production Readiness (Weeks 9-10)
- Error handling and logging
- Performance tuning
- Documentation
- Security review

### 8. Success Criteria

#### 8.1 Functional Success
- [ ] Discovery of all specified GCP resource types
- [ ] Support for multiple authentication methods
- [ ] Multi-project and multi-region support
- [ ] Integration with existing policy engine

#### 8.2 Performance Success
- [ ] Discovery of 1,000+ resources within 60 seconds
- [ ] Memory usage under 512MB for large projects
- [ ] Successful handling of API rate limits
- [ ] Graceful degradation on API failures

#### 8.3 Security Success
- [ ] Zero hardcoded credentials
- [ ] Secure credential handling
- [ ] Input validation and sanitization
- [ ] Security audit pass

#### 8.4 Quality Success
- [ ] 90%+ test coverage
- [ ] Comprehensive documentation
- [ ] Integration with existing CI/CD
- [ ] Performance benchmarks established

### 9. Risks and Mitigations

#### 9.1 Technical Risks

**API Complexity:**
- Risk: GCP APIs are more complex than AWS
- Mitigation: Start with core services, use official SDKs

**Authentication Complexity:**
- Risk: Multiple auth methods increase complexity
- Mitigation: Prioritize service account auth, add others incrementally

**Rate Limiting:**
- Risk: GCP has stricter rate limits than AWS
- Mitigation: Implement intelligent batching and retry logic

#### 9.2 Integration Risks

**Architecture Mismatch:**
- Risk: GCP resource model doesn't fit existing architecture
- Mitigation: Early prototyping and architecture review

**Performance Impact:**
- Risk: GCP provider impacts overall performance
- Mitigation: Performance testing and optimization in each phase

### 10. Documentation Requirements

#### 10.1 Technical Documentation
- Provider architecture and design
- API integration details
- Configuration reference
- Troubleshooting guide

#### 10.2 User Documentation
- GCP provider setup guide
- Authentication configuration
- Supported resource types
- Example policies for GCP resources

#### 10.3 Developer Documentation
- Adding new GCP services
- Testing guidelines
- Performance tuning
- Security considerations

This technical requirements document provides a comprehensive foundation for implementing GCP provider support in CloudScanner, ensuring alignment with existing architecture while meeting security, performance, and usability requirements.