package aws

import (
	"context"

	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// Provider is a provider for AWS.
type Provider struct {
	id      string
	session *config.SharedConfig
}

// New creates a new provider for AWS.
func New(options schema.OptionBlock) (schema.Provider, error) {
	id, _ := options.GetMetadata("id")

	// For the MVP, we'll use the default credential chain.
	// This will be expanded later to support more authentication methods.
	cfg, err := config.LoadDefaultConfig(context.TODO())
	if err != nil {
		return nil, err
	}

	return &Provider{id: id, session: &cfg}, nil
}

// Name returns the name of the provider.
func (p *Provider) Name() string {
	return "aws"
}

// ID returns the name of the provider account.
func (p *Provider) ID() string {
	return p.id
}

// Resources returns the provider's resources.
func (p *Provider) Resources(ctx context.Context) (*schema.Resources, error) {
	// For the MVP, we will start with EC2 instances.
	// This will be expanded to other services later.
	instanceProvider := &instanceProvider{session: p.session}
	return instanceProvider.GetResources(ctx)
}

// Services returns the list of services supported by the provider.
func (p *Provider) Services() []string {
	return []string{"ec2"}
}
