# Linode Provider Documentation

This document provides a detailed overview of the Linode provider for Cloudlist.

## 1. Provider Overview

The Linode provider discovers assets within a Linode account, including Linode instances, domains, and NodeBalancers.

## 2. Configuration

The Linode provider authenticates using a Linode Personal Access Token.

| Key          | Required | Description                                                  |
| :----------- | :--- | :--- |
| `linode_token` | **Yes**  | Your Linode API token. Can also be set via `LINODE_TOKEN`.       |
| `id`           | No       | A user-defined identifier for this configuration block.      |

**Example Configuration:**

```yaml
linode:
  - id: "linode-main"
    linode_token: "$LINODE_TOKEN"
```

## 3. Supported Services

The Linode provider enumerates assets from:

*   **Instances:** Public and private IPs of Linode virtual machines.
*   **Domains:** DNS records configured in the Linode DNS manager.
*   **NodeBalancers:** Public IPs of Linode's load balancers.

## 4. Architecture and Interaction

The provider calls separate functions to gather resources from each supported service concurrently.

### Component Interaction Diagram

```mermaid
graph TD
    A[Runner] --> B("Linode Provider `Resources()` method");
    B --> C[Get Instances];
    B --> D[Get Domains];
    B --> E[Get NodeBalancers];

    subgraph "Linode Provider (`pkg/providers/linode`)"
        B
        C
        D
        E
    end

    C --> F{Linode API};
    D --> F;
    E --> F;

    F --> C;
    F --> D;
    F --> E;

    C --> A;
    D --> A;
    E --> A;
```

### User & System Interaction Flow

```mermaid
sequenceDiagram
    participant User
    participant CloudlistCLI
    participant LinodeProvider
    participant LinodeAPI

    User->>CloudlistCLI: Runs `./cloudlist -p linode`
    CloudlistCLI->>LinodeProvider: `Resources(ctx)`
    LinodeProvider->>LinodeAPI: List Instances
    LinodeAPI-->>LinodeProvider: Returns instances
    LinodeProvider->>LinodeAPI: List Domains
    LinodeAPI-->>LinodeProvider: Returns domains
    LinodeProvider->>LinodeAPI: List NodeBalancers
    LinodeAPI-->>LinodeProvider: Returns NodeBalancers
    LinodeProvider-->>CloudlistCLI: Aggregates all resources
    CloudlistCLI-->>User: Prints final asset list
```
