package aws

import (
	"context"
	"fmt"
	"regexp"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/ec2"
	"github.com/aws/aws-sdk-go-v2/service/ec2/types"
	goec2 "github.com/aws/aws-sdk-go/service/ec2"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// Services is the list of services supported by the AWS provider
var Services = []string{"ec2", "elb", "alb", "s3"}

// ProviderOptions contains options for the AWS provider
type ProviderOptions struct {
	AssumeRoleName   string
	AccountIds       []string
	ExtendedMetadata bool
}

// ARNComponents holds the parsed components of an AWS ARN
type ARNComponents struct {
	Partition    string
	Service      string
	Region       string
	AccountID    string
	ResourceType string
	ResourceID   string
}

// GetResourceName returns the resource name from the ARN
func (a *ARNComponents) GetResourceName() string {
	return a.ResourceID
}

// Provider is a provider for AWS.
type Provider struct {
	id      string
	session aws.Config
	options *ProviderOptions
}

// New creates a new provider for AWS.
func New(options schema.OptionBlock) (schema.Provider, error) {
	id, _ := options.GetMetadata("id")

	// For the MVP, we'''ll use the default credential chain.
	// This will be expanded later to support more authentication methods.
	cfg, err := config.LoadDefaultConfig(context.TODO())
	if err != nil {
		return nil, err
	}

	return &Provider{id: id, session: cfg}, nil
}

// Name returns the name of the provider.
func (p *Provider) Name() string {
	return "aws"
}

// ID returns the name of the provider account.
func (p *Provider) ID() string {
	return p.id
}

// Resources returns the provider'''s resources.
func (p *Provider) Resources(ctx context.Context) (*schema.Resources, error) {
	// For the MVP, we will start with EC2 instances.
	// This will be expanded to other services later.
	instanceProvider := &instanceProvider{ec2Client: ec2.NewFromConfig(p.session)}
	return instanceProvider.GetResources(ctx)
}

// Services returns the list of services supported by the provider.
func (p *Provider) Services() []string {
	return Services
}

// parseARN parses a given AWS ARN and returns its components.
// An ARN is expected to be in the format: arn:partition:service:region:account-id:resource-id
func parseARN(arn string) *ARNComponents {
	// Define the regex for parsing ARNs
	// Example: arn:aws:ec2:us-east-1:123456789012:instance/i-1234567890abcdef0
	re := regexp.MustCompile(`^arn:(?P<Partition>[^:]+):(?P<Service>[^:]+):(?P<Region>[^:]*):(?P<AccountID>[^:]*):(?P<ResourceType>[^/:]+)(?:/|:)(?P<ResourceID>.*)$`)

	if !re.MatchString(arn) {
		return nil
	}

	matches := re.FindStringSubmatch(arn)
	subexpNames := re.SubexpNames()

	components := &ARNComponents{}
	for i, name := range subexpNames {
		if i > 0 && name != "" {
			switch name {
			case "Partition":
				components.Partition = matches[i]
			case "Service":
				components.Service = matches[i]
			case "Region":
				components.Region = matches[i]
			case "AccountID":
				components.AccountID = matches[i]
			case "ResourceType":
				components.ResourceType = matches[i]
			case "ResourceID":
				components.ResourceID = matches[i]
			}
		}
	}
	return components
}

func buildTagString(tags []types.Tag) string {
	var result []string
	for _, tag := range tags {
		result = append(result, fmt.Sprintf("%s=%s", aws.ToString(tag.Key), aws.ToString(tag.Value)))
	}
	return strings.Join(result, ",")
}

func buildAwsMapTagString(tags map[string]*string) string {
	var tagPairs []string
	for key, value := range tags {
		tagPairs = append(tagPairs, fmt.Sprintf("%s=%s", key, aws.ToString(value)))
	}
	return strings.Join(tagPairs, ",")
}

func convertEC2Tags(tags []*goec2.Tag) []types.Tag {
	var result []types.Tag
	for _, tag := range tags {
		result = append(result, types.Tag{
			Key:   tag.Key,
			Value: tag.Value,
		})
	}
	return result
}
