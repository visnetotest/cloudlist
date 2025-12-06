package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/elasticache"
	"github.com/aws/aws-sdk-go-v2/service/elasticache/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// elasticacheProvider is an instance of the AWS ElastiCache provider
type elasticacheProvider struct {
	provider *Provider
}

// newElastiCacheProvider returns a new ElastiCache provider
func newElastiCacheProvider(p *Provider) (*elasticacheProvider, error) {
	return &elasticacheProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *elasticacheProvider) Name() string {
	return "elasticache"
}

// ID returns the unique identifier of the service provider
func (p *elasticacheProvider) ID() string {
	return "elasticache"
}

// Services returns the list of services offered by the provider
func (p *elasticacheProvider) Services() []string {
	return []string{"elasticache"}
}

// Resources returns the resources as a schema.Resources object
func (p *elasticacheProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get ElastiCache client
	elasticacheClient := elasticache.NewFromConfig(p.provider.session)

	// Get all cache clusters
	clusters, err := p.getCacheClusters(ctx, elasticacheClient)
	if err != nil {
		return nil, fmt.Errorf("could not get ElastiCache clusters: %w", err)
	}
	resources.Merge(clusters)

	// Get all replication groups
	replicationGroups, err := p.getReplicationGroups(ctx, elasticacheClient)
	if err != nil {
		return nil, fmt.Errorf("could not get ElastiCache replication groups: %w", err)
	}
	resources.Merge(replicationGroups)

	return resources, nil
}

// getCacheClusters retrieves all ElastiCache clusters
func (p *elasticacheProvider) getCacheClusters(ctx context.Context, client *elasticache.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &elasticache.DescribeCacheClustersInput{
		MaxRecords: aws.Int32(100),
	}

	paginator := elasticache.NewDescribeCacheClustersPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, cluster := range page.CacheClusters {
			// Extract cluster information
			resource := &schema.Resource{
				ID:       fmt.Sprintf("elasticache-cluster-%s", aws.ToString(cluster.CacheClusterId)),
				Public:   true,
				Provider: "aws",
				Service:  "elasticache",
				DNSName:  p.extractClusterEndpoint(cluster),
				Metadata: map[string]string{
					"cluster_id":                  aws.ToString(cluster.CacheClusterId),
					"cluster_name":                aws.ToString(cluster.CacheClusterId),
					"status":                      aws.ToString(cluster.Status),
					"engine":                      aws.ToString(cluster.Engine),
					"engine_version":              aws.ToString(cluster.EngineVersion),
					"node_type":                   aws.ToString(cluster.NodeType),
					"num_cache_nodes":             fmt.Sprintf("%d", aws.ToInt32(cluster.NumCacheNodes)),
					"preferred_availability_zone": aws.ToString(cluster.PreferredAvailabilityZone),
					"cache_node_type":             aws.ToString(cluster.CacheNodeType),
					"notification_topic_arn":      aws.ToString(cluster.NotificationTopicARN),
					"iam_role_arn":                aws.ToString(cluster.IAMRoleARN),
					"auto_minor_version_upgrade":  fmt.Sprintf("%v", aws.ToBool(cluster.AutoMinorVersionUpgrade)),
					"snapshot_retention_limit":    fmt.Sprintf("%d", aws.ToInt32(cluster.SnapshotRetentionLimit)),
					"encrypted":                   fmt.Sprintf("%v", aws.ToBool(cluster.Encrypted)),
					"at_rest_encryption":          fmt.Sprintf("%v", aws.ToBool(cluster.AtRestEncryption)),
					"transit_encryption":          fmt.Sprintf("%v", aws.ToBool(cluster.TransitEncryption)),
					"auth_token_enabled":          fmt.Sprintf("%v", aws.ToBool(cluster.AuthTokenEnabled)),
					"replication_group_id":        aws.ToString(cluster.ReplicationGroupId),
				},
			}

			// Add configuration endpoint information
			if cluster.ConfigurationEndpoint != nil {
				resource.Metadata["configuration_endpoint"] = aws.ToString(cluster.ConfigurationEndpoint.Address)
				resource.Metadata["configuration_port"] = fmt.Sprintf("%d", aws.ToInt32(cluster.ConfigurationEndpoint.Port))
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// getReplicationGroups retrieves all ElastiCache replication groups
func (p *elasticacheProvider) getReplicationGroups(ctx context.Context, client *elasticache.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &elasticache.DescribeReplicationGroupsInput{
		MaxRecords: aws.Int32(100),
	}

	paginator := elasticache.NewDescribeReplicationGroupsPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, group := range page.ReplicationGroups {
			// Extract replication group information
			resource := &schema.Resource{
				ID:       fmt.Sprintf("elasticache-replication-group-%s", aws.ToString(group.ReplicationGroupId)),
				Public:   true,
				Provider: "aws",
				Service:  "elasticache",
				DNSName:  aws.ToString(group.ReplicationGroupId),
				Metadata: map[string]string{
					"replication_group_id": aws.ToString(group.ReplicationGroupId),
					"description":          aws.ToString(group.Description),
					"status":               aws.ToString(group.Status),
					"automatic_failover":   fmt.Sprintf("%v", aws.ToBool(group.AutomaticFailover)),
					"node_type":            aws.ToString(group.NodeType),
					"num_cache_clusters":   fmt.Sprintf("%d", aws.ToInt32(group.NumCacheClusters)),
					"num_node_groups":      fmt.Sprintf("%d", aws.ToInt32(group.NumNodeGroups)),
				},
			}

			// Add primary endpoint information
			if group.PrimaryEndpoint != nil {
				resource.Metadata["primary_endpoint"] = aws.ToString(group.PrimaryEndpoint.Address)
				resource.Metadata["primary_port"] = fmt.Sprintf("%d", aws.ToInt32(group.PrimaryEndpoint.Port))
			}

			// Add node group information
			if len(group.NodeGroups) > 0 {
				var nodeGroups []string
				for _, nodeGroup := range group.NodeGroups {
					if nodeGroup.PrimaryEndpoint != nil {
						nodeGroups = append(nodeGroups, fmt.Sprintf("%s:%s", aws.ToString(nodeGroup.NodeGroupId), aws.ToString(nodeGroup.PrimaryEndpoint.Address)))
					}
				}
				resource.Metadata["node_groups"] = strings.Join(nodeGroups, ",")
			}

			// Add global replication configuration
			if group.GlobalReplicationConfiguration != nil {
				resource.Metadata["global_replication_group_id"] = aws.ToString(group.GlobalReplicationConfiguration.GlobalReplicationGroupIdentifier)
				resource.Metadata["global_replication_group_status"] = aws.ToString(group.GlobalReplicationConfiguration.Status)
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// extractClusterEndpoint extracts the endpoint from a cluster
func (p *elasticacheProvider) extractClusterEndpoint(cluster *types.CacheCluster) string {
	if cluster.ConfigurationEndpoint != nil {
		return aws.ToString(cluster.ConfigurationEndpoint.Address)
	}
	return ""
}
