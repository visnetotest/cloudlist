# TRD-018: YAML Policy for Public S3

- **Status:** Proposed
- **Author:** Gemini
- **Date:** 2024-05-22

## 1. Context

This document describes the technical decision for implementing the "YAML Policy for Public S3" as part of Goal G-2 for PoC2. This builds upon the high-level policy engine design outlined in `TRD-004-Policy-Engine.md`. To achieve a successful end-to-end scan (G-3), we need a concrete policy definition that can be evaluated against the resources discovered by the AWS S3 Provider Plugin (G-1, TRD-017).

The core principle is to keep the policy rules simple and delegate the complex logic of determining an asset's state to the provider plugin. The provider will inspect the resource and attach clear, queryable metadata for the policy engine to evaluate.

## 2. Decision

1.  **Policy Structure:** The policy to detect public S3 buckets will be defined in a YAML file, following the structure specified in TRD-004.

2.  **Provider Responsibility:** The AWS S3 provider plugin (`cloudscanner-provider-aws`) will be responsible for determining if a bucket is public. It will analyze the bucket's ACLs, Public Access Block settings, and other relevant configurations. If the bucket is deemed public, the provider will add a specific key-value pair to the asset's metadata: `is_public: "true"`. This abstracts the complex logic away from the policy itself.

3.  **Rule Definition:** The policy will consist of a set of simple, conjunctive (`AND`) rules. An asset will be flagged as non-compliant if all rules in the policy match. The specific policy will be:

    ```yaml
    policies:
      - id: s3-public-read-prohibited
        name: "Public S3 Buckets Prohibited"
        description: "This policy flags any S3 bucket that is publicly accessible."
        rules:
          - field: asset_type
            value: "s3_bucket"
          - field: metadata.is_public
            value: "true"
    ```

4.  **Engine Implementation:** The Rust-based policy engine in `cloudscanner` will be responsible for:
    -   Parsing the YAML policy files.
    -   Iterating through the discovered assets.
    -   For each asset, checking if all rules within a policy evaluate to true.
    -   Flagging assets that match the policy.

## 3. Consequences

### Positive

-   **Separation of Concerns:** The provider handles the "how" (determining public status), while the policy handles the "what" (flagging public buckets). This makes policies clean and readable.
-   **Reusability:** The `metadata.is_public` field could be reused by other policies or reporting tools.
-   **Simplicity:** Keeps the policy engine itself simple, as it only needs to perform basic key-value matching.

### Negative

-   **Lack of Transparency in Policy:** An auditor reading only the policy file will not know the exact criteria for `is_public`. This logic is contained within the provider's implementation. This is an accepted trade-off for the PoC to maintain policy simplicity.

## 4. Next Steps

1.  **Implement Policy Engine:** Build the core logic in the Rust `cloudscanner` engine to parse and evaluate the YAML policies as described.
2.  **Update S3 Provider:** Ensure the `cloudscanner-provider-aws` crate correctly implements the public bucket detection logic and sets the `metadata.is_public` flag.
3.  **Integration Test:** Create an end-to-end test that loads this policy, runs a scan against a LocalStack environment containing a public S3 bucket, and asserts that the bucket is correctly flagged as non-compliant.
