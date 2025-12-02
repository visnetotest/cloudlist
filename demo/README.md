# Automated Demo: Policy Engine with LocalStack

This directory contains a fully automated, self-contained demonstration of the `cloudlist` native policy engine.

It uses Docker Compose to orchestrate:

1.  **LocalStack**: A local cloud service emulator for AWS.
2.  **AWS Setup**: A script that creates a public S3 bucket (the violation) and an EC2 instance within LocalStack.
3.  **Cloudlist**: The application itself, which scans the LocalStack environment and evaluates it against a defined policy.

## How to Run the Demo

### Prerequisites

*   Docker
*   Docker Compose

### Instructions

1.  **Navigate to the `demo` directory:**

    ```bash
    cd demo
    ```

2.  **Run Docker Compose:**

    ```bash
    docker-compose up --build
    ```

## Expected Outcome

Docker Compose will build the necessary images and run the services in the correct order.

You will see logs from LocalStack starting up, followed by the `aws-setup` container creating the resources.

Finally, the `cloudlist` container will run. It will scan the LocalStack environment, and its policy engine will find a violation (the public S3 bucket). Because the command includes the `--policy-block` (`-pb`) flag, it will report the violation and exit with a non-zero status code, which you will see in the Docker Compose output.

**Example Output from Cloudlist:**

```
cloudlist-1  | INFO[0003] Listing assets from provider: aws services: s3,ec2 id: localstack
cloudlist-1  | WARN[0004] Found 1 policy violations
cloudlist-1  | WARN[0004]   - Policy: Public S3 Buckets Prohibited, Asset: my-public-bucket-for-demo, Rule: s3_is_public=true
cloudlist-1  | FATA[0004] Exiting due to policy violations and policy-block flag.
```

This output successfully demonstrates that the policy engine has detected and blocked a non-compliant resource in the simulated cloud environment.
