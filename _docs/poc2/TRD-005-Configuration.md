# TRD-005: TOML-Based Configuration

## 1. Status

**Decided**

## 2. Context

The `cloudscanner` application requires a flexible and user-friendly method for configuring its core settings and the various provider plugins it manages. Configuration needs to be clearly structured, easy to edit, and capable of handling both global parameters (like logging) and provider-specific details (like API credentials, regions, or project IDs).

## 3. Decision

All application and provider configuration will be managed through a single TOML file named `cloudscanner.toml`. TOML (Tom's Obvious, Minimal Language) is chosen for its clear, explicit semantics and excellent human readability, which aligns with the project's goal of providing a streamlined and intuitive user experience.

The configuration structure will consist of a main `[core]` section for global settings and an array of `[[provider]]` tables for defining the configuration for each active provider plugin.

## 4. Consequences

### 4.1. Advantages

*   **Readability:** TOML is designed to be easy for humans to read and write, reducing the risk of configuration errors.
*   **Structured & Explicit:** The format clearly distinguishes between different sections, making it obvious how global settings and provider configurations are separated.
*   **Flexibility:** Using an array of tables (`[[provider]]`) allows users to define multiple configurations for different providers or even multiple instances of the same provider (e.g., scanning different AWS organizations with separate roles).
*   **Standardized:** TOML is a well-established standard with robust and efficient parsing libraries available in the Rust ecosystem (e.g., the `toml` crate).

### 4.2. Disadvantages

*   **Verbosity for Lists:** Defining lists of complex objects can be more verbose in TOML than in YAML.
*   **Ecosystem Adoption:** While popular in the Rust ecosystem, TOML is not as universally adopted as YAML or JSON in the broader cloud-native tooling space.

## 5. Example Configuration: `cloudscanner.toml`

This example demonstrates how to configure global settings and multiple providers.

```toml
# Global settings for the cloudscanner core engine
[core]
# Log level can be "debug", "info", "warn", or "error"
log_level = "info"
# Log format can be "text" or "json" for machine-readable output
log_format = "json"
# Optional: Override the default directory for loading dynamic plugins
plugins_dir = "/usr/local/lib/cloudscanner/plugins"

# Provider configurations are defined in an array of tables.
# Each table represents a single provider instance to be loaded.

[[provider]]
# The 'name' must match the name exposed by the provider plugin (e.g., "aws").
name = "aws"
enabled = true

# Provider-specific settings are passed directly to the plugin as a key-value map.
# This example shows a potential configuration for the AWS provider.
[provider.config]
regions = ["us-east-1", "eu-west-1", "ap-southeast-2"]
roles_to_assume = [
  "arn:aws:iam::123456789012:role/CloudlistScannerRole",
  "arn:aws:iam::987654321098:role/CloudlistScannerRole"
]

[[provider]]
name = "gcp"
enabled = true

[provider.config]
project_ids = ["my-gcp-project-1", "my-gcp-project-2"]
# The plugin should resolve the path, including home directory shorthand (~)
credentials_file = "~/.config/gcloud/application_default_credentials.json"

[[provider]]
# This demonstrates a disabled provider, which will be ignored by the engine.
name = "digitalocean"
enabled = false

[provider.config]
# The plugin can be designed to read the token from an environment variable.
api_token_env_var = "DO_API_TOKEN"

```

## 6. Reference

This decision is a core component of the application's architecture and is referenced in the main technical specification: [poc2.md#3.1.-High-Level-Component-Diagram](./poc2.md#3.1.-High-Level-Component-Diagram)
