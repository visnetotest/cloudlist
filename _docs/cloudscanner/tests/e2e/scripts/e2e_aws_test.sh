#!/bin/bash

# AWS E2E Test Script for CloudScanner
# This script performs comprehensive AWS provider tests using LocalStack

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration - use absolute paths for simplicity
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLOUDSCANNER_BIN="/Volumes/MacMini4-ssd/home/Users/kong/code_security/cloudlist/target/release/cloudscanner"
CONFIG_DIR="/Volumes/MacMini4-ssd/home/Users/kong/code_security/cloudlist/cloudscanner/tests/e2e/configs"
POLICIES_DIR="/Volumes/MacMini4-ssd/home/Users/kong/code_security/cloudlist/cloudscanner/tests/e2e/policies"
RESULTS_DIR="/Volumes/MacMini4-ssd/home/Users/kong/code_security/cloudlist/cloudscanner/tests/e2e/results"

# LocalStack Configuration
LOCALSTACK_HOST="localhost"
LOCALSTACK_PORT="4566"
AWS_REGION="us-east-1"
AWS_ENDPOINT_URL="http://${LOCALSTACK_HOST}:${LOCALSTACK_PORT}"

# Test Results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
TEST_RESULTS=()

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

log_test() {
    echo -e "${PURPLE}[TEST]${NC} $1"
}

log_result() {
    echo -e "${CYAN}[RESULT]${NC} $1"
}

# Initialize test environment
init_test_env() {
    log_info "Initializing AWS E2E test environment..."
    
    # Check if CloudScanner binary exists
    if [ ! -f "$CLOUDSCANNER_BIN" ]; then
        log_error "CloudScanner binary not found at $CLOUDSCANNER_BIN"
        log_info "Please build CloudScanner first: cargo build --release"
        exit 1
    fi
    
    # Create results directory
    mkdir -p "$RESULTS_DIR"
    
    # Set AWS environment variables for LocalStack
    export AWS_ACCESS_KEY_ID="test"
    export AWS_SECRET_ACCESS_KEY="test"
    export AWS_DEFAULT_REGION="$AWS_REGION"
    export AWS_ENDPOINT_URL="$AWS_ENDPOINT_URL"
    
    # Check if LocalStack is running
    if ! docker ps | grep -q "localstack-main"; then
        log_error "LocalStack container is not running. Please run setup script first."
        exit 1
    fi
    
    # Check if LocalStack is responding
    if ! curl -s "http://${LOCALSTACK_HOST}:${LOCALSTACK_PORT}/health" &>/dev/null; then
        log_warning "LocalStack health check failed, but container is running. Proceeding anyway..."
    fi
    
    log_success "Test environment initialized"
}

# Run a single test case
run_test() {
    local test_name="$1"
    local config_file="$2"
    local expected_resources="$3"
    local description="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Running: $test_name"
    log_info "Description: $description"
    log_info "Config: $config_file"
    log_info "Expected resources: $expected_resources"
    
    local result_file="$RESULTS_DIR/${test_name}.json"
    local log_file="$RESULTS_DIR/${test_name}.log"
    
    # Run CloudScanner
    if "$CLOUDSCANNER_BIN" \
        --config "$config_file" \
        --policies "$POLICIES_DIR" \
        --output-format json \
        > "$result_file" 2> "$log_file"; then
        
        # Validate results
        local actual_resources=$(jq -r '.total_resources' "$result_file" 2>/dev/null || echo "0")
        
        if [ "$actual_resources" -eq "$expected_resources" ]; then
            log_result "✅ PASS: Found $actual_resources resources (expected $expected_resources)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TEST_RESULTS+=("PASS:$test_name")
        else
            log_result "❌ FAIL: Found $actual_resources resources (expected $expected_resources)"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TEST_RESULTS+=("FAIL:$test_name")
            log_info "Check log file: $log_file"
            log_info "Check result file: $result_file"
        fi
        
        # Show resource breakdown
        log_info "Resource types found:"
        jq -r '.resources_by_type | to_entries[] | "  \(.key): \(.value)"' "$result_file" 2>/dev/null || log_warning "Could not parse resource types"
        
    else
        log_result "❌ FAIL: CloudScanner execution failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
        log_info "Check log file: $log_file"
    fi
    
    echo "----------------------------------------"
}

# Run policy evaluation test
run_policy_test() {
    local test_name="$1"
    local config_file="$2"
    local expected_violations="$3"
    local description="$4"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Running Policy Test: $test_name"
    log_info "Description: $description"
    
    local result_file="$RESULTS_DIR/${test_name}_policy.json"
    local log_file="$RESULTS_DIR/${test_name}_policy.log"
    
    # Run CloudScanner with policies
    if "$CLOUDSCANNER_BIN" \
        --config "$config_file" \
        --policies "$POLICIES_DIR" \
        --output-format json \
        > "$result_file" 2> "$log_file"; then
        
        # Check for policy violations in logs
        local violations=$(grep -c "violations detected" "$log_file" 2>/dev/null || echo "0")
        
        if [ "$violations" -ge "$expected_violations" ]; then
            log_result "✅ PASS: Found $violations policy violations (expected at least $expected_violations)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TEST_RESULTS+=("PASS:$test_name")
        else
            log_result "❌ FAIL: Found $violations policy violations (expected at least $expected_violations)"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TEST_RESULTS+=("FAIL:$test_name")
        fi
        
        # Show policy evaluation details
        log_info "Policy evaluation summary:"
        grep "Policy engine evaluating" "$log_file" 2>/dev/null || log_warning "No policy evaluation found in logs"
        
    else
        log_result "❌ FAIL: Policy evaluation failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    fi
    
    echo "----------------------------------------"
}

# Test AWS S3 Discovery
test_aws_s3_discovery() {
    log_info "=== Testing AWS S3 Discovery ==="
    
    run_test "aws_s3_only" \
        "$CONFIG_DIR/aws_s3_test.toml" \
        3 \
        "Discover S3 buckets only (should find 3 buckets)"
    
    run_test "aws_s3_localstack" \
        "$CONFIG_DIR/aws_localstack_test.toml" \
        3 \
        "Discover S3 buckets using LocalStack endpoint"
}

# Test AWS EC2 Discovery
test_aws_ec2_discovery() {
    log_info "=== Testing AWS EC2 Discovery ==="
    
    run_test "aws_ec2_only" \
        "$CONFIG_DIR/aws_ec2_test.toml" \
        3 \
        "Discover EC2 instances only (should find 3 instances)"
    
    run_test "aws_ec2_localstack" \
        "$CONFIG_DIR/aws_localstack_test.toml" \
        3 \
        "Discover EC2 instances using LocalStack endpoint"
}

# Test AWS Multi-Service Discovery
test_aws_multi_service() {
    log_info "=== Testing AWS Multi-Service Discovery ==="
    
    run_test "aws_multi_service" \
        "$CONFIG_DIR/aws_multi_test.toml" \
        6 \
        "Discover both EC2 instances and S3 buckets (should find 6 total resources)"
}

# Test Policy Evaluation
test_policy_evaluation() {
    log_info "=== Testing Policy Evaluation ==="
    
    run_policy_test "s3_public_policy" \
        "$CONFIG_DIR/aws_s3_test.toml" \
        1 \
        "Detect public S3 buckets (should find at least 1 violation)"
    
    run_policy_test "s3_encryption_policy" \
        "$CONFIG_DIR/aws_s3_test.toml" \
        1 \
        "Detect unencrypted S3 buckets (should find at least 1 violation)"
    
    run_policy_test "ec2_public_ip_policy" \
        "$CONFIG_DIR/aws_ec2_test.toml" \
        1 \
        "Detect EC2 instances with public IPs"
}

# Test Output Formats
test_output_formats() {
    log_info "=== Testing Output Formats ==="
    
    local test_name="output_format_console"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Testing Console Output Format"
    
    if "$CLOUDSCANNER_BIN" \
        --config "$CONFIG_DIR/aws_s3_test.toml" \
        --output-format console \
        > "$RESULTS_DIR/${test_name}.txt" 2> "$RESULTS_DIR/${test_name}.log"; then
        
        if grep -q "Cloud Scanner Report" "$RESULTS_DIR/${test_name}.txt"; then
            log_result "✅ PASS: Console output format working"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TEST_RESULTS+=("PASS:$test_name")
        else
            log_result "❌ FAIL: Console output format invalid"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TEST_RESULTS+=("FAIL:$test_name")
        fi
    else
        log_result "❌ FAIL: Console output format failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    fi
    
    echo "----------------------------------------"
    
    test_name="output_format_yaml"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Testing YAML Output Format"
    
    if "$CLOUDSCANNER_BIN" \
        --config "$CONFIG_DIR/aws_s3_test.toml" \
        --output-format yaml \
        > "$RESULTS_DIR/${test_name}.yaml" 2> "$RESULTS_DIR/${test_name}.log"; then
        
        if python3 -c "import yaml; yaml.safe_load(open('$RESULTS_DIR/${test_name}.yaml'))" 2>/dev/null; then
            log_result "✅ PASS: YAML output format valid"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TEST_RESULTS+=("PASS:$test_name")
        else
            log_result "❌ FAIL: YAML output format invalid"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TEST_RESULTS+=("FAIL:$test_name")
        fi
    else
        log_result "❌ FAIL: YAML output format failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    fi
    
    echo "----------------------------------------"
}

# Test Error Handling
test_error_handling() {
    log_info "=== Testing Error Handling ==="
    
    local test_name="invalid_config"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Testing Invalid Configuration File"
    
    # Create invalid config
    echo "invalid toml content" > "$RESULTS_DIR/invalid.toml"
    
    if "$CLOUDSCANNER_BIN" \
        --config "$RESULTS_DIR/invalid.toml" \
        > "$RESULTS_DIR/${test_name}.txt" 2> "$RESULTS_DIR/${test_name}.log"; then
        
        log_result "❌ FAIL: Should have failed with invalid config"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    else
        log_result "✅ PASS: Correctly failed with invalid config"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TEST_RESULTS+=("PASS:$test_name")
    fi
    
    echo "----------------------------------------"
    
    test_name="nonexistent_config"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Testing Nonexistent Configuration File"
    
    if "$CLOUDSCANNER_BIN" \
        --config "/nonexistent/config.toml" \
        > "$RESULTS_DIR/${test_name}.txt" 2> "$RESULTS_DIR/${test_name}.log"; then
        
        log_result "❌ FAIL: Should have failed with nonexistent config"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    else
        log_result "✅ PASS: Correctly failed with nonexistent config"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TEST_RESULTS+=("PASS:$test_name")
    fi
    
    echo "----------------------------------------"
}

# Test Performance
test_performance() {
    log_info "=== Testing Performance ==="
    
    local test_name="performance_test"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    log_test "Testing Discovery Performance"
    
    local start_time=$(date +%s.%N)
    
    if "$CLOUDSCANNER_BIN" \
        --config "$CONFIG_DIR/aws_multi_test.toml" \
        --output-format json \
        > "$RESULTS_DIR/${test_name}.json" 2> "$RESULTS_DIR/${test_name}.log"; then
        
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "1")
        
        # Should complete within 30 seconds for LocalStack
        if (( $(echo "$duration < 30" | bc -l 2>/dev/null || echo "1") )); then
            log_result "✅ PASS: Discovery completed in ${duration}s (under 30s)"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TEST_RESULTS+=("PASS:$test_name")
        else
            log_result "❌ FAIL: Discovery took too long: ${duration}s (over 30s)"
            FAILED_TESTS=$((FAILED_TESTS + 1))
            TEST_RESULTS+=("FAIL:$test_name")
        fi
    else
        log_result "❌ FAIL: Performance test failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL:$test_name")
    fi
    
    echo "----------------------------------------"
}

# Generate test report
generate_report() {
    log_info "=== Generating Test Report ==="
    
    local report_file="$RESULTS_DIR/test_report_$(date +%Y%m%d_%H%M%S).txt"
    
    cat > "$report_file" << EOF
========================================
CloudScanner AWS E2E Test Report
========================================

Test Date: $(date)
Test Environment: LocalStack (AWS Mock)
CloudScanner Binary: $CLOUDSCANNER_BIN

Test Summary:
- Total Tests: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS
- Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%

Detailed Results:
EOF

    for result in "${TEST_RESULTS[@]}"; do
        local status=$(echo "$result" | cut -d':' -f1)
        local name=$(echo "$result" | cut -d':' -f2)
        
        if [ "$status" = "PASS" ]; then
            echo "✅ PASS: $name" >> "$report_file"
        else
            echo "❌ FAIL: $name" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

Files Generated:
- Test Results: $RESULTS_DIR/
- Config Files: $CONFIG_DIR/
- Policy Files: $POLICIES_DIR/

Next Steps:
- Review failed tests and logs
- Check resource discovery accuracy
- Validate policy evaluation results
- Performance optimization if needed

EOF

    log_success "Test report generated: $report_file"
    
    # Display summary
    echo
    echo "========================================"
    echo "Test Summary"
    echo "========================================"
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    echo "Success Rate: $(( PASSED_TESTS * 100 / TOTAL_TESTS ))%"
    echo "========================================"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        log_success "🎉 All tests passed!"
        return 0
    else
        log_warning "⚠️  Some tests failed. Check the report for details."
        return 1
    fi
}

# Cleanup test artifacts
cleanup_tests() {
    log_info "Cleaning up test artifacts..."
    
    # Remove temporary files
    rm -f "$RESULTS_DIR/invalid.toml"
    
    log_success "Cleanup completed"
}

# Main execution
main() {
    echo "========================================"
    echo "CloudScanner AWS E2E Tests"
    echo "========================================"
    echo
    
    init_test_env
    
    # Run all test suites
    test_aws_s3_discovery
    test_aws_ec2_discovery
    test_aws_multi_service
    test_policy_evaluation
    test_output_formats
    test_error_handling
    test_performance
    
    # Generate final report
    generate_report
    
    # Cleanup
    cleanup_tests
    
    echo
    echo "Test completed. Check results in: $RESULTS_DIR"
}

# Handle script interruption
trap cleanup_tests EXIT

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Please install missing dependencies and try again."
        exit 1
    fi
}

# Run dependency check
check_dependencies

# Run main function
main "$@"