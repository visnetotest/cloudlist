package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/cloudfront"
	"github.com/aws/aws-sdk-go-v2/service/cloudfront/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// cloudfrontProvider is an instance of the AWS CloudFront provider
type cloudfrontProvider struct {
	provider *awsProvider
}

// newCloudFrontProvider returns a new CloudFront provider
func newCloudFrontProvider(p *awsProvider) (*cloudfrontProvider, error) {
	return &cloudfrontProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *cloudfrontProvider) Name() string {
	return "cloudfront"
}

// ID returns the unique identifier of the service provider
func (p *cloudfrontProvider) ID() string {
	return "cloudfront"
}

// Services returns the list of services offered by the provider
func (p *cloudfrontProvider) Services() []string {
	return []string{"cloudfront"}
}

// Resources returns the resources as a schema.Resources object
func (p *cloudfrontProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get CloudFront client
	client, err := p.provider.getClient()
	if err != nil {
		return nil, fmt.Errorf("could not get CloudFront client: %w", err)
	}

	cloudfrontClient := cloudfront.NewFromConfig(client)

	// Get all distributions
	distributions, err := p.getDistributions(ctx, cloudfrontClient)
	if err != nil {
		return nil, fmt.Errorf("could not get CloudFront distributions: %w", err)
	}
	resources.Merge(distributions)

	// Get all functions
	functions, err := p.getFunctions(ctx, cloudfrontClient)
	if err != nil {
		return nil, fmt.Errorf("could not get CloudFront functions: %w", err)
	}
	resources.Merge(functions)

	// Get all origin access identities
	originAccessIdentities, err := p.getOriginAccessIdentities(ctx, cloudfrontClient)
	if err != nil {
		return nil, fmt.Errorf("could not get CloudFront origin access identities: %w", err)
	}
	resources.Merge(originAccessIdentities)

	return resources, nil
}

// getDistributions retrieves all CloudFront distributions
func (p *cloudfrontProvider) getDistributions(ctx context.Context, client *cloudfront.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &cloudfront.ListDistributionsInput{
		MaxItems: aws.Int64(100),
	}

	paginator := cloudfront.NewListDistributionsPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, distribution := range page.DistributionList.Items {
			// Extract distribution information
			resource := &schema.Resource{
				ID:       fmt.Sprintf("cloudfront-distribution-%s", aws.ToString(distribution.Id)),
				Public:   true,
				Provider: "aws",
				Service:  "cloudfront",
				DNSName:  p.extractDomainName(distribution),
				Metadata: map[string]string{
					"distribution_id":    aws.ToString(distribution.Id),
					"domain_name":        p.extractDomainName(distribution),
					"status":             aws.ToString(distribution.Status),
					"enabled":            fmt.Sprintf("%v", aws.ToBool(distribution.Enabled)),
					"price_class":        aws.ToString(distribution.PriceClass),
					"web_acl_id":         aws.ToString(distribution.WebACLId),
					"http_version":       aws.ToString(distribution.HttpVersion),
					"is_ipv6_enabled":    fmt.Sprintf("%v", aws.ToBool(distribution.IsIPV6Enabled)),
					"last_modified_time": aws.ToString(distribution.LastModifiedTime),
					"comment":            aws.ToString(distribution.Comment),
					"aliases":            strings.Join(distribution.Aliases.Items, ","),
				},
			}

			// Add default cache behavior information
			if len(distribution.DefaultCacheBehavior) > 0 {
				defaultBehavior := distribution.DefaultCacheBehavior[0]
				resource.Metadata["default_cache_behavior"] = fmt.Sprintf("PathPattern=%s, TargetOriginId=%s, ViewerProtocolPolicy=%s, MinTTL=%d, MaxTTL=%d, Compress=%v, ForwardedValues=%v",
					aws.ToString(defaultBehavior.PathPattern),
					aws.ToString(defaultBehavior.TargetOriginId),
					aws.ToString(defaultBehavior.ViewerProtocolPolicy),
					aws.ToInt32(defaultBehavior.MinTTL),
					aws.ToInt32(defaultBehavior.MaxTTL),
					aws.ToBool(defaultBehavior.Compress),
					strings.Join(defaultBehavior.ForwardedValues.Items, ","),
				)
			}

			// Add logging information
			if distribution.Logging != nil {
				resource.Metadata["logging_enabled"] = fmt.Sprintf("%v", aws.ToBool(distribution.Logging.Enabled))
				resource.Metadata["log_bucket"] = aws.ToString(distribution.Logging.Bucket)
				resource.Metadata["log_prefix"] = aws.ToString(distribution.Logging.Prefix)
				resource.Metadata["log_include_cookies"] = aws.ToString(distribution.Logging.IncludeCookies)
			}

			// Add viewer certificate information
			if distribution.ViewerCertificate != nil {
				resource.Metadata["certificate_type"] = aws.ToString(distribution.ViewerCertificate.CertificateType)
				resource.Metadata["certificate_source"] = aws.ToString(distribution.ViewerCertificate.CertificateSource)
				resource.Metadata["certificate_protocol"] = aws.ToString(distribution.ViewerCertificate.CertificateProtocol)
				resource.Metadata["ssl_protocol"] = aws.ToString(distribution.ViewerCertificate.SSLSupportMethod)
				resource.Metadata["minimum_protocol_version"] = aws.ToString(distribution.ViewerCertificate.MinimumProtocolVersion)
			}

			// Add origin information
			if len(distribution.Origins.Items) > 0 {
				var origins []string
				for _, origin := range distribution.Origins.Items {
					if origin.DomainName != nil {
						origins = append(origins, fmt.Sprintf("%s:%s", aws.ToString(origin.DomainName), aws.ToString(origin.Id)))
					}
				}
				resource.Metadata["origins"] = strings.Join(origins, ",")
			}

			// Add cache behaviors
			if len(distribution.CacheBehaviors.Items) > 0 {
				var behaviors []string
				for _, behavior := range distribution.CacheBehaviors.Items {
					behaviors = append(behaviors, fmt.Sprintf("PathPattern=%s, TargetOriginId=%s", aws.ToString(behavior.PathPattern), aws.ToString(behavior.TargetOriginId)))
				}
				resource.Metadata["cache_behaviors"] = strings.Join(behaviors, ",")
			}

			// Add custom error responses
			if distribution.CustomErrorResponses != nil {
				var errorResponses []string
				for _, errorResponse := range distribution.CustomErrorResponses.Items {
					errorResponses = append(errorResponses, fmt.Sprintf("ErrorCode=%s, ResponsePagePath=%s, ResponseCode=%s, ErrorCachingMinTTL=%d",
						aws.ToInt32(errorResponse.ErrorCode),
						aws.ToString(errorResponse.ResponsePagePath),
						aws.ToInt32(errorResponse.ResponseCode),
						aws.ToInt32(errorResponse.ErrorCachingMinTTL)))
				}
				resource.Metadata["custom_error_responses"] = strings.Join(errorResponses, ",")
			}

			// Add restrictions
			if distribution.Restrictions != nil {
				var geoRestrictions []string
				if distribution.Restrictions.GeoRestriction != nil {
					for _, restriction := range distribution.Restrictions.GeoRestriction.Restrictions {
						geoRestrictions = append(geoRestrictions, fmt.Sprintf("%s:%s", aws.ToString(restriction.RestrictionType), strings.Join(restriction.Restriction.Items.Items, ",")))
					}
				}
				resource.Metadata["geo_restrictions"] = strings.Join(geoRestrictions, ",")
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getFunctions retrieves all CloudFront functions
func (p *cloudfrontProvider) getFunctions(ctx context.Context, client *cloudfront.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &cloudfront.ListFunctions2020_09_07Input{
		MaxItems: aws.Int64(100),
	}

	paginator := cloudfront.NewListFunctions2020_09_07Paginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, function := range page.FunctionList.Items {
			// Extract function information
			resource := &schema.Resource{
				ID:       fmt.Sprintf("cloudfront-function-%s", aws.ToString(function.Name)),
				Public:   true,
				Provider: "aws",
				Service:  "cloudfront",
				DNSName:  aws.ToString(function.Name),
				Metadata: map[string]string{
					"function_name":      aws.ToString(function.Name),
					"function_arn":       aws.ToString(function.FunctionARN),
					"status":             aws.ToString(function.Status),
					"runtime":            aws.ToString(function.Runtime),
					"memory_size":        fmt.Sprintf("%d", aws.ToInt32(function.MemorySize)),
					"code_size":          fmt.Sprintf("%d", aws.ToInt32(function.CodeSize)),
					"comment":            aws.ToString(function.Comment),
					"last_modified_time": aws.ToString(function.LastModifiedTime),
				},
			}

			// Add environment variables
			if len(function.Environment.Variables) > 0 {
				var envVars []string
				for _, envVar := range function.Environment.Variables {
					envVars = append(envVars, fmt.Sprintf("%s=%s", aws.ToString(envVar.Key), aws.ToString(envVar.Value)))
				}
				resource.Metadata["environment_variables"] = strings.Join(envVars, ",")
			}

			// Add function associations
			if len(function.FunctionAssociations.Items) > 0 {
				var associations []string
				for _, association := range function.FunctionAssociations.Items {
					associations = append(associations, fmt.Sprintf("Event=%s, FunctionARN=%s",
						aws.ToString(association.EventType),
						aws.ToString(association.FunctionARN)))
				}
				resource.Metadata["function_associations"] = strings.Join(associations, ",")
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getOriginAccessIdentities retrieves all CloudFront origin access identities
func (p *cloudfrontProvider) getOriginAccessIdentities(ctx context.Context, client *cloudfront.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &cloudfront.ListOriginAccessIdentitiesInput{
		MaxItems: aws.Int64(100),
	}

	paginator := cloudfront.NewListOriginAccessIdentitiesPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, identity := range page.OriginAccessIdentityList.Items {
			// Extract identity information
			resource := &schema.Resource{
				ID:       fmt.Sprintf("cloudfront-origin-access-identity-%s", aws.ToString(identity.Id)),
				Public:   true,
				Provider: "aws",
				Service:  "cloudfront",
				DNSName:  aws.ToString(identity.Id),
				Metadata: map[string]string{
					"identity_id":      aws.ToString(identity.Id),
					"description":      aws.ToString(identity.Description),
					"iam_arn":          aws.ToString(identity.IamArn),
					"caller_reference": aws.ToString(identity.CallerReference),
					"paths":            strings.Join(identity.Paths.Items, ","),
				},
			}

			// Add signers information
			if len(identity.Signers.Items) > 0 {
				var signers []string
				for _, signer := range identity.Signers.Items {
					signers = append(signers, fmt.Sprintf("%s:%s", aws.ToString(signer.AwsAccountNumber), aws.ToString(signer.SignerName)))
				}
				resource.Metadata["signers"] = strings.Join(signers, ",")
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// extractDomainName extracts the domain name from a CloudFront distribution
func (p *cloudfrontProvider) extractDomainName(distribution *types.Distribution) string {
	if distribution.DomainName != nil {
		return aws.ToString(distribution.DomainName)
	}

	// Try to extract domain from aliases
	if len(distribution.Aliases.Items) > 0 {
		for _, alias := range distribution.Aliases.Items {
			if alias.Alias != nil {
				return aws.ToString(alias.Alias)
			}
		}
	}

	return ""
}
