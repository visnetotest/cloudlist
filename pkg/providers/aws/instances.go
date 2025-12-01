package aws

import (
	"context"

	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/ec2"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// instanceProvider is a provider for AWS EC2 instances.
type instanceProvider struct {
	session *config.SharedConfig
}

// GetResources returns all the resources in the store for a provider.
func (p *instanceProvider) GetResources(ctx context.Context) (*schema.Resources, error) {
	list := schema.NewResources()

	client := ec2.NewFromConfig(*p.session)

	// TODO: Handle pagination
	result, err := client.DescribeInstances(ctx, &ec2.DescribeInstancesInput{})
	if err != nil {
		return nil, err
	}

	for _, reservation := range result.Reservations {
		for _, instance := range reservation.Instances {
			// Create a new resource and add it to the list
			resource := &schema.Resource{
				Provider:   "aws",
				ID:         *instance.InstanceId,
				PublicIPv4: *instance.PublicIpAddress,
				DNSName:    *instance.PublicDnsName,
			}
			list.Append(resource)
		}
	}

	return list, nil
}
