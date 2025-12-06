# CloudScanner AWS Provider Plugin

## Overview

The CloudScanner AWS Provider Plugin enables discovery of AWS cloud resources including EC2 instances and S3 buckets. This plugin implements the CloudScanner provider interface using Rust's FFI (Foreign Function Interface) for dynamic loading.

## Features

- **EC2 Instance Discovery**: Enumerate EC2 instances across regions
- **S3 Bucket Discovery**: List S3 buckets and their configurations
- **LocalStack Support**: Test with LocalStack for development/testing
- **Flexible Configuration**: Support for multiple AWS authentication methods
- **Plugin Architecture**: Dynamically loadable plugin for CloudScanner

## Building the Plugin

### Prerequisites

- Rust 1.70+ 
- AWS SDK dependencies (included in Cargo.toml)

### Build Commands

```bash
# Build the AWS provider plugin
cargo build -p cloudscanner-provider-aws

# Copy to plugins directory
cp target/debug/libcloudscanner_provider_aws.dylib cloudscanner/plugins/
```

## Configuration

### Plugin Configuration (TOML)

```toml
[[provider]]
id = "aws-localstack-test"
type = "plugin"
plugin_path = "plugins/libcloudscanner_provider_aws.dylib"

[provider.config]
region = "us-east-1"
endpoint_url = "http://localhost:4566"  # Optional: for LocalStack
access_key_id = "your-access-key"       # Optional: can use env vars
secret_access_key = "your-secret-key"  # Optional: can use env vars
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]
```

### Configuration Options

| Field | Type | Required | Description |
|-------|------|----------|
| `region` | String | Yes | AWS region |
| `endpoint_url` | String | No | Custom endpoint (for LocalStack) |
| `access_key_id` | String | No | AWS access key (can use env var) |
| `secret_access_key` | String | No | AWS secret key (can use env var) |
| `session_token` | String | No | AWS session token |
| `profile` | String | No | AWS profile name |
| `role_arn` | String | No | Role ARN to assume |
| `external_id` | String | No | External ID for role assumption |
| `use_ssl` | Boolean | No | Use SSL (default: true) |
| `services` | Array | Yes | List of services to discover |-------------|
| `region` | String | Yes | AWS region |
| `endpoint_url` | String | No | Custom endpoint (for LocalStack) |
| `access_key_id` | String | No | AWS access key (can use env var) |
| `secret_access_key` | String | No | AWS secret key (can use env var) |
| `session_token` | String | No | AWS session token |
| `profile` | String | No | AWS profile name |
| `role_arn` | String | No | Role ARN to assume |
| `external_id` | String | No | External ID for role assumption |
| `use_ssl` | Boolean | No | Use SSL (default: true) |
| `services` | Array | Yes | List of services to discover |

### Service-Specific Configuration

| Service | Configuration Options |
|---------|---------------------|
| `ec2` | No service-specific options |
| `s3` | No service-specific options |
| `lambda` | No service-specific options |
| `rds` | No service-specific options |
| `ecs` | No service-specific options |
| `vpc` | No service-specific options |
| `iam` | No service-specific options |
| `cloudtrail` | No service-specific options |
| `cloudwatch` | No service-specific options |

### Supported Services

| Service Name | Description | Resources Discovered |
|---------------|-------------|---------------------|
| `ec2` | Amazon EC2 | EC2 instances |
| `s3` | Amazon S3 | S3 buckets |
| `lambda` | AWS Lambda | Lambda functions |
| `rds` | Amazon RDS | RDS instances and clusters |
| `ecs` | Amazon ECS | ECS clusters, services, and tasks |
| `vpc` | Amazon VPC | VPCs, subnets, security groups, route tables, internet gateways, NAT gateways |
| `iam` | AWS IAM | IAM roles, policies, users, and groups |
| `cloudtrail` | AWS CloudTrail | CloudTrail trails and events |
| `cloudwatch` | Amazon CloudWatch | CloudWatch log groups, metrics, and alarms |
| `lambda` | AWS Lambda | Lambda functions |
| `rds` | Amazon RDS | RDS instances and clusters |
| `ecs` | Amazon ECS | ECS clusters, services, and tasks |
| `vpc` | Amazon VPC | VPCs, subnets, security groups, route tables, internet gateways, NAT gateways |
| `iam` | AWS IAM | IAM roles, policies, users, and groups |
| `cloudtrail` | AWS CloudTrail | CloudTrail trails and events |
| `cloudwatch` | Amazon CloudWatch | CloudWatch log groups, metrics, and alarms |
| `lambda` | AWS Lambda | Lambda functions |
| `rds` | Amazon RDS | RDS instances and clusters |
| `ecs` | Amazon ECS | ECS clusters, services, and tasks |
| `vpc` | Amazon VPC | VPCs, subnets, security groups, route tables, internet gateways, NAT gateways |
| `iam` | AWS IAM | IAM roles, policies, users, and groups |
| `cloudtrail` | AWS CloudTrail | CloudTrail trails and events |
| `cloudwatch` | Amazon CloudWatch | CloudWatch log groups, metrics, and alarms |

## Usage Examples

### Basic AWS Discovery

```bash
# Run CloudScanner with AWS plugin
cloudscanner -c aws_config.toml
```

### LocalStack Testing

```bash
# Start LocalStack
docker run -d -p 4566:4566 localstack/localstack

# Run with LocalStack endpoint
cloudscanner -c aws_localstack_config.toml
```

### Environment Variables

```bash
# Set AWS credentials
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_DEFAULT_REGION=us-east-1

# Run CloudScanner
cloudscanner -c aws_config.toml
```

## Plugin Architecture

### FFI Interface

The plugin exports the following C-compatible functions:

```rust
#[no_mangle]
pub extern "C" fn _create_provider() -> *mut std::ffi::c_void;

#[no_mangle]
pub extern "C" fn _destroy_provider(provider: *mut std::ffi::c_void);

#[no_mangle]
pub extern "C" fn create_provider(config_json: *const i8) -> *mut std::ffi::c_void;
```

### Provider Implementation

The AWS provider implements:

- **Resource Discovery**: EC2 instances and S3 buckets
- **Configuration Parsing**: TOML/JSON configuration support
- **Error Handling**: Comprehensive error reporting
- **Logging**: Integration with CloudScanner logging system

## Development

### Project Structure

```
cloudscanner-provider-aws/
├── src/
│   └── lib.rs              # Main plugin implementation
├── Cargo.toml              # Plugin dependencies
└── README.md               # This documentation
```

### Testing

```bash
# Run plugin tests
cargo test -p cloudscanner-provider-aws

# Run E2E tests with LocalStack
cd cloudscanner/tests/e2e/scripts
./setup_localstack.sh
./e2e_aws_test.sh
```

## Troubleshooting

### Common Issues

1. **Plugin Loading Failed**
   - Ensure plugin file exists at specified path
   - Check file permissions
   - Verify plugin architecture matches system

2. **AWS Authentication Errors**
   - Check AWS credentials in environment or config
   - Verify IAM permissions
   - Ensure region is correctly specified

3. **LocalStack Connection Issues**
   - Verify LocalStack is running: `docker ps | grep localstack`
   - Check endpoint URL configuration
   - Ensure port 4566 is accessible

### Debug Logging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cloudscanner -c aws_config.toml
```

## Dependencies

- `aws-config`: AWS configuration
- `aws-sdk-ec2`: EC2 service client
- `aws-sdk-s3`: S3 service client
- `serde`: Serialization/deserialization
- `tokio`: Async runtime
- `tracing`: Logging framework
- `libloading`: Dynamic library loading

## License

This plugin is part of the CloudScanner project and follows the same license terms.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit a pull request

For more details, see the main CloudScanner documentation.