#!/bin/bash

# Configure AWS CLI to use LocalStack
aws configure set aws_access_key_id test
aws configure set aws_secret_access_key test
aws configure set region us-east-1

ENDPOINT_URL=http://localstack:4566

# Create a public S3 bucket (this will violate our policy)
echo "Creating public S3 bucket..."
aws --endpoint-url=$ENDPOINT_URL s3api create-bucket --bucket my-public-bucket-for-demo --region us-east-1
aws --endpoint-url=$ENDPOINT_URL s3api put-bucket-acl --bucket my-public-bucket-for-demo --acl public-read

# Create an EC2 instance
echo "Creating EC2 instance..."
aws --endpoint-url=$ENDPOINT_URL ec2 run-instances --image-id ami-12345678 --instance-type t2.micro --count 1 > /dev/null

echo "AWS resources created."
