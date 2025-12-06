package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/sns"
	"github.com/aws/aws-sdk-go-v2/service/sns/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// snsProvider is an instance of the AWS SNS provider
type snsProvider struct {
	provider *awsProvider
}

// newSNSProvider returns a new SNS provider
func newSNSProvider(p *awsProvider) (*snsProvider, error) {
	return &snsProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *snsProvider) Name() string {
	return "sns"
}

// ID returns the unique identifier of the service provider
func (p *snsProvider) ID() string {
	return "sns"
}

// Services returns the list of services offered by the provider
func (p *snsProvider) Services() []string {
	return []string{"sns"}
}

// Resources returns the resources as a schema.Resources object
func (p *snsProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get SNS client
	client, err := p.provider.getClient()
	if err != nil {
		return nil, fmt.Errorf("could not get SNS client: %w", err)
	}

	snsClient := sns.NewFromConfig(client)

	// Get all topics
	topics, err := p.getTopics(ctx, snsClient)
	if err != nil {
		return nil, fmt.Errorf("could not get SNS topics: %w", err)
	}
	resources.Merge(topics)

	return resources, nil
}

// getTopics retrieves all SNS topics
func (p *snsProvider) getTopics(ctx context.Context, client *sns.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &sns.ListTopicsInput{
		MaxResults: aws.Int32(1000),
	}

	paginator := sns.NewListTopicsPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, topic := range page.Topics {
			// Get topic attributes
			attributes, err := p.getTopicAttributes(ctx, client, *topic.TopicArn)
			if err != nil {
				continue
			}

			// Get topic subscriptions
			subscriptions, err := p.getTopicSubscriptions(ctx, client, *topic.TopicArn)
			if err != nil {
				continue
			}

			// Extract topic name from ARN
			topicName := p.extractTopicName(*topic.TopicArn)

			resource := &schema.Resource{
				ID:       fmt.Sprintf("sns-%s", topicName),
				Public:   true,
				Provider: "aws",
				Service:  "sns",
				DNSName:  topicName,
				Metadata: map[string]string{
					"topic_name":         topicName,
					"topic_arn":          *topic.TopicArn,
					"region":             p.provider.region,
					"display_name":       aws.ToString(topic.DisplayName),
					"type":               aws.ToString(topic.Type),
					"owner":              aws.ToString(topic.Owner),
					"created_date":       aws.ToString(topic.CreatedTimestamp),
					"subscription_count": fmt.Sprintf("%d", len(subscriptions)),
				},
			}

			// Add delivery policy information
			if deliveryPolicy, ok := attributes["DeliveryPolicy"]; ok && deliveryPolicy != "" {
				resource.Metadata["delivery_policy"] = deliveryPolicy
			}

			// Add KMS encryption information
			if kmsMasterKeyId, ok := attributes["KmsMasterKeyId"]; ok && kmsMasterKeyId != "" {
				resource.Metadata["encryption"] = "KMS"
				resource.Metadata["kms_master_key_id"] = kmsMasterKeyId
			} else {
				resource.Metadata["encryption"] = "Server-side"
			}

			// Add effective delivery policy
			if effectiveDeliveryPolicy, ok := attributes["EffectiveDeliveryPolicy"]; ok {
				resource.Metadata["effective_delivery_policy"] = effectiveDeliveryPolicy
			}

			// Add subscription information
			if len(subscriptions) > 0 {
				var subscriptionTypes []string
				var subscriptionEndpoints []string
				for _, sub := range subscriptions {
					subscriptionTypes = append(subscriptionTypes, string(sub.Protocol))
					subscriptionEndpoints = append(subscriptionEndpoints, *sub.Endpoint)
				}
				resource.Metadata["subscription_types"] = strings.Join(subscriptionTypes, ",")
				resource.Metadata["subscription_endpoints"] = strings.Join(subscriptionEndpoints, ",")
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getTopicAttributes retrieves attributes for a specific topic
func (p *snsProvider) getTopicAttributes(ctx context.Context, client *sns.Client, topicARN string) (map[string]string, error) {
	input := &sns.GetTopicAttributesInput{
		TopicArn: aws.String(topicARN),
	}

	result, err := client.GetTopicAttributes(ctx, input)
	if err != nil {
		return nil, err
	}

	attributes := make(map[string]string)
	for key, value := range result.Attributes {
		attributes[string(key)] = aws.ToString(value)
	}

	return attributes, nil
}

// getTopicSubscriptions retrieves subscriptions for a specific topic
func (p *snsProvider) getTopicSubscriptions(ctx context.Context, client *sns.Client, topicARN string) ([]types.Subscription, error) {
	input := &sns.ListSubscriptionsByTopicInput{
		TopicArn:   aws.String(topicARN),
		MaxResults: aws.Int32(1000),
	}

	paginator := sns.NewListSubscriptionsByTopicPaginator(client, input)
	var subscriptions []types.Subscription
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}
		subscriptions = append(subscriptions, page.Subscriptions...)
	}

	return subscriptions, nil
}

// extractTopicName extracts topic name from topic ARN
func (p *snsProvider) extractTopicName(topicARN string) string {
	// Topic ARN format: arn:aws:sns:{region}:{account-id}:{topic-name}
	parts := strings.Split(topicARN, ":")
	if len(parts) >= 6 {
		return parts[5]
	}
	return ""
}
