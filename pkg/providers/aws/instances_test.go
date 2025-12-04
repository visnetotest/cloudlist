package aws

import (
	"context"
	"testing"

	"github.com/aws/aws-sdk-go-v2/service/ec2"
	"github.com/aws/aws-sdk-go-v2/service/ec2/types"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestInstanceProvider_GetResources_WithGomock(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mockClient := NewMockEC2Client(ctrl)

	const ( // Define constants for test data
		expectedDNS   = "ec2-1-2-3-4.compute-1.amazonaws.com"
		expectedIP    = "1.2.3.4"
		expectedID    = "i-1234567890abcdef0"
		providerID    = "test-id"
	)

	mockResp := &ec2.DescribeInstancesOutput{
		Reservations: []types.Reservation{
			{
				Instances: []types.Instance{
					{
						InstanceId:    aws.String(expectedID),
						PublicDnsName: aws.String(expectedDNS),
						NetworkInterfaces: []types.InstanceNetworkInterface{
							{
								Association: &types.InstanceNetworkInterfaceAssociation{
									PublicIp: aws.String(expectedIP),
								},
							},
						},
					},
				},
			},
		},
	}

	mockClient.EXPECT().
		DescribeInstances(gomock.Any(), gomock.Any(), gomock.Any()).
		Return(mockResp, nil)

	provider := &instanceProvider{
		ec2Client: mockClient,
		id:        providerID,
	}

	resources, err := provider.GetResources(context.Background())

	assert.NoError(t, err, "unexpected error")
	assert.NotNil(t, resources, "resources should not be nil")

	// The schema automatically splits a resource with both DNS and IP into two.
	assert.Equal(t, 2, len(resources.Items), "expected two resources due to splitting")

	var dnsFound, ipFound bool
	for _, item := range resources.Items {
		assert.Equal(t, expectedID, item.ID, "unexpected instance ID")
		if item.DNSName == expectedDNS {
			dnsFound = true
		}
		if item.PublicIPv4 == expectedIP {
			ipFound = true
		}
	}

	assert.True(t, dnsFound, "DNS resource was not found")
	assert.True(t, ipFound, "IP resource was not found")
}
