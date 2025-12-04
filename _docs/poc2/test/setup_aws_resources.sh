#!/bin/bash

# This script creates dummy AWS resources in a Localstack container
# It is designed to be run as part of the docker-compose setup for PoC2.

echo "Waiting for LocalStack services to be ready..."

# Wait for S3
until aws --endpoint-url=http://localstack:4566 s3 ls > /dev/null 2>&1; do
  >&2 echo "S3 is unavailable - sleeping"
  sleep 1
done
echo "S3 is up."

# Wait for EC2
until aws --endpoint-url=http://localstack:4566 ec2 describe-regions > /dev/null 2>&1; do
  >&2 echo "EC2 is unavailable - sleeping"
  sleep 1
done
echo "EC2 is up."

echo "LocalStack is ready. Creating resources..."

# Create S3 Bucket
BUCKET_NAME="my-test-bucket-$(date +%s)"
echo "Creating S3 bucket: $BUCKET_NAME"
aws --endpoint-url=http://localstack:4566 s3api create-bucket --bucket $BUCKET_NAME --region us-east-1

# Create EC2 Instance
# NOTE: We use a placeholder AMI ID, as Localstack does not validate it.
echo "Creating EC2 instance..."
aws --endpoint-url=http://localstack:4566 ec2 run-instances --image-id ami-12345678 --instance-type t2.micro --count 1 --region us-east-1

echo "Finished creating AWS resources."
