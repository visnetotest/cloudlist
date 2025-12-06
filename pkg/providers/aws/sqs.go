package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/sqs"
	"github.com/aws/aws-sdk-go-v2/service/sqs/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// sqsProvider is an instance of the AWS SQS provider
type sqsProvider struct {
	provider *awsProvider
}

// newSQSProvider returns a new SQS provider
func newSQSProvider(p *awsProvider) (*sqsProvider, error) {
	return &sqsProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *sqsProvider) Name() string {
	return "sqs"
}

// ID returns the unique identifier of the service provider
func (p *sqsProvider) ID() string {
	return "sqs"
}

// Services returns the list of services offered by the provider
func (p *sqsProvider) Services() []string {
	return []string{"sqs"}
}

// Resources returns the resources as a schema.Resources object
func (p *sqsProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get SQS client
	client, err := p.provider.getClient()
	if err != nil {
		return nil, fmt.Errorf("could not get SQS client: %w", err)
	}

	sqsClient := sqs.NewFromConfig(client)

	// Get all queues
	queues, err := p.getQueues(ctx, sqsClient)
	if err != nil {
		return nil, fmt.Errorf("could not get SQS queues: %w", err)
	}
	resources.Merge(queues)

	return resources, nil
}

// getQueues retrieves all SQS queues
func (p *sqsProvider) getQueues(ctx context.Context, client *sqs.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &sqs.ListQueuesInput{
		MaxResults: aws.Int32(1000),
	}

	paginator := sqs.NewListQueuesPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, queueURL := range page.QueueUrls {
			// Extract queue name from URL
			queueName := p.extractQueueName(queueURL)
			if queueName == "" {
				continue
			}

			// Get queue attributes
			attributes, err := p.getQueueAttributes(ctx, client, queueURL)
			if err != nil {
				continue
			}

			// Get queue ARN
			queueARN := p.getQueueARN(attributes)

			resource := &schema.Resource{
				ID:       fmt.Sprintf("sqs-%s", queueName),
				Public:   true,
				Provider: "aws",
				Service:  "sqs",
				DNSName:  queueName,
				Metadata: map[string]string{
					"queue_name":                                 queueName,
					"queue_url":                                  queueURL,
					"queue_arn":                                  queueARN,
					"region":                                     p.provider.region,
					"visibility_timeout":                         attributes["VisibilityTimeout"],
					"message_retention_period":                   attributes["MessageRetentionPeriod"],
					"maximum_message_size":                       attributes["MaximumMessageSize"],
					"delay_seconds":                              attributes["DelaySeconds"],
					"receive_message_wait_time_seconds":          attributes["ReceiveMessageWaitTimeSeconds"],
					"created_timestamp":                          attributes["CreatedTimestamp"],
					"last_modified_timestamp":                    attributes["LastModifiedTimestamp"],
					"approximate_number_of_messages":             attributes["ApproximateNumberOfMessages"],
					"approximate_number_of_messages_not_visible": attributes["ApproximateNumberOfMessagesNotVisible"],
					"approximate_number_of_messages_delayed":     attributes["ApproximateNumberOfMessagesDelayed"],
				},
			}

			// Add queue type information
			if queueType, ok := attributes["QueueArn"]; ok {
				if strings.Contains(queueType, ".fifo") {
					resource.Metadata["queue_type"] = "FIFO"
				} else {
					resource.Metadata["queue_type"] = "Standard"
				}
			}

			// Add encryption information
			if kmsKeyId, ok := attributes["KmsMasterKeyId"]; ok && kmsKeyId != "" {
				resource.Metadata["encryption"] = "KMS"
				resource.Metadata["kms_key_id"] = kmsKeyId
			} else {
				resource.Metadata["encryption"] = "Server-side"
			}

			// Add dead letter queue information
			if dlqARN, ok := attributes["DeadLetterTargetArn"]; ok && dlqARN != "" {
				resource.Metadata["dead_letter_queue_arn"] = dlqARN
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// extractQueueName extracts queue name from queue URL
func (p *sqsProvider) extractQueueName(queueURL string) string {
	// Queue URL format: https://sqs.{region}.amazonaws.com/{queue_name}
	parts := strings.Split(queueURL, "/")
	if len(parts) > 0 {
		return parts[len(parts)-1]
	}
	return ""
}

// getQueueAttributes retrieves attributes for a specific queue
func (p *sqsProvider) getQueueAttributes(ctx context.Context, client *sqs.Client, queueURL string) (map[string]string, error) {
	input := &sqs.GetQueueAttributesInput{
		QueueUrl: aws.String(queueURL),
		AttributeNames: []types.QueueAttributeName{
			types.QueueAttributeNameAll,
		},
	}

	result, err := client.GetQueueAttributes(ctx, input)
	if err != nil {
		return nil, err
	}

	attributes := make(map[string]string)
	for key, value := range result.Attributes {
		attributes[string(key)] = aws.ToString(value)
	}

	return attributes, nil
}

// getQueueARN extracts queue ARN from attributes
func (p *sqsProvider) getQueueARN(attributes map[string]string) string {
	if arn, ok := attributes["QueueArn"]; ok {
		return arn
	}
	return ""
}
