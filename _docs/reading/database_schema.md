# Database Schema

## Data Model

We will use a graph-based data model to represent assets and their relationships. The core entities are **Nodes** and **Edges**.

### Nodes (Assets)

A node represents a single cloud asset (e.g., an EC2 instance, an S3 bucket, a GCP project).

**Node Properties:**

| Field | Type | Description | Example |
|:---|:---|:---|:---|
| `id` | string | Unique identifier for the asset | `arn:aws:ec2:us-east-1:123456789012:instance/i-0e5f3f2a7b1e9d4a1` |
| `type` | string | The type of asset | `aws_ec2_instance` |
| `provider` | string | The cloud provider | `aws` |
| `region` | string | The cloud region, if applicable | `us-east-1` |
| `properties` | map | A key-value map of asset-specific attributes | `{"instance_type": "t2.micro", "image_id": "ami-0c55b159cbfafe1f0"}` |
| `tags` | map | A key-value map of user-defined tags | `{"owner": "security-team", "project": "asset-inventory"}` |

### Edges (Relationships)

An edge represents a relationship between two assets.

**Edge Properties:**

| Field | Type | Description | Example |
|:---|:---|:---|:---|
| `id` | string | Unique identifier for the relationship | `rel_12345` |
| `from` | string | The ID of the source node | `arn:aws:ec2:us-east-1:123456789012:instance/i-0e5f3f2a7b1e9d4a1` |
| `to` | string | The ID of the target node | `arn:aws:s3:::my-bucket` |
| `type` | string | The type of relationship | `READS_FROM` |
