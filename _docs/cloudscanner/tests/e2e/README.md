# CloudScanner E2E Testing

This directory contains comprehensive end-to-end testing infrastructure for CloudScanner with LocalStack (AWS mock).

## 📁 Directory Structure

```
tests/e2e/
├── scripts/
│   ├── setup_localstack.sh    # LocalStack setup script
│   └── e2e_aws_test.sh       # AWS E2E test runner
├── configs/                   # Test configuration files (auto-generated)
├── policies/                  # Test policy files (auto-generated)
├── results/                   # Test results and reports
├── docker-compose.yml          # LocalStack Docker configuration
└── README.md                  # This file
```

## 🚀 Quick Start

### 1. Setup LocalStack Test Environment

```bash
# Run the setup script to start LocalStack and create test resources
./tests/e2e/scripts/setup_localstack.sh
```

This script will:
- ✅ Check prerequisites (Docker, AWS CLI, jq)
- 🐳 Start LocalStack with AWS services (S3, EC2, IAM, etc.)
- 📦 Create test S3 buckets (public, private, encrypted)
- 🖥️  Create test EC2 instances (web server, database, bastion)
- ⚙️  Generate test configuration files
- 📋 Create test security policies
- 🔍 Verify setup and create cleanup script

### 2. Run AWS E2E Tests

```bash
# Run comprehensive AWS provider tests
./tests/e2e/scripts/e2e_aws_test.sh
```

This script will test:
- 🔍 **S3 Discovery**: Bucket discovery and metadata
- 🖥️  **EC2 Discovery**: Instance discovery and attributes
- 🔗 **Multi-Service Discovery**: Combined S3 + EC2 discovery
- 🛡️  **Policy Evaluation**: Security policy testing
- 📊 **Output Formats**: JSON, YAML, Console output
- ⚠️  **Error Handling**: Invalid configs and edge cases
- ⚡ **Performance**: Discovery speed and efficiency

### 3. Clean Up

```bash
# Stop LocalStack and clean up resources
./tests/e2e/cleanup_localstack.sh
```

## 📋 Test Configuration Files

The setup script generates several test configurations:

### `aws_s3_test.toml`
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

### `aws_ec2_test.toml`
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

### `aws_multi_test.toml`
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

### `aws_localstack_test.toml`
```toml
[[provider]]
id = "aws-localstack-test"
type = "aws"
[provider.config]
region = "us-east-1"
endpoint_url = "http://localhost:4566"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]
```

## 🛡️ Test Security Policies

The setup creates security policies for testing:

### `public_s3_buckets.yaml`
Detects S3 buckets with public read access
```yaml
name: "Public S3 Buckets"
description: "Detect S3 buckets with public access"
resource_type: "s3-bucket"
conditions:
  - field: "public_read"
    operator: "equals"
    value: true
severity: "high"
```

### `unencrypted_s3_buckets.yaml`
Detects S3 buckets without encryption
```yaml
name: "Unencrypted S3 Buckets"
description: "Detect S3 buckets without encryption"
resource_type: "s3-bucket"
conditions:
  - field: "encryption_enabled"
    operator: "equals"
    value: false
severity: "medium"
```

### `ec2_public_ip.yaml`
Detects EC2 instances with public IP addresses
```yaml
name: "EC2 Instances with Public IP"
description: "Detect EC2 instances that have public IP addresses"
resource_type: "ec2-instance"
conditions:
  - field: "public_ip"
    operator: "exists"
severity: "low"
```

## 🧪 Test Scenarios

### AWS S3 Discovery Tests
- **Single Service**: Discover only S3 buckets
- **Resource Count**: Verify correct number of buckets found
- **Metadata Accuracy**: Check bucket properties (encryption, public access)
- **LocalStack Integration**: Test with custom endpoint

### AWS EC2 Discovery Tests
- **Single Service**: Discover only EC2 instances
- **Resource Count**: Verify correct number of instances found
- **Instance Types**: Different instance types (t2.nano, t2.micro, t2.small)
- **Security Groups**: Instance security group associations
- **Tags and Metadata**: Instance tags and properties

### Multi-Service Discovery Tests
- **Combined Discovery**: S3 + EC2 in single run
- **Resource Classification**: Proper type identification
- **No Duplication**: Ensure no duplicate resources
- **Performance**: Multi-service discovery efficiency

### Policy Evaluation Tests
- **Security Violations**: Detect public S3 buckets
- **Compliance Checking**: Verify encryption settings
- **Network Exposure**: Identify public IP addresses
- **Severity Levels**: Test different policy severities

### Output Format Tests
- **Console Output**: Human-readable report format
- **JSON Output**: Machine-readable structured data
- **YAML Output**: Alternative structured format
- **Validation**: Verify output format validity

### Error Handling Tests
- **Invalid Configuration**: Malformed TOML files
- **Missing Files**: Nonexistent configuration files
- **Network Issues**: Connection failures
- **Permission Errors**: Insufficient AWS permissions

### Performance Tests
- **Discovery Speed**: Time to complete discovery
- **Resource Scaling**: Performance with multiple resources
- **Memory Usage**: Resource consumption during discovery
- **Concurrent Operations**: Multi-provider performance

## 📊 Test Results

### Test Report Format
```
========================================
CloudScanner AWS E2E Test Report
========================================

Test Date: 2025-12-05 14:30:00
Test Environment: LocalStack (AWS Mock)
CloudScanner Binary: /path/to/cloudscanner

Test Summary:
- Total Tests: 15
- Passed: 14
- Failed: 1
- Success Rate: 93%

Detailed Results:
✅ PASS: aws_s3_only
✅ PASS: aws_ec2_only
✅ PASS: aws_multi_service
❌ FAIL: s3_public_policy
✅ PASS: output_format_console
...
```

### Generated Files
- **Test Results**: `tests/e2e/results/test_report_YYYYMMDD_HHMMSS.txt`
- **JSON Outputs**: `tests/e2e/results/{test_name}.json`
- **Log Files**: `tests/e2e/results/{test_name}.log`
- **Configuration**: `tests/e2e/configs/*.toml`
- **Policies**: `tests/e2e/policies/*.yaml`

## 🔧 Environment Variables

The scripts use these environment variables:

```bash
# AWS Configuration (for LocalStack)
export AWS_ACCESS_KEY_ID="test"
export AWS_SECRET_ACCESS_KEY="test"
export AWS_DEFAULT_REGION="us-east-1"
export AWS_ENDPOINT_URL="http://localhost:4566"

# LocalStack Configuration
LOCALSTACK_HOST="localhost"
LOCALSTACK_PORT="4566"
```

## 🐳 LocalStack Services

The setup configures these LocalStack services:
- **S3**: Simple Storage Service
- **EC2**: Elastic Compute Cloud
- **IAM**: Identity and Access Management
- **STS**: Security Token Service
- **DynamoDB**: NoSQL Database
- **Lambda**: Serverless Functions

## 📝 Manual Testing

### Test Individual Configurations
```bash
# Test S3 discovery only
./target/release/cloudscanner --config tests/e2e/configs/aws_s3_test.toml

# Test with policies
./target/release/cloudscanner \
  --config tests/e2e/configs/aws_multi_test.toml \
  --policies tests/e2e/policies/ \
  --output-format json

# Test YAML output
./target/release/cloudscanner \
  --config tests/e2e/configs/aws_localstack_test.toml \
  --output-format yaml
```

### Verify LocalStack Resources
```bash
# List S3 buckets
aws --endpoint-url=http://localhost:4566 s3 ls

# Describe EC2 instances
aws --endpoint-url=http://localhost:4566 ec2 describe-instances

# Check LocalStack health
curl http://localhost:4566/health
```

## 🐛 Troubleshooting

### Common Issues

1. **LocalStack Won't Start**
   ```bash
   # Check Docker status
   docker ps
   docker logs localstack-main
   
   # Reset Docker environment
   docker system prune -f
   ```

2. **AWS CLI Connection Issues**
   ```bash
   # Test LocalStack connection
   aws --endpoint-url=http://localhost:4566 sts get-caller-identity
   
   # Check environment variables
   env | grep AWS
   ```

3. **CloudScanner Binary Not Found**
   ```bash
   # Build CloudScanner
   cargo build --release
   
   # Check binary location
   ls -la target/release/cloudscanner
   ```

4. **Permission Errors**
   ```bash
   # Make scripts executable
   chmod +x tests/e2e/scripts/*.sh
   
   # Check file permissions
   ls -la tests/e2e/scripts/
   ```

### Debug Mode

Enable debug logging for troubleshooting:
```bash
# Run with debug logging
RUST_LOG=debug ./target/release/cloudscanner \
  --config tests/e2e/configs/aws_s3_test.toml \
  --log-level debug
```

## 🔄 Continuous Integration

### GitHub Actions Integration
```yaml
name: E2E Tests
on: [push, pull_request]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build CloudScanner
        run: cargo build --release
      - name: Setup LocalStack
        run: ./tests/e2e/scripts/setup_localstack.sh
      - name: Run E2E Tests
        run: ./tests/e2e/scripts/e2e_aws_test.sh
```

## 📚 Additional Resources

- [LocalStack Documentation](https://docs.localstack.cloud/)
- [AWS CLI Documentation](https://docs.aws.amazon.com/cli/)
- [CloudScanner README](../../../README.md)
- [Test Plan Details](../testplan_e2e.md)

## 🤝 Contributing

When adding new tests:

1. **Update Test Plan**: Modify `testplan_e2e.md` first
2. **Add Test Cases**: Update `e2e_aws_test.sh` with new scenarios
3. **Update Configurations**: Add new TOML configurations as needed
4. **Document Changes**: Update this README with new test descriptions
5. **Test Locally**: Verify tests work before submitting PR

## 📞 Support

For issues with E2E testing:
1. Check the test logs in `tests/e2e/results/`
2. Verify LocalStack is running properly
3. Ensure all prerequisites are installed
4. Review the troubleshooting section above
5. Check GitHub Issues for known problems