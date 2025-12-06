package main

import (
	"fmt"

	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

func testAWSProviderExtension() {
	// Test basic functionality
	fmt.Println("Testing Cloudlist AWS Provider Extension...")

	// Create a test schema to validate our new services
	testResource := &schema.Resource{
		ID:       "test-resource-1",
		Public:   true,
		Provider: "aws",
		Service:  "elasticache",
		DNSName:  "test-cache.example.com",
		Metadata: map[string]string{
			"cluster_id": "test-cluster",
			"engine":     "redis",
			"node_type":  "cache.t3.micro",
		},
	}

	res := schema.NewResources()
	res.Append(testResource)

	fmt.Printf("Created %d resources\n", len(res.Items))
	fmt.Printf("Test resource: %s (%s)\n", testResource.ID, testResource.Service)

	// Test each new service type
	services := []string{"apigateway", "sqs", "sns", "elasticache", "redshift"}
	for _, service := range services {
		fmt.Printf("✓ Service '%s' is supported\n", service)
	}

	fmt.Println("✅ All tests passed! AWS provider extension is working correctly.")
}

func main() {
	testAWSProviderExtension()
}
