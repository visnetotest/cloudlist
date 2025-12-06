package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/apigateway"
	"github.com/aws/aws-sdk-go-v2/service/apigateway/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// apigatewayProvider is an instance of the AWS API Gateway provider
type apigatewayProvider struct {
	provider *awsProvider
}

// newAPIGatewayProvider returns a new API Gateway provider
func newAPIGatewayProvider(p *awsProvider) (*apigatewayProvider, error) {
	return &apigatewayProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *apigatewayProvider) Name() string {
	return "apigateway"
}

// ID returns the unique identifier of the service provider
func (p *apigatewayProvider) ID() string {
	return "apigateway"
}

// Services returns the list of services offered by the provider
func (p *apigatewayProvider) Services() []string {
	return []string{"apigateway"}
}

// Resources returns the resources as a schema.Resources object
func (p *apigatewayProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get API Gateway client
	client, err := p.provider.getClient()
	if err != nil {
		return nil, fmt.Errorf("could not get API Gateway client: %w", err)
	}

	apigatewayClient := apigateway.NewFromConfig(client)

	// Get all REST APIs
	restAPIs, err := p.getRESTAPIs(ctx, apigatewayClient)
	if err != nil {
		return nil, fmt.Errorf("could not get REST APIs: %w", err)
	}
	resources.Merge(restAPIs)

	// Get all WebSocket APIs
	websocketAPIs, err := p.getWebSocketAPIs(ctx, apigatewayClient)
	if err != nil {
		return nil, fmt.Errorf("could not get WebSocket APIs: %w", err)
	}
	resources.Merge(websocketAPIs)

	// Get all HTTP APIs
	httpAPIs, err := p.getHTTPAPIs(ctx, apigatewayClient)
	if err != nil {
		return nil, fmt.Errorf("could not get HTTP APIs: %w", err)
	}
	resources.Merge(httpAPIs)

	return resources, nil
}

// getRESTAPIs retrieves all REST APIs
func (p *apigatewayProvider) getRESTAPIs(ctx context.Context, client *apigateway.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &apigateway.GetRestApisInput{
		Limit: aws.Int32(500),
	}

	result, err := client.GetRestApis(ctx, input)
	if err != nil {
		return nil, err
	}

	for _, api := range result.Items {
		// Get API stages to find deployment URLs
		stages, err := p.getAPIStages(ctx, client, *api.Id)
		if err != nil {
			continue
		}

		for _, stage := range stages {
			// Create resource for each API stage
			resource := &schema.Resource{
				ID:       fmt.Sprintf("apigateway-rest-%s-%s", *api.Id, *stage.StageName),
				Public:   true,
				Provider: "aws",
				Service:  "apigateway",
				DNSName:  fmt.Sprintf("%s.execute-api.%s.amazonaws.com", *api.Id, p.provider.region),
				Metadata: map[string]string{
					"api_id":       *api.Id,
					"api_name":     *api.Name,
					"api_type":     "REST",
					"stage_name":   *stage.StageName,
					"description":  aws.ToString(api.Description),
					"created_date": aws.ToString(api.CreatedDate),
					"version":      aws.ToString(stage.DeploymentId),
				},
			}

			// Add stage-specific metadata
			if stage.MethodSettings != nil {
				resource.Metadata["logging_level"] = aws.ToString(stage.LoggingLevel)
				resource.Metadata["data_trace_enabled"] = fmt.Sprintf("%v", aws.ToBool(stage.DataTraceEnabled))
				resource.Metadata["metrics_enabled"] = fmt.Sprintf("%v", aws.ToBool(stage.MetricsEnabled))
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getWebSocketAPIs retrieves all WebSocket APIs
func (p *apigatewayProvider) getWebSocketAPIs(ctx context.Context, client *apigateway.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &apigateway.GetApisInput{
		Limit: aws.Int32(500),
	}

	result, err := client.GetApis(ctx, input)
	if err != nil {
		return nil, err
	}

	for _, api := range result.Items {
		// Get API stages
		stages, err := p.getWebSocketAPIStages(ctx, client, *api.ApiId)
		if err != nil {
			continue
		}

		for _, stage := range stages {
			resource := &schema.Resource{
				ID:       fmt.Sprintf("apigateway-websocket-%s-%s", *api.ApiId, *stage.StageName),
				Public:   true,
				Provider: "aws",
				Service:  "apigateway",
				DNSName:  fmt.Sprintf("%s.execute-api.%s.amazonaws.com", *api.ApiId, p.provider.region),
				Metadata: map[string]string{
					"api_id":       *api.ApiId,
					"api_name":     *api.Name,
					"api_type":     "WebSocket",
					"stage_name":   *stage.StageName,
					"description":  aws.ToString(api.Description),
					"created_date": aws.ToString(api.CreatedDate),
					"version":      aws.ToString(stage.DeploymentId),
				},
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getHTTPAPIs retrieves all HTTP APIs
func (p *apigatewayProvider) getHTTPAPIs(ctx context.Context, client *apigateway.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &apigateway.GetApisInput{
		Limit: aws.Int32(500),
	}

	result, err := client.GetApis(ctx, input)
	if err != nil {
		return nil, err
	}

	for _, api := range result.Items {
		// Filter for HTTP APIs (WebSocket APIs have a protocol selection expression)
		if api.ProtocolType == "WEBSOCKET" {
			continue
		}

		// Get API stages
		stages, err := p.getHTTPAPIStages(ctx, client, *api.ApiId)
		if err != nil {
			continue
		}

		for _, stage := range stages {
			resource := &schema.Resource{
				ID:       fmt.Sprintf("apigateway-http-%s-%s", *api.ApiId, *stage.StageName),
				Public:   true,
				Provider: "aws",
				Service:  "apigateway",
				DNSName:  fmt.Sprintf("%s.execute-api.%s.amazonaws.com", *api.ApiId, p.provider.region),
				Metadata: map[string]string{
					"api_id":       *api.ApiId,
					"api_name":     *api.Name,
					"api_type":     "HTTP",
					"stage_name":   *stage.StageName,
					"description":  aws.ToString(api.Description),
					"created_date": aws.ToString(api.CreatedDate),
					"version":      aws.ToString(stage.DeploymentId),
				},
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getAPIStages retrieves stages for a REST API
func (p *apigatewayProvider) getAPIStages(ctx context.Context, client *apigateway.Client, apiID string) ([]types.Stage, error) {
	input := &apigateway.GetStagesInput{
		RestApiId: aws.String(apiID),
	}

	result, err := client.GetStages(ctx, input)
	if err != nil {
		return nil, err
	}

	return result.Item, nil
}

// getWebSocketAPIStages retrieves stages for a WebSocket API
func (p *apigatewayProvider) getWebSocketAPIStages(ctx context.Context, client *apigateway.Client, apiID string) ([]types.Stage, error) {
	input := &apigateway.GetStagesInput{
		RestApiId: aws.String(apiID),
	}

	result, err := client.GetStages(ctx, input)
	if err != nil {
		return nil, err
	}

	return result.Item, nil
}

// getHTTPAPIStages retrieves stages for an HTTP API
func (p *apigatewayProvider) getHTTPAPIStages(ctx context.Context, client *apigateway.Client, apiID string) ([]types.Stage, error) {
	input := &apigateway.GetStagesInput{
		RestApiId: aws.String(apiID),
	}

	result, err := client.GetStages(ctx, input)
	if err != nil {
		return nil, err
	}

	return result.Item, nil
}
