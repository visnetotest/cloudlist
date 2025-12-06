# CloudScanner Test Plan

## Overview

This document outlines comprehensive testing strategy for CloudScanner to ensure production readiness, security, reliability, and maintainability.

## Current State

- **Test Coverage**: 0% (No existing tests)
- **Critical Gap**: No unit tests, integration tests, or security validation tests
- **Risk Level**: HIGH - Production deployment without comprehensive test coverage

## Testing Goals

### 1. Functional Testing

- Verify all core functionality works as expected
- Test provider discovery and resource enumeration
- Validate policy evaluation and reporting
- Ensure configuration loading and validation

### 2. Security Testing

- Validate plugin security mechanisms
- Test input validation and sanitization
- Verify error handling doesn't leak sensitive information
- Test plugin sandboxing and isolation

### 3. Performance Testing

- Establish performance baselines
- Test scalability with multiple providers
- Validate timeout handling and resource limits
- Test memory usage and cleanup

### 4. Reliability Testing

- Test error handling and recovery
- Validate graceful degradation
- Test concurrent operations
- Verify resource cleanup

## Test Structure

```
cloudscanner/
├── src/
│   ├── config/
│   │   ├── mod.rs
│   │   └── tests.rs          # Config unit tests
│   ├── engine/
│   │   ├── mod.rs
│   │   └── tests.rs          # Engine unit tests
│   ├── models/
│   │   ├── provider.rs
│   │   └── tests.rs          # Model tests
│   ├── policy/
│   │   ├── mod.rs
│   │   └── tests.rs          # Policy engine tests
│   └── reporter/
│       ├── mod.rs
│       └── tests.rs          # Reporter tests
├── tests/
│   ├── integration_test.rs     # Integration tests
│   ├── security_test.rs       # Security tests
│   ├── performance_test.rs    # Performance tests
│   └── common/
│       ├── mod.rs
│       └── helpers.rs         # Test utilities
└── Cargo.toml               # Test dependencies
```

## High Priority Tests (Must Have)

### 1. Configuration Tests (`src/config/tests.rs`)

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_valid_config() {
        // Test loading valid TOML configuration
    }

    #[test]
    fn test_load_invalid_toml() {
        // Test handling of malformed TOML
    }

    #[test]
    fn test_missing_config_file() {
        // Test handling of missing configuration file
    }

    #[test]
    fn test_empty_provider_id() {
        // Test validation of empty provider IDs
    }

    #[test]
    fn test_empty_provider_type() {
        // Test validation of empty provider types
    }

    #[test]
    fn test_duplicate_provider_ids() {
        // Test handling of duplicate provider IDs
    }
}
```

### 2. Engine Tests (`src/engine/tests.rs`)

```rust
#[cfg(test)]
mod engine_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_provider_loading_success() {
        // Test successful provider loading
    }

    #[tokio::test]
    async fn test_provider_loading_invalid_library() {
        // Test loading invalid library files
    }

    #[tokio::test]
    async fn test_provider_loading_missing_symbol() {
        // Test loading libraries without required symbols
    }

    #[tokio::test]
    async fn test_discovery_with_no_providers() {
        // Test discovery with empty provider list
    }

    #[tokio::test]
    async fn test_discovery_with_multiple_providers() {
        // Test concurrent provider execution
    }

    #[tokio::test]
    async fn test_provider_error_handling() {
        // Test graceful handling of provider failures
    }

    #[tokio::test]
    async fn test_discovery_timeout() {
        // Test timeout handling during discovery
    }
}
```

### 3. Plugin Security Tests (`src/engine/tests.rs` - continued)

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_plugin_size_validation() {
        // Test rejection of oversized plugins
    }

    #[test]
    fn test_plugin_extension_validation() {
        // Test acceptance of valid extensions (.so, .dylib, .dll)
    }

    #[test]
    fn test_plugin_invalid_extension_rejection() {
        // Test rejection of invalid extensions
    }

    #[test]
    fn test_plugin_permission_validation() {
        // Test rejection of world-writable files (Unix)
    }

    #[test]
    fn test_plugin_missing_file() {
        // Test handling of non-existent plugin files
    }

    #[test]
    fn test_trait_object_validation() {
        // Test validation of null trait objects
    }
}
```

### 4. Policy Engine Tests (`src/policy/tests.rs`)

```rust
#[cfg(test)]
mod policy_tests {
    use super::*;
    use crate::models::provider::Resource;

    #[test]
    fn test_load_valid_policy() {
        // Test loading valid YAML policy files
    }

    #[test]
    fn test_load_invalid_yaml() {
        // Test handling of malformed YAML
    }

    #[test]
    fn test_policy_evaluation_match() {
        // Test policy matching resources
    }

    #[test]
    fn test_policy_evaluation_no_match() {
        // Test policy not matching resources
    }

    #[test]
    fn test_policy_missing_metadata() {
        // Test policy evaluation with missing metadata
    }

    #[test]
    fn test_multiple_policy_evaluation() {
        // Test resource evaluation against multiple policies
    }

    #[test]
    fn test_violation_reporting() {
        // Test violation detection and reporting
    }
}
```

### 5. Model Tests (`src/models/tests.rs`)

```rust
#[cfg(test)]
mod model_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resource_creation() {
        // Test resource creation with valid data
    }

    #[test]
    fn test_resource_with_metadata() {
        // Test resource metadata handling
    }

    #[test]
    fn test_resource_unique_id() {
        // Test unique ID generation
    }

    #[test]
    fn test_provider_info_validation() {
        // Test provider info structure validation
    }

    #[test]
    fn test_c_string_conversion() {
        // Test C string conversion functions
    }

    #[test]
    fn test_c_resource_conversion() {
        // Test resource to C resource conversion
    }

    #[test]
    fn test_c_resource_cleanup() {
        // Test memory cleanup for C resources
    }
}
```

### 6. Reporter Tests (`src/reporter/tests.rs`)

```rust
#[cfg(test)]
mod reporter_tests {
    use super::*;
    use crate::models::provider::Resource;

    #[test]
    fn test_console_report_generation() {
        // Test console report format
    }

    #[test]
    fn test_json_report_generation() {
        // Test JSON report format
    }

    #[test]
    fn test_yaml_report_generation() {
        // Test YAML report format
    }

    #[test]
    fn test_empty_resource_report() {
        // Test report with no resources
    }

    #[test]
    fn test_resource_aggregation() {
        // Test resource counting and grouping
    }
}
```

## Integration Tests (`tests/integration_test.rs`)

```rust
use cloudscanner::config;
use cloudscanner::engine;
use cloudscanner::policy;
use cloudscanner::reporter;
use tempfile::{TempDir, NamedTempFile};
use std::fs;
use std::io::Write;

#[tokio::test]
async fn test_full_discovery_pipeline() {
    // Test complete discovery -> policy -> report pipeline
}

#[tokio::test]
async fn test_end_to_end_with_real_plugin() {
    // Test with actual compiled plugin
}

#[tokio::test]
async fn test_multiple_policy_evaluation() {
    // Test multiple policies against discovered resources
}

#[tokio::test]
async fn test_error_recovery_and_continuation() {
    // Test system continues when one provider fails
}

#[tokio::test]
async fn test_configuration_driven_execution() {
    // Test execution driven by configuration file
}
```

## Security Tests (`tests/security_test.rs`)

```rust
#[cfg(test)]
mod security_tests {
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_malicious_large_plugin() {
        // Test rejection of oversized plugins
    }

    #[test]
    fn test_plugin_with_world_writable_permissions() {
        // Test rejection of insecure permissions
    }

    #[test]
    fn test_plugin_with_invalid_extension() {
        // Test rejection of non-library files
    }

    #[test]
    fn test_plugin_manipulation_detection() {
        // Test detection of modified plugin files
    }

    #[test]
    fn test_null_pointer_injection() {
        // Test handling of null pointers from plugins
    }

    #[test]
    fn test_memory_cleanup_on_failure() {
        // Test proper memory cleanup when plugins fail
    }
}
```

## Performance Tests (`tests/performance_test.rs`)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cloudscanner::engine;
use std::time::Duration;

fn benchmark_provider_discovery(c: &mut Criterion) {
    c.bench_function("provider_discovery", |b| {
        b.iter(|| {
            // Benchmark provider discovery performance
        })
    });
}

fn benchmark_policy_evaluation(c: &mut Criterion) {
    c.bench_function("policy_evaluation", |b| {
        b.iter(|| {
            // Benchmark policy evaluation performance
        })
    });
}

fn benchmark_report_generation(c: &mut Criterion) {
    c.bench_function("report_generation", |b| {
        b.iter(|| {
            // Benchmark report generation performance
        })
    });
}

criterion_group!(
    benches,
    benchmark_provider_discovery,
    benchmark_policy_evaluation,
    benchmark_report_generation
);
criterion_main!(benches);
```

## Property-Based Tests (`tests/property_test.rs`)

```rust
use proptest::prelude::*;
use cloudscanner::models::provider::Resource;

proptest! {
    #[test]
    fn test_resource_unique_id_properties(
        asset_type in "[a-z0-9-]+",
        id in "[a-z0-9-]+"
    ) {
        let resource = Resource::new(asset_type.clone(), id.clone());
        let unique_id = resource.unique_id();

        // Test unique ID format invariants
        prop_assert!(unique_id.contains(&asset_type));
        prop_assert!(unique_id.contains(&id));
    }

    #[test]
    fn test_metadata_properties(
        keys in prop::collection::vec(".*", 0..10),
        values in prop::collection::vec(".*", 0..10)
    ) {
        let metadata: HashMap<String, String> = keys
            .into_iter()
            .zip(values.into_iter())
            .collect();

        let resource = Resource::new("test".to_string(), "test-id".to_string())
            .with_metadata("key".to_string(), "value".to_string());

        // Test metadata invariants
        prop_assert!(resource.metadata.contains_key("key"));
    }
}
```

## Test Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.0"
proptest = "1.0"
criterion = { version = "0.5", features = ["html_reports"] }
tokio-test = "0.4"
serial_test = "2.0"
```

## Test Execution

### Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test config::tests
cargo test engine::tests
cargo test policy::tests

# Run with coverage
cargo tarpaulin --out Html
```

### Integration Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --test integration_test

# Run with specific test
cargo test test_full_discovery_pipeline
```

### Performance Tests

```bash
# Run benchmarks
cargo bench

# Generate HTML report
cargo bench -- --output-format html
```

### Security Tests

```bash
# Run security-focused tests
cargo test security_test

# Run with address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test security_test
```

## Test Coverage Requirements

### Minimum Coverage for Production

- **Unit Tests**: 90% line coverage
- **Integration Tests**: 80% feature coverage
- **Security Tests**: 100% of security-critical paths
- **Error Paths**: 100% coverage of error handling

### Coverage Tools

```bash
# Install coverage tools
cargo install cargo-tarpaulin
cargo install cargo-llvm-cov

# Generate coverage reports
cargo tarpaulin --out Html
cargo llvm-cov --html --open
```

## Continuous Integration

### GitHub Actions Workflow (`.github/workflows/test.yml`)

```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: ${{ matrix.rust }}
          override: true

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Run security tests
        run: cargo test security_test

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Test Data Management

### Test Fixtures (`tests/fixtures/`)

```
fixtures/
├── configs/
│   ├── valid.toml
│   ├── invalid.toml
│   └── empty.toml
├── policies/
│   ├── valid.yaml
│   ├── invalid.yaml
│   └── malicious.yaml
├── plugins/
│   ├── valid.so
│   ├── oversized.so
│   └── invalid.txt
└── resources/
    └── sample_resources.json
```

### Mock Providers (`tests/mocks/`)

```
mocks/
├── dummy_provider.rs
├── failing_provider.rs
└── slow_provider.rs
```

## Success Criteria

### Phase 1 (Week 1-2): Foundation

- [ ] All unit tests implemented and passing
- [ ] Basic integration tests working
- [ ] CI pipeline running
- [ ] Coverage reporting functional

### Phase 2 (Week 3-4): Security & Performance

- [ ] Security tests implemented
- [ ] Performance benchmarks established
- [ ] Property-based tests added
- [ ] 90%+ coverage achieved

### Phase 3 (Week 5-6): Production Readiness

- [ ] Load testing completed
- [ ] Fuzz testing implemented
- [ ] Documentation for tests complete
- [ ] Production deployment approved

## Risk Mitigation

### High Risk Areas

1. **Plugin Loading**: Memory safety, symbol resolution
2. **Async Runtime**: Deadlocks, race conditions
3. **Resource Management**: Memory leaks, file handles
4. **Security Validation**: Bypass attempts, edge cases

### Mitigation Strategies

1. **Comprehensive Input Validation**: All external inputs validated
2. **Memory Safety**: Extensive testing of unsafe code paths
3. **Error Path Testing**: Every error condition tested
4. **Security Testing**: Malicious input testing
5. **Property-Based Testing**: Edge case discovery

## Timeline

| Week | Focus          | Deliverables                                   |
| ---- | -------------- | ---------------------------------------------- |
| 1    | Unit Tests     | Config, Engine, Models, Policy, Reporter tests |
| 2    | Integration    | Full pipeline tests, CI setup                  |
| 3    | Security       | Plugin validation, input sanitization tests    |
| 4    | Performance    | Benchmarks, load testing, coverage             |
| 5    | Property-Based | Proptest integration, edge case testing        |
| 6    | Production     | Documentation, final validation, deployment    |

This comprehensive test plan ensures CloudScanner meets production standards for reliability, security, and maintainability.
