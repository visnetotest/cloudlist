package aws

import (
	"context"

	"github.com/aws/aws-sdk-go-v2/service/ec2"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// instanceProvider is a provider for AWS EC2 instances.
type instanceProvider struct {
	id      string
	ec2Client *ec2.Client
}

// GetResources returns all the resources in the store for a provider.
func (p *instanceProvider) GetResources(ctx context.Context) (*schema.Resources, error) {
	list := schema.NewResources()

	// TODO: Handle pagination
	result, err := p.ec2Client.DescribeInstances(ctx, &ec2.DescribeInstancesInput{})
	if err != nil {
		return nil, err
	}

	for _, reservation := range result.Reservations {
		for _, instance := range reservation.Instances {
			var publicIPv4, publicIPv6, privateIPv4, privateIPv6 string
			for _, netInterface := range instance.NetworkInterfaces {
				if netInterface.Association != nil {
					publicIPv4 = *netInterface.Association.PublicIp
				}
				for _, ipv6 := range netInterface.Ipv6Addresses {
					publicIPv6 = *ipv6.Ipv6Address
				}
				for _, privateIP := range netInterface.PrivateIpAddresses {
					privateIPv4 = *privateIP.PrivateIpAddress
				}
			}

			// Create a new resource and add it to the list
			resource := &schema.Resource{
				Provider:    "aws",
				ID:          p.id,
				PublicIPv4:  publicIPv4,
				PublicIPv6:  publicIPv6,
				PrivateIpv4: privateIPv4,
				PrivateIpv6: privateIPv6,
				DNSName:     *instance.PublicDnsName,
			}
			list.Append(resource)
		}
	}

	return list, nil
}
