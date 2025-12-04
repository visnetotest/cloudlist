# Cloudlist Technical FAQ

This document answers frequently asked questions about Cloudlist's architecture, technology stack, and development practices.

## 1. Technology Stack and Dependencies

### Q: What is the complete technology stack for Cloudlist?

Cloudlist is written entirely in **Go (Golang)**. It is a self-contained, cross-platform binary with no external runtime dependencies.

-   **Core Language:** Go (version 1.17+)
-   **CLI Framework:** `goflags` - A lightweight, Project Discovery-native library for command-line flag parsing.
-   **Logging:** `gologger` - A simple and configurable logging library, also from Project Discovery.
-   **Concurrency:** `github.com/alitto/pond/v2` - A high-performance, goroutine-based worker pool is used to manage concurrent API calls to providers efficiently.

### Q: What are the project's main external dependencies?

Cloudlist relies on the official Go SDKs for each cloud provider it supports. These are the primary dependencies:

-   **AWS:** `github.com/aws/aws-sdk-go`
-   **Azure:** `github.com/Azure/azure-sdk-for-go`
-   **Google Cloud:** `google.golang.org/api`
-   **DigitalOcean:** `github.com/digitalocean/godo`
-   **Scaleway:** `github.com/scaleway/scaleway-sdk-go`
-   **Linode:** `github.com/linode/linodego`
-   **Fastly:** `github.com/fastly/go-fastly/v3`
-   **Heroku:** `github.com/heroku/heroku-go/v5`
-   **Cloudflare:** `github.com/cloudflare/cloudflare-go`
-   **Namecheap:** `github.com/namecheap/go-namecheap-sdk/v2`

For a complete and up-to-date list, please refer to the `go.mod` file in the project root.

## 2. Architectural Decisions

### Q: What were the key architectural decisions made, and why?

1.  **Provider-Based Interface (`schema.Provider`)**:
    *   **Decision**: Instead of a monolithic application, the logic for each cloud provider is isolated into its own package and must implement a common `Provider` interface.
    *   **Reasoning**: This de-couples the core enumeration engine from the provider-specific implementations. It makes the codebase highly modular and allows new providers to be added with zero impact on existing ones. It promotes separation of concerns and simplifies maintenance.

2.  **Automatic Resource Deduplication**:
    *   **Decision**: Discovered assets (IPs and hostnames) are automatically deduplicated by the `schema.Resources` object when using the `Append()` method.
    *   **Reasoning**: Cloud environments are complex, and the same asset can often be discovered through multiple APIs (e.g., a VM's IP and a DNS record pointing to it). By handling deduplication at the core data structure level, we simplify provider implementations and ensure the final output is clean and non-redundant, without placing that burden on each developer.

3.  **Configuration-Driven Enumeration**:
    *   **Decision**: Cloudlist is driven by a YAML configuration file (`provider-config.yaml`) that defines which providers to scan and with what credentials.
    *   **Reasoning**: This makes the tool flexible and scriptable. Users can maintain separate configuration files for different environments (e.g., dev, prod) and easily switch between them. It also provides a secure way to manage credentials using environment variables.

## 3. Development and Contribution

### Q: Is there a development roadmap or future milestones planned?

While there is no formal public roadmap, the project's direction is guided by community feedback and the evolving cloud landscape. Potential future milestones include:

*   **Support for More Providers:** Adding new cloud and IaaS providers based on user requests.
*   **Expanded Service Coverage:** Adding support for more services within existing providers (e.g., serverless functions, database-as-a-service endpoints).
*   **IPv6 Support:** Ensuring consistent and comprehensive discovery of IPv6 assets across all providers.
*   **Enhanced Metadata:** Enriching the output with more contextual metadata, such as tags, regions, and instance sizes.

### Q: How can I contribute?

Contributions are welcome! Please refer to the `CONTRIBUTING.md` file (if available) for detailed guidelines. The general process is:

1.  **Fork** the repository.
2.  Create a new **branch** for your feature or bugfix.
3.  Write your code and add **tests**.
4.  Ensure your code passes **linting** checks (`golangci-lint run`).
5.  Submit a **Pull Request** with a clear description of your changes.

## 4. Troubleshooting and Security

### Q: How can I troubleshoot common errors like authentication failures?

1.  **Check Credentials:** The vast majority of errors are due to incorrect or insufficient API credentials. Double-check your tokens, keys, and secrets.
2.  **Verify Permissions:** Ensure the credentials you are using have the necessary read-only/viewer permissions to list the resources (e.g., IAM policies in AWS, roles in Azure).
3.  **Whitelist IP Addresses:** Some providers (like Namecheap) require you to whitelist the IP address from which you are making API calls.
4.  **Use `-v` for Verbose Output:** Run Cloudlist with the `-v` flag to see verbose logging, which can provide more context on which provider or API call is failing.
5.  **Check for API Rate Limiting:** If you are scanning a very large account, you may hit API rate limits. Cloudlist has some built-in retries, but persistent failures may require you to run the tool less frequently.

### Q: How does the tool handle credentials and sensitive data?

Cloudlist is designed with security in mind. The recommended way to handle secrets is to **use environment variables**.

-   **In your `provider-config.yaml`:** Reference the environment variable using the `$` prefix.

    ```yaml
    aws:
      - id: "aws-prod"
        aws_access_key: "$AWS_ACCESS_KEY_ID"
        aws_secret_key: "$AWS_SECRET_ACCESS_KEY"
        aws_session_token: "$AWS_SESSION_TOKEN"
    ```

-   **In your shell:** Export the variables before running the tool.

    ```bash
    export AWS_ACCESS_KEY_ID="..."
    export AWS_SECRET_ACCESS_KEY="..."
    export AWS_SESSION_TOKEN="..."
    ./cloudlist -pc provider-config.yaml
    ```

This practice prevents you from hardcoding sensitive information in plaintext files.

## 5. Advanced Usage

### Q: Can you provide advanced use cases combining Cloudlist with other tools?

Cloudlist is a powerful enumeration source for security workflows. Here are some examples:

1.  **Port Scanning Discovered IPs with Nmap:**
    *   Find all IP addresses and scan them for open ports.
    ```bash
    ./cloudlist -pc config.yaml -ip | nmap -iL - -T4 -p- -oN nmap_results.txt
    ```

2.  **Finding HTTP/HTTPS Servers with `httpx`:**
    *   Discover all assets, filter for unique IPs/hosts, and probe for running web servers.
    ```bash
    ./cloudlist -pc config.yaml | sort -u | httpx -o http_servers.txt
    ```

3.  **Subdomain Enumeration on Discovered Domains with `subfinder`:**
    *   Extract all unique hostnames and use them as a seed for further subdomain discovery.
    ```bash
    ./cloudlist -pc config.yaml -host | sort -u | subfinder -iL - -o subdomains.txt
    ```

4.  **JSON-based Asset Inventory and Querying with `jq`:**
    *   Create a detailed JSON inventory and then query it for specific information, like finding all EC2 instances in a particular region.
    ```bash
    ./cloudlist -pc config.yaml -json > assets.json
    cat assets.json | jq 'select(.provider == "aws" and .service == "ec2" and .metadata.region == "us-west-2")'
    ```
