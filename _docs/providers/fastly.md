# Fastly Provider Documentation

This document provides a detailed overview of the Fastly provider for Cloudlist.

## 1. Provider Overview

The Fastly provider is designed to discover public-facing IP addresses and domains associated with a Fastly account. It interacts with the Fastly API to gather information about services, domains, and backends.

## 2. Configuration

The Fastly provider authenticates using a Fastly API token.

| Key          | Required | Description                                                  |
| :----------- | :--- | :--- |
| `fastly_token` | **Yes**  | Your Fastly API token. Can also be set via `FASTLY_API_TOKEN`. |
| `id`           | No       | A user-defined identifier for this configuration block.      |

**Example Configuration:**

```yaml
fastly:
  - id: "fastly-main"
    fastly_token: "$FASTLY_TOKEN"
```

## 3. Supported Services

The Fastly provider enumerates assets from the following sources:

*   **Services:** Public IPs of Fastly services.
*   **Domains:** Domain names configured for services.
*   **Backends:** Hostnames and IP addresses of backend servers.

## 4. Architecture and Interaction

The provider fetches a list of all services and then inspects each service for its domains and backends.

### Component Interaction Diagram

```mermaid
graph TD
    A[Runner] --> B("Fastly Provider `Resources()` method");
    B --> C[Get All Services];
    C --> D{For each Service};
    D --> E[Get Domains];
    D --> F[Get Backends];

    subgraph "Fastly Provider (`pkg/providers/fastly`)"
        B
        C
        D
        E
        F
    end

    C --> G{Fastly API};
    E --> G;
    F --> G;

    G --> C;
    G --> E;
    G --> F;

    E --> A;
    F --> A;
```

### User & System Interaction Flow

```mermaid
sequenceDiagram
    participant User
    participant CloudlistCLI
    participant FastlyProvider
    participant FastlyAPI

    User->>CloudlistCLI: Runs `./cloudlist -p fastly`
    CloudlistCLI->>FastlyProvider: `Resources(ctx)`
    FastlyProvider->>FastlyAPI: List Services
    FastlyAPI-->>FastlyProvider: Returns services

    loop For each service
        FastlyProvider->>FastlyAPI: Get service details (domains, backends)
        FastlyAPI-->>FastlyProvider: Returns details
    end

    FastlyProvider-->>CloudlistCLI: Aggregates and returns all resources
    CloudlistCLI-->>User: Prints final asset list
```
