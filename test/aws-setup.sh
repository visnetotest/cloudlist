#!/bin/bash

# Configure AWS CLI to use LocalStack
aws configure set aws_access_key_id test
aws configure set aws_secret_access_key test
aws configure set region us-east-1

ENDPOINT_URL=http://localstack:4566

# Create test resources for all new AWS services

echo "Creating EC2 instances..."
aws --endpoint-url=$ENDPOINT_URL ec2 run-instances --image-id ami-12345678 --instance-type t2.micro --count 1 > /dev/null
aws --endpoint-url=$ENDPOINT_URL ec2 run-instances --image-id ami-87654321 --instance-type t2.small --count 1 > /dev/null

echo "Creating VPC resources..."
# Create VPC
VPC_ID=$(aws --endpoint-url=$ENDPOINT_URL ec2 create-vpc --cidr-block 10.0.0.0/16 --query 'Vpc.VpcId' --output text)
echo "Created VPC: $VPC_ID"

# Create subnets
SUBNET1_ID=$(aws --endpoint-url=$ENDPOINT_URL ec2 create-subnet --vpc-id $VPC_ID --cidr-block 10.0.1.0/24 --availability-zone us-east-1a --query 'Subnet.SubnetId' --output text)
SUBNET2_ID=$(aws --endpoint-url=$ENDPOINT_URL ec2 create-subnet --vpc-id $VPC_ID --cidr-block 10.0.2.0/24 --availability-zone us-east-1b --query 'Subnet.SubnetId' --output text)
echo "Created subnets: $SUBNET1_ID, $SUBNET2_ID"

# Create Internet Gateway
IGW_ID=$(aws --endpoint-url=$ENDPOINT_URL ec2 create-internet-gateway --query 'InternetGateway.InternetGatewayId' --output text)
echo "Created Internet Gateway: $IGW_ID"

# Attach Internet Gateway to VPC
aws --endpoint-url=$ENDPOINT_URL ec2 attach-internet-gateway --internet-gateway-id $IGW_ID --vpc-id $VPC_ID

echo "Creating Lambda functions..."
# Create Lambda function (simplified)
aws --endpoint-url=$ENDPOINT_URL lambda create-function \
  --function-name test-function \
  --runtime nodejs18.x \
  --role arn:aws:iam::123456789012:role/test-role \
  --handler index.handler \
  --code '{"ZipFile":"bG9hZGVyLnJlcXVpcmUoJ2ZzJyk7ZXhwb3J0cy5oYW5kbGVyID0gYXN5bmMgKGV2ZW50KSA9PiB7IGNvbnNvbGUubG9nKCdIZWxsbyBmcm9tIExhbWJkYSEnKTsgcmV0dXJuIHt9OyB9"}' \
  --description "Test Lambda function" > /dev/null

echo "Creating RDS instances..."
# Create RDS instance (simplified)
aws --endpoint-url=$ENDPOINT_URL rds create-db-instance \
  --db-instance-identifier test-db \
  --db-instance-class db.t3.micro \
  --engine mysql \
  --master-username admin \
  --master-user-password password123 \
  --allocated-storage 20 > /dev/null

echo "Creating ECS cluster..."
# Create ECS cluster
aws --endpoint-url=$ENDPOINT_URL ecs create-cluster --cluster-name test-cluster > /dev/null

echo "Creating IAM roles and policies..."
# Create IAM role
aws --endpoint-url=$ENDPOINT_URL iam create-role \
  --role-name test-role \
  --assume-role-policy-document '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Principal":{"Service":"lambda.amazonaws.com"},"Action":"sts:AssumeRole"}]}' > /dev/null

# Create IAM policy
aws --endpoint-url=$ENDPOINT_URL iam create-policy \
  --policy-name test-policy \
  --policy-document '{"Version":"2012-10-17","Statement":[{"Effect":"Allow","Action":["logs:*"],"Resource":"*"}]}' > /dev/null

echo "Creating CloudTrail..."
# Create CloudTrail
aws --endpoint-url=$ENDPOINT_URL cloudtrail create-trail \
  --name test-trail \
  --s3-bucket-name my-public-bucket-for-demo \
  --include-global-service-events > /dev/null

echo "Creating CloudWatch alarms..."
# Create CloudWatch alarm
aws --endpoint-url=$ENDPOINT_URL cloudwatch put-metric-alarm \
  --alarm-name test-alarm \
  --metric-name CPUUtilization \
  --namespace AWS/EC2 \
  --statistic Average \
  --period 300 \
  --threshold 80 \
  --comparison-operator GreaterThanThreshold \
  --evaluation-periods 2 > /dev/null

echo "Creating Route53 hosted zone..."
# Create Route53 hosted zone
aws --endpoint-url=$ENDPOINT_URL route53 create-hosted-zone \
  --name example.com \
  --caller-reference test-$(date +%s) > /dev/null

echo "Creating S3 buckets..."
# Create S3 buckets
aws --endpoint-url=$ENDPOINT_URL s3api create-bucket --bucket test-bucket-1 --region us-east-1
aws --endpoint-url=$ENDPOINT_URL s3api create-bucket --bucket test-bucket-2 --region us-east-1

echo "AWS resources created for integration testing."