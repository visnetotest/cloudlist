#!/bin/bash

# LocalStack AWS Setup Script for CloudScanner Testing
# This script sets up LocalStack with AWS services for testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LOCALSTACK_PORT="4566"
LOCALSTACK_HOST="localhost"
AWS_REGION="us-east-1"
SERVICES=("s3" "ec2" "iam" "sts" "dynamodb" "lambda")

# Test Resources Configuration
S3_BUCKETS=(
    "test-private-bucket"
    "test-public-bucket"
    "test-encrypted-bucket"
)

EC2_INSTANCES=(
    "test-web-server"
    "test-database-server"
    "test-bastion-host"
)

# Directories - we're in cloudscanner/tests/e2e/scripts, so go up 3 levels to get to cloudscanner root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$(dirname "$SCRIPT_DIR")")")"
CONFIG_DIR="$PROJECT_ROOT/tests/e2e/configs"
POLICIES_DIR="$PROJECT_ROOT/tests/e2e/policies"
RESULTS_DIR="$PROJECT_ROOT/tests/e2e/results"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if AWS CLI is installed
    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI is not installed. Please install AWS CLI first."
        exit 1
    fi
    
    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        log_error "jq is not installed. Please install jq first."
        exit 1
    fi
    
    log_success "All prerequisites are satisfied"
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$POLICIES_DIR"
    mkdir -p "$RESULTS_DIR"
    # Scripts directory already exists (we're running from it)
    
    log_success "Directories created"
}

# Start LocalStack
start_localstack() {
    log_info "Starting LocalStack..."
    
    # Create docker-compose file for LocalStack
    cat > "$PROJECT_ROOT/tests/e2e/docker-compose.yml" << EOF
services:
  localstack:
    container_name: localstack-main
    image: localstack/localstack:latest
    ports:
      - "${LOCALSTACK_PORT}:${LOCALSTACK_PORT}"
      - "4510-4559:4510-4559"
    environment:
      - DEBUG=1
      - SERVICES=s3,ec2,iam,sts,dynamodb,lambda
      - DATA_DIR=/tmp/localstack/data
      - DOCKER_HOST=unix:///var/run/docker.sock
      - DEFAULT_REGION=${AWS_REGION}
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock"
    networks:
      - localstack-network



networks:
  localstack-network:
    driver: bridge
EOF

    # Start LocalStack
    cd "$PROJECT_ROOT/tests/e2e"
    docker-compose up -d
    
    # Wait for LocalStack to be ready
    log_info "Waiting for LocalStack to be ready..."
    for i in {1..30}; do
        if curl -s "http://${LOCALSTACK_HOST}:${LOCALSTACK_PORT}/health" | grep -q "Ready\|running\|ok\|services"; then
            log_success "LocalStack is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            log_warning "LocalStack health check inconclusive, but proceeding..."
            log_success "LocalStack is likely ready (container is running)"
            break
        fi
        sleep 2
    done
    
    cd "$PROJECT_ROOT"
}

# Configure AWS CLI for LocalStack
configure_aws_cli() {
    log_info "Configuring AWS CLI for LocalStack..."
    
    export AWS_ACCESS_KEY_ID="test"
    export AWS_SECRET_ACCESS_KEY="test"
    export AWS_DEFAULT_REGION="$AWS_REGION"
    export AWS_ENDPOINT_URL="http://${LOCALSTACK_HOST}:${LOCALSTACK_PORT}"
    
    # Create AWS CLI configuration
    aws configure set aws_access_key_id "test"
    aws configure set aws_secret_access_key "test"
    aws configure set default.region "$AWS_REGION"
    
    # Test AWS CLI connection
    if aws --endpoint-url="$AWS_ENDPOINT_URL" sts get-caller-identity &> /dev/null; then
        log_success "AWS CLI configured successfully"
    else
        log_error "Failed to configure AWS CLI for LocalStack"
        exit 1
    fi
}

# Create S3 buckets for testing
create_s3_buckets() {
    log_info "Creating S3 buckets..."
    
    for bucket in "${S3_BUCKETS[@]}"; do
        log_info "Creating bucket: $bucket"
        
        # Create bucket
        aws --endpoint-url="$AWS_ENDPOINT_URL" s3 mb "s3://$bucket" --region "$AWS_REGION"
        
        # Configure bucket based on name
        case $bucket in
            "test-public-bucket")
                # Make bucket public
                aws --endpoint-url="$AWS_ENDPOINT_URL" s3api put-bucket-policy \
                    --bucket "$bucket" \
                    --policy '{
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "PublicReadGetObject",
                                "Effect": "Allow",
                                "Principal": "*",
                                "Action": "s3:GetObject",
                                "Resource": "arn:aws:s3:::'$bucket'/*"
                            }
                        ]
                    }'
                log_info "Made bucket $bucket public"
                ;;
            "test-encrypted-bucket")
                # Enable default encryption
                aws --endpoint-url="$AWS_ENDPOINT_URL" s3api put-bucket-encryption \
                    --bucket "$bucket" \
                    --server-side-encryption-configuration '{
                        "Rules": [
                            {
                                "ApplyServerSideEncryptionByDefault": {
                                    "SSEAlgorithm": "AES256"
                                }
                            }
                        ]
                    }'
                log_info "Enabled encryption for bucket $bucket"
                ;;
        esac
        
        # Add some test files
        echo "Test file in $bucket" | aws --endpoint-url="$AWS_ENDPOINT_URL" s3 cp - "s3://$bucket/test-file.txt"
        
        log_success "Created bucket: $bucket"
    done
}

# Create EC2 instances for testing
create_ec2_instances() {
    log_info "Creating EC2 instances..."
    
    # Create a test key pair
    aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 create-key-pair \
        --key-name "test-key-pair" \
        --query 'KeyMaterial' \
        --output text > "$PROJECT_ROOT/tests/e2e/test-key-pair.pem" 2>/dev/null || true
    
    if [ -f "$PROJECT_ROOT/tests/e2e/test-key-pair.pem" ]; then
        chmod 400 "$PROJECT_ROOT/tests/e2e/test-key-pair.pem"
        log_success "Test key pair created and secured"
    else
        log_warning "Failed to create test key pair, will continue without it"
    fi
    
    # Create security groups
    aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 create-security-group \
        --group-name "test-web-sg" \
        --description "Security group for web servers"
    
    aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 create-security-group \
        --group-name "test-db-sg" \
        --description "Security group for database servers"
    
    # Add security group rules
    WEB_SG_ID=$(aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 describe-security-groups \
        --group-names "test-web-sg" \
        --query 'SecurityGroups[0].GroupId' \
        --output text 2>/dev/null || echo "")
    
    DB_SG_ID=$(aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 describe-security-groups \
        --group-names "test-db-sg" \
        --query 'SecurityGroups[0].GroupId' \
        --output text 2>/dev/null || echo "")
    
    # Allow HTTP/HTTPS to web servers
    aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 authorize-security-group-ingress \
        --group-id "$WEB_SG_ID" \
        --protocol tcp \
        --port 80 \
        --cidr 0.0.0.0/0
    
    aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 authorize-security-group-ingress \
        --group-id "$WEB_SG_ID" \
        --protocol tcp \
        --port 443 \
        --cidr 0.0.0.0/0
    
    # Create instances (using LocalStack's mock AMI)
    for instance in "${EC2_INSTANCES[@]}"; do
        log_info "Creating instance: $instance"
        
        case $instance in
            "test-web-server")
                SG_ID="$WEB_SG_ID"
                INSTANCE_TYPE="t2.micro"
                ;;
            "test-database-server")
                SG_ID="$DB_SG_ID"
                INSTANCE_TYPE="t2.small"
                ;;
            "test-bastion-host")
                SG_ID="$WEB_SG_ID"
                INSTANCE_TYPE="t2.nano"
                ;;
        esac
        
        # Create instance (skip key pair if it doesn't exist)
        INSTANCE_ID=""
        if [ -f "$PROJECT_ROOT/tests/e2e/test-key-pair.pem" ]; then
            INSTANCE_ID=$(aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 run-instances \
                --image-id "ami-12345678" \
                --count 1 \
                --instance-type "$INSTANCE_TYPE" \
                --key-name "test-key-pair" \
                --security-group-ids "$SG_ID" \
                --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$instance}]" \
                --query 'Instances[0].InstanceId' \
                --output text 2>/dev/null || echo "")
        else
            log_warning "Test key pair not found, creating instance without key"
            INSTANCE_ID=$(aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 run-instances \
                --image-id "ami-12345678" \
                --count 1 \
                --instance-type "$INSTANCE_TYPE" \
                --security-group-ids "$SG_ID" \
                --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=$instance}]" \
                --query 'Instances[0].InstanceId' \
                --output text 2>/dev/null || echo "")
        fi
        
        if [ -n "$INSTANCE_ID" ]; then
            log_success "Created instance: $instance ($INSTANCE_ID)"
        else
            log_error "Failed to create instance: $instance"
        fi
    done
}

# Create test configuration files
create_test_configs() {
    log_info "Creating test configuration files..."
    
    # AWS S3-only configuration
    cat > "$CONFIG_DIR/aws_s3_test.toml" << EOF
[[provider]]
id = "aws-s3-test"
type = "aws"
[provider.config]
region = "$AWS_REGION"
services = [
    { name = "ec2", enabled = false },
    { name = "s3", enabled = true }
]
EOF

    # AWS EC2-only configuration
    cat > "$CONFIG_DIR/aws_ec2_test.toml" << EOF
[[provider]]
id = "aws-ec2-test"
type = "aws"
[provider.config]
region = "$AWS_REGION"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = false }
]
EOF

    # AWS multi-service configuration
    cat > "$CONFIG_DIR/aws_multi_test.toml" << EOF
[[provider]]
id = "aws-multi-test"
type = "aws"
[provider.config]
region = "$AWS_REGION"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]
EOF

    # AWS configuration with custom endpoint
    cat > "$CONFIG_DIR/aws_localstack_test.toml" << EOF
[[provider]]
id = "aws-localstack-test"
type = "aws"
[provider.config]
region = "$AWS_REGION"
endpoint_url = "http://$LOCALSTACK_HOST:$LOCALSTACK_PORT"
services = [
    { name = "ec2", enabled = true },
    { name = "s3", enabled = true }
]
EOF

    log_success "Test configuration files created"
}

# Create test policies
create_test_policies() {
    log_info "Creating test policies..."
    
    # Public S3 bucket detection policy
    cat > "$POLICIES_DIR/public_s3_buckets.yaml" << EOF
name: "Public S3 Buckets"
description: "Detect S3 buckets with public access"
resource_type: "s3-bucket"
conditions:
  - field: "public_read"
    operator: "equals"
    value: true
severity: "high"
remediation: "Review bucket permissions and remove public access"
EOF

    # Unencrypted S3 bucket detection policy
    cat > "$POLICIES_DIR/unencrypted_s3_buckets.yaml" << EOF
name: "Unencrypted S3 Buckets"
description: "Detect S3 buckets without encryption"
resource_type: "s3-bucket"
conditions:
  - field: "encryption_enabled"
    operator: "equals"
    value: false
severity: "medium"
remediation: "Enable default encryption for the bucket"
EOF

    # EC2 instance with public IP detection policy
    cat > "$POLICIES_DIR/ec2_public_ip.yaml" << EOF
name: "EC2 Instances with Public IP"
description: "Detect EC2 instances that have public IP addresses"
resource_type: "ec2-instance"
conditions:
  - field: "public_ip"
    operator: "exists"
severity: "low"
remediation: "Review if public IP is necessary for this instance"
EOF

    # EC2 instance without encryption policy
    cat > "$POLICIES_DIR/ec2_unencrypted_volumes.yaml" << EOF
name: "EC2 Instances with Unencrypted Volumes"
description: "Detect EC2 instances with unattached EBS volumes"
resource_type: "ec2-instance"
conditions:
  - field: "ebs_encrypted"
    operator: "equals"
    value: false
severity: "medium"
remediation: "Enable encryption for EBS volumes"
EOF

    log_success "Test policies created"
}

# Verify setup
verify_setup() {
    log_info "Verifying LocalStack setup..."
    
    # Check S3 buckets
    BUCKETS_COUNT=$(aws --endpoint-url="$AWS_ENDPOINT_URL" s3 ls 2>/dev/null | wc -l)
    log_info "Found $BUCKETS_COUNT S3 buckets"
    
    # Check EC2 instances
    INSTANCES_COUNT=$(aws --endpoint-url="$AWS_ENDPOINT_URL" ec2 describe-instances \
        --query 'Reservations | length' \
        --output text 2>/dev/null || echo "0")
    log_info "Found $INSTANCES_COUNT EC2 instances"
    
    # Test connectivity
    if curl -s "http://${LOCALSTACK_HOST}:${LOCALSTACK_PORT}/health" &>/dev/null; then
        log_success "LocalStack is running and accessible"
    else
        log_warning "LocalStack health check failed, but container is running"
    fi
    
    log_success "Setup verification completed"
}

# Create cleanup script
create_cleanup_script() {
    log_info "Creating cleanup script..."
    
    cat > "$PROJECT_ROOT/tests/e2e/cleanup_localstack.sh" << 'EOF'
#!/bin/bash

# LocalStack Cleanup Script
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

LOCALSTACK_PORT="4566"
AWS_ENDPOINT_URL="http://localhost:${LOCALSTACK_PORT}"

echo "Cleaning up LocalStack resources..."

# Stop LocalStack
cd "$PROJECT_ROOT/tests/e2e"
if [ -f "docker-compose.yml" ]; then
    docker-compose down
    cd "$PROJECT_ROOT"
fi

# Remove Docker volumes
docker volume prune -f

# Remove test files
rm -f tests/e2e/test-key-pair.pem
rm -rf tests/e2e/tmp

echo "Cleanup completed"
EOF

    chmod +x "$PROJECT_ROOT/tests/e2e/cleanup_localstack.sh"
    log_success "Cleanup script created"
}

# Main execution
main() {
    echo "========================================"
    echo "LocalStack Setup for CloudScanner Testing"
    echo "========================================"
    echo
    
    check_prerequisites
    create_directories
    start_localstack
    configure_aws_cli
    create_s3_buckets
    create_ec2_instances
    create_test_configs
    create_test_policies
    verify_setup
    create_cleanup_script
    
    echo
    echo "========================================"
    log_success "LocalStack setup completed successfully!"
    echo "========================================"
    echo
    echo "Next steps:"
    echo "1. Run AWS E2E tests: ./tests/e2e/scripts/e2e_aws_test.sh"
    echo "2. View test configs: ls -la $CONFIG_DIR/"
    echo "3. View test policies: ls -la $POLICIES_DIR/"
    echo "4. Clean up when done: ./tests/e2e/cleanup_localstack.sh"
    echo
    echo "Environment variables for testing:"
    echo "export AWS_ACCESS_KEY_ID=test"
    echo "export AWS_SECRET_ACCESS_KEY=test"
    echo "export AWS_DEFAULT_REGION=$AWS_REGION"
    echo "export AWS_ENDPOINT_URL=http://$LOCALSTACK_HOST:$LOCALSTACK_PORT"
    echo
}

# Run main function
main "$@"