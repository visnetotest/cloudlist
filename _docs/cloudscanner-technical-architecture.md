# CloudScanner Technical Architecture

## Overview

CloudScanner is a multi-cloud security scanning tool written in Rust that discovers cloud assets and evaluates them against security policies. The architecture follows a modular, plugin-based design with strong security boundaries and async/await concurrency patterns.

## Core Architecture

### High-Level Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Entry     │    │  Discovery      │    │   Policy        │
│   Point         │───▶│  Engine         │───▶│   Engine        │
│  (main.rs)      │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Configuration │    │   Providers     │    │   Reporter      │
│   Management    │    │   (Plugins)     │    │   (Output)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Module Structure

```
src/
├── main.rs              # CLI entry point and orchestration
├── lib.rs               # Module declarations
├── config/              # Configuration management
│   ├── mod.rs
│   └── tests.rs
├── engine/              # Discovery engine core
│   ├── mod.rs
│   └── tests.rs
├── models/              # Data models and traits
│   ├── provider.rs
│   └── tests.rs
├── providers/           # Cloud provider implementations
│   ├── mod.rs
│   ├── base.rs          # Base provider traits
│   └── aws/             # AWS-specific providers
├── policy/              # Policy evaluation engine
│   ├── mod.rs
│   ├── policy.rs
│   └── tests.rs
├── reporter/            # Output formatting and reporting
│   ├── mod.rs
│   └── tests.rs
└── error/               # Error handling
    ├── mod.rs
    └── tests.rs
```

## Core Components

### 1. CLI Entry Point (`main.rs`)

The main function orchestrates the entire scanning pipeline:

1. **Configuration Loading**: Parses TOML configuration files
2. **Component Initialization**: Creates discovery engine, policy engine, and reporter
3. **Provider Loading**: Loads and initializes cloud providers
4. **Policy Loading**: Loads security policies from YAML files
5. **Execution Pipeline**: Runs discovery → evaluation → reporting

Key features:
- Async/await with Tokio runtime
- Structured logging with tracing
- Error handling with anyhow
- CLI argument parsing with clap

### 2. Configuration Management (`config/`)

Handles secure loading and validation of configuration:

**Configuration Structure**:
```toml
[[provider]]
id = "aws-provider"
type = "builtin-aws"  # or "plugin"
plugin_path = "plugins/aws.so"  # for plugin type
config = { region = "us-east-1" }
```

**Security Features**:
- Path traversal protection
- File size limits (10MB max)
- Input sanitization
- Dangerous character detection
- Script injection prevention

### 3. Discovery Engine (`engine/`)

Core orchestration component that manages provider execution:

**Key Features**:
- **Plugin Architecture**: Dynamic loading of provider libraries (.so/.dylib/.dll)
- **Security Validation**: Plugin file validation and sandboxing
- **Concurrent Execution**: Controlled parallelism with semaphores
- **Error Isolation**: Provider failures don't affect other providers
- **Resource Management**: RAII pattern for library cleanup

**Plugin Loading Process**:
1. Security validation (file size, permissions, content scanning)
2. Platform-specific library discovery
3. Dynamic library loading with libloading
4. C-compatible interface extraction
5. Provider instance creation

### 4. Provider System (`providers/`)

#### Provider Trait (`models/provider.rs`)

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    fn name(&self) -> String;
    async fn discover(&self) -> Result<Vec<Resource>>;
    async fn health_check(&self) -> Result<bool>;
    async fn cleanup(&self) -> Result<()>;
}
```

#### Resource Model
```rust
pub struct Resource {
    pub asset_type: String,
    pub id: String,
    pub metadata: HashMap<String, String>,
}
```

#### AWS Provider Implementation

Located in `providers/aws/`, supports multiple AWS services:
- EC2 instances
- S3 buckets
- IAM roles/policies
- Lambda functions
- RDS instances
- ECS clusters
- CloudTrail logs
- CloudFront distributions
- VPC configurations
- EFS file systems
- ELB load balancers

### 5. Policy Engine (`policy/`)

Evaluates discovered resources against security policies:

**Policy Structure** (YAML):
```yaml
name: "public-s3-buckets"
asset_type: "s3-bucket"
rules:
  - key: "public_read"
    value: "false"
  - key: "encryption"
    value: "enabled"
```

**Features**:
- Rule-based evaluation
- Metadata matching
- Violation tracking
- Multi-policy support

### 6. Reporter System (`reporter/`)

Handles output formatting and report generation:

**Output Formats**:
- Console (human-readable)
- JSON (machine-readable)
- YAML (structured data)

## Security Architecture

### 1. Plugin Security

**Validation Layers**:
- File size limits (10MB max)
- Permission checks (no world-writable files)
- Content scanning for suspicious patterns
- Extension validation (.so/.dylib/.dll only)
- Platform-specific library discovery

**Runtime Protection**:
- RAII pattern for automatic cleanup
- Memory safety with Rust
- Error isolation between providers
- C-compatible interface with bounds checking

### 2. Input Validation

**Configuration Security**:
- Path traversal prevention
- Script injection detection
- Size limits on all inputs
- Character sanitization
- YAML structure validation

**Plugin Interface Security**:
- String length limits (1024 chars)
- Null byte detection
- UTF-8 validation
- Memory allocation limits
- Safe pointer arithmetic

### 3. Error Handling

**Error Types** (`error/mod.rs`):
- Configuration errors
- Plugin loading errors
- Security violations
- Provider execution errors
- Policy evaluation errors

**Error Sanitization**:
- Sensitive data filtering
- Safe error logging
- Structured error reporting

## Concurrency Model

### Async/Await Architecture

- **Tokio Runtime**: Full async runtime with all features
- **Controlled Parallelism**: Semaphore-based concurrency limiting
- **Resource Isolation**: Each provider runs in isolated tasks
- **Error Propagation**: Structured error handling across async boundaries

### Execution Flow

```
1. Load Configuration (sync)
2. Initialize Components (sync)
3. Load Providers (async, concurrent)
4. Run Discovery (async, concurrent with semaphore)
5. Evaluate Policies (sync, single-threaded)
6. Generate Report (sync)
```

## Plugin System

### C-Compatible Interface

**Plugin Export Function**:
```c
extern "C" fn _create_provider() -> TraitObject
```

**Memory Management**:
- Manual memory allocation for C compatibility
- Explicit cleanup functions
- Bounds checking on all operations
- Safe string conversion utilities

### Provider Types

1. **Built-in Providers**: Compiled into main binary
2. **Plugin Providers**: Dynamically loaded libraries
3. **Hybrid Approach**: Built-in fallbacks for plugin failures

## Data Flow

### Discovery Pipeline

```
Configuration → Provider Loading → Resource Discovery → Policy Evaluation → Report Generation
```

### Resource Flow

```
Cloud APIs → Provider → Resource Model → Policy Engine → Violation Detection → Reporter
```

## Testing Architecture

### Test Categories

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Multi-component interaction
3. **E2E Tests**: Full pipeline testing with LocalStack
4. **Security Tests**: Input validation and plugin security

### Test Infrastructure

- **LocalStack**: Local AWS service simulation
- **Mock Providers**: Test provider implementations
- **Property-Based Testing**: Proptest for edge cases
- **Security Testing**: Malicious input simulation

## Performance Considerations

### Optimizations

1. **Concurrent Discovery**: Parallel provider execution
2. **Memory Efficiency**: RAII and careful resource management
3. **Lazy Loading**: Providers loaded on-demand
4. **Bounded Concurrency**: Semaphore prevents resource exhaustion

### Limits and Safeguards

- Maximum plugin size: 10MB
- Maximum providers: 100
- Maximum resources per provider: 10,000
- Maximum metadata per resource: 100 items
- String length limits: 1024 characters

## Extensibility

### Adding New Providers

1. **Implement Provider Trait**: Create new provider struct
2. **Add to Engine**: Register in discovery engine
3. **Configuration**: Add provider type support
4. **Testing**: Implement comprehensive tests

### Adding New Policy Types

1. **Extend Policy Model**: Add new rule types
2. **Update Engine**: Modify evaluation logic
3. **Documentation**: Update policy format docs

## Deployment Architecture

### Binary Distribution

- **Static Linking**: Self-contained binary
- **Cross-Platform**: Linux, macOS, Windows support
- **Plugin Discovery**: Platform-specific library loading

### Configuration Management

- **TOML Configuration**: Human-readable config format
- **Environment Variables**: Supported in configuration
- **Security Defaults**: Secure-by-default settings

## Monitoring and Observability

### Logging

- **Structured Logging**: Tracing framework
- **Log Levels**: Configurable verbosity
- **Performance Metrics**: Timing and resource counts
- **Error Tracking**: Detailed error reporting

### Health Checks

- **Provider Health**: Optional health check interface
- **System Health**: Memory and resource monitoring
- **Error Rates**: Provider failure tracking

This architecture provides a secure, extensible, and performant foundation for multi-cloud security scanning with strong isolation between components and comprehensive security validation throughout the system.