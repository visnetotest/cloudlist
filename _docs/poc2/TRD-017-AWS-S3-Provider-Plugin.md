# TRD-017: AWS S3 Provider Plugin Implementation

- **Status:** Proposed
- **Author:** Gemini
- **Date:** 2024-05-22

## 1. Context

This document describes the technical decision for implementing the AWS S3 provider plugin as part of Goal G-1 for PoC2. The `cloudscanner` engine is being written in Rust and requires a concrete provider implementation to validate the plugin architecture defined in TRD-001 and the language choice in TRD-002.

The S3 service is a critical component of cloud infrastructure and a common source of security misconfigurations, making it an ideal first target for the PoC. The existing Go-based `cloudlist` tool contains a mature S3 provider in `pkg/providers/aws/s3.go`, which will serve as a functional reference.

## 2. Decision

1.  **Crate Structure:** A new Rust crate, `cloudscanner-provider-aws`, will be created within the `cloudscanner/` directory. This crate will encapsulate all AWS-related provider logic, starting with the S3 service.

2.  **SDK Usage:** The official `aws-sdk-rust` (specifically the `aws-sdk-s3` crate) will be used to interact with the AWS S3 API. This ensures long-term maintainability and alignment with AWS best practices.

3.  **Interface Implementation:** The AWS provider will implement the `Provider` trait defined in the core `cloudscanner` engine. This will serve as the contract between the engine and the plugin.

4.  **Core Functionality:** The initial implementation will focus on discovering S3 buckets and gathering essential metadata for security analysis, including:
    - Bucket name and ARN
    - Public access block configuration
    - Access Control Lists (ACLs)
    - Region
    - Creation date

5.  **Data Transformation:** The collected data will be transformed into the standardized `Resource` struct defined by the `cloudscanner` schema before being sent to the core engine.

## 3. Consequences

### Positive

-   Provides a concrete, testable implementation of the provider plugin architecture, which is a core requirement (TR-1).
-   It allows the development of the `cloudscanner`'s discovery engine to proceed with a real-world plugin.
-   Validates the choice of Rust (TR-2) for building performant and safe provider plugins.
-   Aligns the project with the official AWS SDK, ensuring future compatibility.

### Negative

-   This represents a re-implementation of existing provider logic from the Go-based `cloudlist`, which requires dedicated development effort.
-   It adds a significant new dependency (`aws-sdk-rust`) to the `cloudscanner` project.

## 4. Next Steps

1.  **Define `Provider` Trait:** Finalize the `Provider` trait and `Resource` struct in the `cloudscanner` core engine.
2.  **Create Crate:** Initialize the `cloudscanner-provider-aws` crate.
3.  **Implement S3 Logic:** Implement the S3 bucket discovery logic using the `aws-sdk-s3` crate.
4.  **Integration Test:** Create an integration test that uses the existing LocalStack environment to verify that the plugin can successfully discover a sample S3 bucket.
