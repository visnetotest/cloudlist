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
