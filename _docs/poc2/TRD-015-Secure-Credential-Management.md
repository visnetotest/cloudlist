# TRD-015: Secure Credential Management

## 1. Status

**Proposed**

## 2. Context

`cloudscanner` requires credentials to access cloud provider APIs. Hardcoding secrets (like access keys or API tokens) directly into the `cloudscanner.toml` configuration file is a major security risk. It makes the configuration file highly sensitive and difficult to share or manage in version control. A secure cloud tool must support standard, externalized credential management practices.

## 3. Decision

`cloudscanner` will adopt a layered approach to credential loading, prioritizing secure, industry-standard mechanisms.

1.  **Environment Variables (Highest Priority):** Provider plugins will be designed to automatically detect and use standard environment variables for authentication (e.g., `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `GCP_SERVICE_ACCOUNT_KEY`). This is the preferred method for CI/CD environments and local development.

2.  **Instance Metadata Service:** When running within a cloud environment (e.g., on an EC2 instance or in a Lambda function), the providers will automatically attempt to retrieve credentials from the instance metadata service. This is the most secure method for applications running on cloud infrastructure.

3.  **Configuration File (Last Resort):** While discouraged, the `cloudscanner.toml` file may contain credential information. However, the documentation will strongly recommend using environment variables or instance profiles instead.

The credential loading logic will be embedded within each provider plugin, allowing each to use the specific SDK-standard methods for its ecosystem.

## 4. Consequences

### 4.1. Advantages

*   **Vastly Improved Security:** Avoids the need to store plaintext secrets on disk, which is a critical security best practice.
*   **Simplified Configuration:** Configuration files become less sensitive and can be more easily shared or committed to version control (if they contain no secrets).
*   **Seamless Integration with Cloud Environments:** The tool works out-of-the-box in automated, production-like environments without manual credential configuration.

### 4.2. Disadvantages

*   **Increased Provider Complexity:** Each provider plugin is responsible for implementing this layered credential logic, though most major cloud SDKs provide this functionality out-of-the-box.

## 5. Reference

This decision supports a core security requirement referenced in: [poc2.md#2.-Technical-Requirements](./poc2.md#2.-Technical-Requirements)
