# CloudScanner End-to-End Test Plan

## Overview

This document outlines comprehensive end-to-end testing scenarios for CloudScanner with multiple cloud providers including AWS, GCP, Azure, and other providers.

## Test Environment Setup

### Prerequisites

1. **Cloud Accounts**: Active accounts with appropriate permissions
2. **Credentials**: Configured authentication for each cloud provider
3. **Test Resources**: Pre-existing cloud resources to discover
4. **Network Access**: Internet connectivity for cloud API access

### Environment Variables

```bash
# AWS Credentials
export AWS_ACCESS_KEY_ID="your_aws_access_key"
export AWS_SECRET_ACCESS_KEY="your_aws_secret_key"
export AWS_DEFAULT_REGION="us-east-1"

# GCP Credentials  
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account.json"
export GCP_PROJECT_ID="your-gcp-project-id"
export GCP_ORG_ID="your-gcp-org-id"

# Azure Credentials
export AZURE_CLIENT_ID="your_azure_client_id"
export AZURE_CLIENT_SECRET="your_azure_client_secret"
export AZURE_TENANT_ID="your_azure_tenant_id"
export AZURE_SUBSCRIPTION_ID="your_azure_subscription_id"

# DigitalOcean
export DIGITALOCEAN_TOKEN="your_do_token"

# Other providers as needed
```

## Test Scenarios

### 1. AWS Provider Tests

#### 1.1 AWS EC2 Discovery
**Objective**: Verify discovery of EC2 instances across regions

**Test Configuration**:
```toml
[[provider]]
id = "aws-ec2-test"
type = "aws"
[provider.config]
region = "us-east-1"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = false }
]
```

**Expected Results**:
- Discover all running EC2 instances
- Include instance metadata (type, state, tags)
- Handle regions without instances gracefully
- Respect service filtering (only EC2, not S3)

**Test Cases**:
- [ ] Discover instances in single region
- [ ] Discover instances across multiple regions
- [ ] Handle empty regions (no instances)
- [ ] Verify instance metadata accuracy
- [ ] Test with invalid credentials
- [ ] Test with insufficient permissions

#### 1.2 AWS S3 Discovery
**Objective**: Verify discovery of S3 buckets

**Test Configuration**:
```toml
[[provider]]
id = "aws-s3-test"
type = "aws"
[provider.config]
region = "us-east-1"
services = [
    { name = "ec2", enabled = false },
    { name = "s3", enabled = true }
]
```

**Expected Results**:
- Discover all S3 buckets in account
- Include bucket metadata (region, creation date, public access)
- Handle buckets with different access configurations
- Detect public buckets for security policies

**Test Cases**:
- [ ] Discover all buckets in account
- [ ] Verify bucket metadata accuracy
- [ ] Identify public vs private buckets
- [ ] Test with buckets in different regions
- [ ] Handle empty S3 accounts
- [ ] Test with bucket naming variations

#### 1.3 AWS Multi-Service Discovery
**Objective**: Test discovery of multiple AWS services simultaneously

**Test Configuration**:
```toml
[[provider]]
id = "aws-multi-test"
type = "aws"
[provider.config]
region = "us-east-1"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]
```

**Expected Results**:
- Discover both EC2 instances and S3 buckets
- Proper resource type classification
- No resource duplication
- Combined reporting

**Test Cases**:
- [ ] Multi-service discovery accuracy
- [ ] Resource type classification
- [ ] Performance with multiple services
- [ ] Error handling for partial failures

### 2. GCP Provider Tests

#### 2.1 GCP Project-Level Discovery
**Objective**: Verify discovery using project-level APIs

**Test Configuration**:
```toml
[[provider]]
id = "gcp-project-test"
type = "gcp"
[provider.config]
project_id = "your-gcp-project"
services = [
    { name = "compute", enabled = true },
    { name = "dns", enabled = true },
    { name = "storage", enabled = true }
]
```

**Expected Results**:
- Discover Compute Engine instances
- Discover Cloud DNS zones and records
- Discover Cloud Storage buckets
- Proper GCP resource metadata

**Test Cases**:
- [ ] Compute Engine instance discovery
- [ ] Cloud DNS zone and record discovery
- [ ] Cloud Storage bucket discovery
- [ ] Handle projects without resources
- [ ] Test with invalid project ID
- [ ] Verify GCP-specific metadata

#### 2.2 GCP Organization-Level Discovery
**Objective**: Verify discovery using Asset API at organization level

**Test Configuration**:
```toml
[[provider]]
id = "gcp-org-test"
type = "gcp"
[provider.config]
organization_id = "your-gcp-org-id"
services = [
    { name = "asset-api", enabled = true }
]
```

**Expected Results**:
- Discover all assets across organization
- Include projects, instances, buckets, etc.
- Handle large-scale discovery
- Proper asset type classification

**Test Cases**:
- [ ] Organization-wide asset discovery
- [ ] Multiple project resource discovery
- [ ] Asset API pagination handling
- [ ] Performance with large organizations
- [ ] Handle org-level permission issues
- [ ] Test asset type filtering

### 3. Azure Provider Tests

#### 3.1 Azure Resource Discovery
**Objective**: Verify discovery of Azure resources

**Test Configuration**:
```toml
[[provider]]
id = "azure-test"
type = "azure"
[provider.config]
subscription_id = "your-azure-subscription"
services = [
    { name = "virtual-machines", enabled = true },
    { name = "storage-accounts", enabled = true },
    { name = "dns-zones", enabled = true }
]
```

**Expected Results**:
- Discover Azure VMs across resource groups
- Discover storage accounts
- Discover DNS zones
- Proper Azure resource metadata

**Test Cases**:
- [ ] Virtual machine discovery
- [ ] Storage account discovery
- [ ] DNS zone discovery
- [ ] Multi-resource group discovery
- [ ] Handle empty subscriptions
- [ ] Test with different Azure regions

### 4. Multi-Cloud Provider Tests

#### 4.1 Simultaneous Multi-Cloud Discovery
**Objective**: Test discovery across multiple cloud providers

**Test Configuration**:
```toml
[[provider]]
id = "aws-multi"
type = "aws"
[provider.config]
region = "us-east-1"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]

[[provider]]
id = "gcp-multi"
type = "gcp"
[provider.config]
project_id = "your-gcp-project"
services = [
    { name = "compute", enabled = true },
    { name = "storage", enabled = true }
]

[[provider]]
id = "azure-multi"
type = "azure"
[provider.config]
subscription_id = "your-azure-subscription"
services = [
    { name = "virtual-machines", enabled = true }
]
```

**Expected Results**:
- Discover resources from all configured providers
- Proper provider identification in results
- No cross-provider resource conflicts
- Combined reporting with provider breakdown

**Test Cases**:
- [ ] Multi-cloud resource discovery
- [ ] Provider-specific resource classification
- [ ] Error handling for individual provider failures
- [ ] Performance with multiple providers
- [ ] Resource deduplication across providers
- [ ] Provider-specific error reporting

### 5. Policy Evaluation Tests

#### 5.1 Security Policy Testing
**Objective**: Test policy evaluation against discovered resources

**Test Policies**:
```yaml
# Public S3 Bucket Detection
- name: "Public S3 Buckets"
  description: "Detect S3 buckets with public access"
  resource_type: "s3-bucket"
  conditions:
    - field: "public_read"
      operator: "equals"
      value: true
  severity: "high"

# Unencrypted EBS Volumes
- name: "Unencrypted EBS Volumes"
  description: "Detect EBS volumes without encryption"
  resource_type: "ec2-instance"
  conditions:
    - field: "ebs_encrypted"
      operator: "equals"
      value: false
  severity: "medium"

# Public IP Exposure
- name: "Public IP Exposure"
  description: "Detect resources with public IP addresses"
  resource_type: "ec2-instance"
  conditions:
    - field: "public_ip"
      operator: "exists"
  severity: "low"
```

**Test Cases**:
- [ ] Policy violation detection
- [ ] Policy compliance reporting
- [ ] Multiple policy evaluation
- [ ] Severity-based reporting
- [ ] Custom policy testing
- [ ] Policy syntax validation

### 6. Output Format Tests

#### 6.1 Output Format Validation
**Objective**: Test different output formats

**Test Cases**:
- [ ] Console output format
- [ ] JSON output format
- [ ] YAML output format
- [ ] Output format validation
- [ ] Large dataset handling
- [ ] Output filtering capabilities

### 7. Performance and Scale Tests

#### 7.1 Large-Scale Discovery
**Objective**: Test performance with large numbers of resources

**Test Scenarios**:
- [ ] Discovery with 100+ resources
- [ ] Discovery with 1000+ resources
- [ ] Multi-region performance
- [ ] Memory usage monitoring
- [ ] Discovery timeout handling
- [ ] Concurrent provider execution

### 8. Error Handling and Edge Cases

#### 8.1 Error Scenarios
**Objective**: Test error handling and edge cases

**Test Cases**:
- [ ] Invalid cloud credentials
- [ ] Network connectivity issues
- [ ] API rate limiting
- [ ] Invalid configuration files
- [ ] Missing provider plugins
- [ ] Corrupted policy files
- [ ] Insufficient permissions
- [ ] Service unavailability

### 9. Security Tests

#### 9.1 Security Validation
**Objective**: Test security aspects of CloudScanner

**Test Cases**:
- [ ] Credential protection
- [ ] Plugin security validation
- [ ] Configuration file security
- [ ] Network traffic encryption
- [ ] Audit logging
- [ ] Access control validation

## Test Execution Framework

### Automation Script

```bash
#!/bin/bash
# e2e_test_runner.sh

set -e

CLOUDSCANNER_BIN="./target/release/cloudscanner"
TEST_CONFIG_DIR="./tests/e2e/configs"
RESULTS_DIR="./tests/e2e/results"
POLICIES_DIR="./tests/e2e/policies"

# Create directories
mkdir -p "$RESULTS_DIR"
mkdir -p "$TEST_CONFIG_DIR"

# Test functions
run_test() {
    local test_name="$1"
    local config_file="$2"
    local expected_resources="$3"
    
    echo "Running test: $test_name"
    
    $CLOUDSCANNER_BIN \
        --config "$config_file" \
        --policies "$POLICIES_DIR" \
        --output-format json \
        > "$RESULTS_DIR/${test_name}.json" 2>&1
    
    # Validate results
    validate_results "$test_name" "$expected_resources"
}

validate_results() {
    local test_name="$1"
    local expected_resources="$2"
    
    # Add validation logic here
    echo "Validating results for: $test_name"
}

# Test execution
echo "Starting CloudScanner E2E Tests..."

# Run all test scenarios
run_test "aws_ec2_single_region" "$TEST_CONFIG_DIR/aws_ec2_single.toml" "expected_count"
run_test "gcp_project_discovery" "$TEST_CONFIG_DIR/gcp_project.toml" "expected_count"
run_test "multi_cloud_discovery" "$TEST_CONFIG_DIR/multi_cloud.toml" "expected_count"

echo "E2E Tests completed. Results in: $RESULTS_DIR"
```

### Test Data Management

**Test Resources Setup**:
- Use dedicated test projects/accounts
- Create predictable test resources
- Tag resources for easy identification
- Clean up resources after tests

**Mock Services**:
- LocalStack for AWS testing
- GCP emulator for GCP testing
- Azurite for Azure testing
- Custom mock providers

## Success Criteria

### Functional Requirements
- [ ] All supported providers discover resources correctly
- [ ] Policy evaluation identifies violations accurately
- [ ] Output formats produce valid, parseable results
- [ ] Error handling is graceful and informative
- [ ] Performance meets acceptable thresholds

### Non-Functional Requirements
- [ ] Security best practices are followed
- [ ] Resource usage is within acceptable limits
- [ ] Logging provides adequate debugging information
- [ ] Configuration validation prevents invalid setups
- [ ] Plugin system works securely and reliably

## Test Reporting

### Metrics to Collect
- Resource discovery accuracy
- Policy evaluation precision
- Performance benchmarks
- Error rates and types
- Resource utilization

### Test Report Template
```
# CloudScanner E2E Test Report

## Test Execution Summary
- Date: [timestamp]
- Environment: [test environment details]
- Test Duration: [total time]

## Provider Test Results
### AWS
- EC2 Discovery: [PASS/FAIL] - [details]
- S3 Discovery: [PASS/FAIL] - [details]
- Multi-Service: [PASS/FAIL] - [details]

### GCP
- Project Discovery: [PASS/FAIL] - [details]
- Organization Discovery: [PASS/FAIL] - [details]

### Azure
- Resource Discovery: [PASS/FAIL] - [details]

## Cross-Cutting Concerns
- Policy Evaluation: [PASS/FAIL] - [details]
- Output Formats: [PASS/FAIL] - [details]
- Performance: [PASS/FAIL] - [details]
- Error Handling: [PASS/FAIL] - [details]

## Issues and Recommendations
[Detailed findings and improvement suggestions]
```

## Maintenance

### Regular Test Updates
- Update test configurations for new provider features
- Add test cases for new policy types
- Refresh test resources and credentials
- Update performance benchmarks

### Continuous Integration
- Integrate E2E tests into CI/CD pipeline
- Run tests against staging environments
- Automated test result reporting
- Performance regression detection