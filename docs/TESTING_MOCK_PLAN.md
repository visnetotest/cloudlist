# AWS Provider Mocking and Testing Plan

This document outlines the plan to implement a mocking framework for the AWS provider in `cloudlist`. The goal is to enable robust unit testing without requiring actual AWS credentials, making tests faster, more reliable, and runnable in any environment, including CI.

This plan follows the testing guidelines mentioned in `CLAUDE.md`: "Unit tests should mock cloud provider clients".

## 1. Problem

Currently, the AWS provider code is tightly coupled to the AWS SDK's concrete client implementations (e.g., `*ec2.Client` from SDK v2, and `*ecs.ECS` from SDK v1). This makes it difficult to write unit tests for the resource enumeration logic in providers like `ec2`, `ecs`, `s3`, etc., as they attempt to make real API calls.

## 2. Proposed Solution: Dependency Injection and Mocking

We will refactor the AWS provider to use interfaces for AWS service clients. This allows us to inject "mock" clients during testing that simulate the behavior of the real AWS services.

The implementation will involve three main steps:
1.  Define interfaces for AWS service clients.
2.  Implement mock clients for testing.
3.  Refactor provider code to use these interfaces (Dependency Injection).

## 3. Detailed Plan

### Step 3.1: Define Client Interfaces

For each AWS service used (EC2, ECS, S3, etc.), we will define an interface that includes only the methods our application calls. This follows the Interface Segregation Principle.

**Example for EC2 (in `pkg/providers/aws/ec2.go` or a new `clients.go`):**

The code seems to use both v1 and v2 SDKs. We need to handle both.
For v2 (`aws.go` and `instances.go`):
```go
// in pkg/providers/aws/instances.go or a new file
type EC2Clientv2 interface {
    DescribeInstances(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error)
    // Add other EC2 methods used by cloudlist here
}
```

For v1 (`ecs.go`):
The v1 SDK already provides interfaces in the `service/*iface` packages (e.g., `github.com/aws/aws-sdk-go/service/ec2/ec2iface.EC2API`). We should use them.

`ecs.go` uses `*ecs.ECS` and `*ec2.EC2`. We should change it to use `ecsiface.ECSAPI` and `ec2iface.EC2API`.

### Step 3.2: Create Mock Implementations

For each interface, we will create a mock implementation for use in tests. These mocks will be part of the test package (`_test`).

**Example for a mock EC2 client (in `pkg/providers/aws/ec2_mock_test.go`):**

```go
// MockEC2Clientv2 is a mock of EC2Clientv2 for testing
type MockEC2Clientv2 struct {
    // you can add fields here to control the mock's behavior
    DescribeInstancesFunc func(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error)
}

func (m *MockEC2Clientv2) DescribeInstances(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error) {
    if m.DescribeInstancesFunc != nil {
        return m.DescribeInstancesFunc(ctx, params, optFns...)
    }
    // Default mock behavior
    return &ec2.DescribeInstancesOutput{}, nil
}
```
A similar approach will be used for v1 SDK mocks.

### Step 3.3: Refactor Providers to Use Interfaces

We will modify the provider structs to hold the interface type instead of the concrete client type.

**Example for `instanceProvider` (in `pkg/providers/aws/instances.go`):**

*Before:*
```go
import "github.com/aws/aws-sdk-go-v2/service/ec2"

type instanceProvider struct {
    id               string
    ec2Client        *ec2.Client // Concrete type
    extendedMetadata bool
}

// ...
instanceProvider := &instanceProvider{
    ec2Client: ec2.NewFromConfig(p.session),
    //...
}
```

*After:*
```go
// import "github.com/aws/aws-sdk-go-v2/service/ec2"

type instanceProvider struct {
    id               string
    ec2Client        EC2Clientv2 // Interface type
    extendedMetadata bool
}

// ...
// In production code:
instanceProvider := &instanceProvider{
    ec2Client: ec2.NewFromConfig(p.session),
    //...
}

// In test code:
mockClient := &MockEC2Clientv2{ ... }
instanceProvider := &instanceProvider{
    ec2Client: mockClient,
    //...
}
```
The same refactoring will be applied to `ecsProvider` in `pkg/providers/aws/ecs.go` to use `ec2iface.EC2API` and `ecsiface.ECSAPI`.

### Step 3.4: Write Unit Tests

With the mocking infrastructure in place, we can write comprehensive unit tests.

**Example test for `instanceProvider` (in `pkg/providers/aws/instances_test.go`):**

```go
func TestInstanceProvider_GetResources(t *testing.T) {
    mockClient := &MockEC2Clientv2{
        DescribeInstancesFunc: func(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error) {
            // Return a mocked response
            return &ec2.DescribeInstancesOutput{
                Reservations: []types.Reservation{
                    {
                        Instances: []types.Instance{
                            {
                                InstanceId:       aws.String("i-1234567890abcdef0"),
                                PublicIpAddress:  aws.String("1.2.3.4"),
                                PrivateIpAddress: aws.String("10.0.0.1"),
                            },
                        },
                    },
                },
            }, nil
        },
    }

    provider := &instanceProvider{ec2Client: mockClient, id: "test"}
    resources, err := provider.GetResources(context.Background())
    
    // Assertions
    assert.NoError(t, err)
    assert.Equal(t, 1, resources.Total())
    // ... more assertions
}
```

## 4. Unifying AWS SDK versions

The AWS provider currently uses a mix of AWS SDK for Go v1 (`ecs.go`) and v2 (`aws.go`). This is not ideal.
As part of this testing effort, we should consider migrating all AWS provider code to use the AWS SDK for Go v2 for consistency. This will simplify client management, configuration, and mocking.

If a full migration is out of scope for now, the mocking strategy must accommodate both SDK versions as described above.

## 5. Implementation Roadmap

1.  **Phase 1: EC2 and ECS**
    *   Create interfaces and mocks for EC2 (v2) and ECS (v1).
    *   Refactor `instanceProvider` and `ecsProvider`.
    *   Write unit tests for `instanceProvider` and `ecsProvider`.

2.  **Phase 2: Remaining Services**
    *   Apply the same pattern to other AWS services: S3, ELB, ALB, CloudFront, EKS, Lambda, Lightsail, Route53.

3.  **Phase 3 (Optional but Recommended): SDK v2 Migration**
    *   Migrate all remaining v1 SDK usages in the AWS provider to v2.
    *   Update mocks and tests accordingly.

This plan provides a clear path to improving the testability and reliability of the AWS provider.

## 6. Advanced Testing Strategies (Enhancements)

The plan above provides a solid foundation for unit testing. We can further enhance our testing strategy for better efficiency and coverage with the following approaches.

### 6.1. Automated Mock Generation with `gomock`

Instead of writing mock clients manually, we can use `go.uber.org/mock/gomock` to generate them automatically. This saves significant effort and provides a rich API for setting expectations.

**Plan:**
1.  **Install `gomock`:**
    ```bash
    go install go.uber.org/mock/mockgen@latest
    ```
2.  **Add `go:generate` directive:** Add a comment to the file containing the interface to instruct `mockgen` to create the mock.
    ```go
    //go:generate mockgen -destination=mock_ec2_test.go -package=aws_test . EC2Clientv2
    type EC2Clientv2 interface {
        DescribeInstances(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error)
    }
    ```
3.  **Generate mocks:** Run `go generate ./...` to create the mock files.
4.  **Use in tests:** The generated mocks provide a controller to set expectations.

    ```go
    func TestInstanceProvider_GetResources_WithGomock(t *testing.T) {
        ctrl := gomock.NewController(t)
        defer ctrl.Finish()

        mockClient := NewMockEC2Clientv2(ctrl)

        // Set expectation
        mockClient.EXPECT().
            DescribeInstances(gomock.Any(), gomock.Any()).
            Return(&ec2.DescribeInstancesOutput{
                // ... mocked response ...
            }, nil)

        provider := &instanceProvider{ec2Client: mockClient, id: "test"}
        // ... run test and assert ...
    }
    ```

### 6.2. Integration Testing with LocalStack

Unit tests are essential, but they don't verify the actual interaction with the AWS APIs. Integration tests using LocalStack can fill this gap. LocalStack runs a local mock of the AWS cloud environment in a Docker container.

**Plan:**
1.  **Setup LocalStack:** Add a `docker-compose.yml` file to the project root for running LocalStack.
2.  **Configure AWS SDK for LocalStack:** In integration tests, configure the AWS SDK to send requests to the LocalStack endpoint instead of the real AWS. This is typically done by creating a custom resolver.

    ```go
    customResolver := aws.EndpointResolverWithOptionsFunc(func(service, region string, options ...interface{}) (aws.Endpoint, error) {
		if service == ec2.ServiceID && region == "us-east-1" {
			return aws.Endpoint{
				URL: "http://localhost:4566",
				SigningRegion: "us-east-1",
			}, nil
		}
		// returning EndpointNotFoundError will allow the service client to fall back to it's default resolution
		return aws.Endpoint{}, &aws.EndpointNotFoundError{}
	})

    cfg, err := config.LoadDefaultConfig(context.TODO(), config.WithEndpointResolverWithOptions(customResolver))
    ```
3.  **Write Integration Tests:** Write tests that use the real provider code against the LocalStack-configured SDK. These tests would live in a separate build-tagged file (e.g., `instances_integration_test.go` with a `//go:build integration` tag).

This two-pronged approach (unit tests with generated mocks + integration tests with LocalStack) will provide very high confidence in the correctness of the AWS provider.
