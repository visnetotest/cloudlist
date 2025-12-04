package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/service/ec2"
	"github.com/aws/aws-sdk-go-v2/service/ec2/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

//go:generate mockgen -destination=mock_ec2_test.go -package=aws . EC2Client
type EC2Client interface {
	DescribeInstances(ctx context.Context, params *ec2.DescribeInstancesInput, optFns ...func(*ec2.Options)) (*ec2.DescribeInstancesOutput, error)
}

// instanceProvider is a provider for AWS EC2 instances.
type instanceProvider struct {
	id               string
	ec2Client        EC2Client
	extendedMetadata bool
}

// GetResources returns all the resources in the store for a provider.
func (p *instanceProvider) GetResources(ctx context.Context) (*schema.Resources, error) {
	list := schema.NewResources()

	paginator := ec2.NewDescribeInstancesPaginator(p.ec2Client, &ec2.DescribeInstancesInput{})
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, reservation := range page.Reservations {
			for _, instance := range reservation.Instances {
				// Ignore terminated instances
				if instance.State != nil && instance.State.Name == types.InstanceStateNameTerminated {
					continue
				}

				publicIPv4, _, privateIPv4, _ := getNetworkAddresses(instance.NetworkInterfaces)

				resource := &schema.Resource{
					Provider:    ProviderName,
					ID:          *instance.InstanceId,
					PublicIPv4:  publicIPv4,
					PrivateIpv4: privateIPv4,
					DNSName:     getPublicDnsName(instance),
					Service:     "ec2",
				}

				if p.extendedMetadata {
					resource.Metadata = p.getExtendedMetadata(instance)
				}
				list.Append(resource)
			}
		}
	}
	return list, nil
}

func (p *instanceProvider) getExtendedMetadata(instance types.Instance) map[string]string {
	metadata := make(map[string]string)
	metadata["instance_type"] = string(instance.InstanceType)
	if instance.Placement != nil && instance.Placement.AvailabilityZone != nil {
		// Region is the AZ without the last character
		az := *instance.Placement.AvailabilityZone
		if len(az) > 1 {
			metadata["region"] = az[:len(az)-1]
		}
	}
	if instance.IamInstanceProfile != nil && instance.IamInstanceProfile.Arn != nil {
		metadata["iam_instance_profile"] = *instance.IamInstanceProfile.Arn
	}
	if len(instance.Tags) > 0 {
		var tagBuilder strings.Builder
		for i, tag := range instance.Tags {
			if i > 0 {
				tagBuilder.WriteString(",")
			}
			tagBuilder.WriteString(fmt.Sprintf("%s=%s", *tag.Key, *tag.Value))
		}
		metadata["tags"] = tagBuilder.String()
	}
	return metadata
}

func getNetworkAddresses(interfaces []types.InstanceNetworkInterface) (publicIPv4 string, publicIPv6 string, privateIPv4 string, privateIPv6 string) {
	for _, iface := range interfaces {
		if iface.Association != nil && iface.Association.PublicIp != nil {
			publicIPv4 = *iface.Association.PublicIp
		}
		if len(iface.Ipv6Addresses) > 0 && iface.Ipv6Addresses[0].Ipv6Address != nil {
			publicIPv6 = *iface.Ipv6Addresses[0].Ipv6Address
		}
		if len(iface.PrivateIpAddresses) > 0 && iface.PrivateIpAddresses[0].PrivateIpAddress != nil {
			privateIPv4 = *iface.PrivateIpAddresses[0].PrivateIpAddress
		}
		// Similarly for private IPv6 if needed
	}
	return
}

func getPublicDnsName(instance types.Instance) string {
	if instance.PublicDnsName != nil && *instance.PublicDnsName != "" {
		return *instance.PublicDnsName
	}
	return ""
}
