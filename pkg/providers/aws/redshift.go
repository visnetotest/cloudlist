package aws

import (
	"context"
	"fmt"
	"strings"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/redshift"
	"github.com/aws/aws-sdk-go-v2/service/redshift/types"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// redshiftProvider is an instance of the AWS Redshift provider
type redshiftProvider struct {
	provider *Provider
}

// newRedshiftProvider returns a new Redshift provider
func newRedshiftProvider(p *Provider) (*redshiftProvider, error) {
	return &redshiftProvider{provider: p}, nil
}

// Name returns the name of the service provider
func (p *redshiftProvider) Name() string {
	return "redshift"
}

// ID returns the unique identifier of the service provider
func (p *redshiftProvider) ID() string {
	return "redshift"
}

// Services returns the list of services offered by the provider
func (p *redshiftProvider) Services() []string {
	return []string{"redshift"}
}

// Resources returns the resources as a schema.Resources object
func (p *redshiftProvider) Resources(ctx context.Context) (*schema.Resources, error) {
	resources := schema.NewResources()

	// Get Redshift client
	redshiftClient := redshift.NewFromConfig(p.provider.session)

	// Get all clusters
	clusters, err := p.getClusters(ctx, redshiftClient)
	if err != nil {
		return nil, fmt.Errorf("could not get Redshift clusters: %w", err)
	}
	resources.Merge(clusters)

	return resources, nil
}

// getClusters retrieves all Redshift clusters
func (p *redshiftProvider) getClusters(ctx context.Context, client *redshift.Client) (*schema.Resources, error) {
	resources := schema.NewResources()

	input := &redshift.DescribeClustersInput{
		MaxRecords: aws.Int32(100),
	}

	paginator := redshift.NewDescribeClustersPaginator(client, input)
	for paginator.HasMorePages() {
		page, err := paginator.NextPage(ctx)
		if err != nil {
			return nil, err
		}

		for _, cluster := range page.Clusters {
			// Extract cluster endpoint information
			var dnsName string
			var publicIPv4, privateIPv4 string

			if cluster.Endpoint != nil {
				dnsName = aws.ToString(cluster.Endpoint.Address)
				if cluster.Endpoint.Port != nil {
					dnsName += fmt.Sprintf(":%d", aws.ToInt32(cluster.Endpoint.Port))
				}
			}

			// Extract IP addresses from VPC endpoints
			if len(cluster.VpcSecurityGroups) > 0 {
				for _, sg := range cluster.VpcSecurityGroups {
					if sg.VpcId != nil {
						privateIPv4 = aws.ToString(sg.VpcId)
					}
				}
			}

			// Extract cluster information
			resource := &schema.Resource{
				ID:          fmt.Sprintf("redshift-cluster-%s", aws.ToString(cluster.ClusterIdentifier)),
				Public:      cluster.PubliclyAccessible != nil && aws.ToBool(cluster.PubliclyAccessible),
				Provider:    "aws",
				Service:     "redshift",
				DNSName:     dnsName,
				PublicIPv4:  publicIPv4,
				PrivateIPv4: privateIPv4,
				Metadata: map[string]string{
					"cluster_identifier":                      aws.ToString(cluster.ClusterIdentifier),
					"cluster_name":                            aws.ToString(cluster.ClusterIdentifier),
					"node_type":                               aws.ToString(cluster.NodeType),
					"cluster_status":                          aws.ToString(cluster.ClusterStatus),
					"cluster_availability_zone":               aws.ToString(cluster.AvailabilityZone),
					"cluster_create_time":                     aws.ToString(cluster.ClusterCreateTime),
					"automated_snapshot_retention_period":     fmt.Sprintf("%d", aws.ToInt32(cluster.AutomatedSnapshotRetentionPeriod)),
					"manual_snapshot_retention_period":        fmt.Sprintf("%d", aws.ToInt32(cluster.ManualSnapshotRetentionPeriod)),
					"number_of_nodes":                         fmt.Sprintf("%d", aws.ToInt32(cluster.NumberOfNodes)),
					"publicly_accessible":                     fmt.Sprintf("%v", aws.ToBool(cluster.PubliclyAccessible)),
					"encrypted":                               fmt.Sprintf("%v", aws.ToBool(cluster.Encrypted)),
					"multi_az":                                fmt.Sprintf("%v", aws.ToBool(cluster.MultiAZ)),
					"enhanced_vpc_routing":                    fmt.Sprintf("%v", aws.ToBool(cluster.EnhancedVpcRouting)),
					"maintenance_track_name":                  aws.ToString(cluster.MaintenanceTrackName),
					"deferred_maintenance_window_identifiers": strings.Join(cluster.DeferredMaintenanceWindowIdentifiers, ","),
					"cluster_revision_number":                 aws.ToString(cluster.ClusterRevisionNumber),
					"allow_version_upgrade":                   fmt.Sprintf("%v", aws.ToBool(cluster.AllowVersionUpgrade)),
					"cluster_parameter_groups":                p.extractClusterParameterGroups(cluster.ClusterParameterGroups),
					"cluster_subnet_group_name":               aws.ToString(cluster.ClusterSubnetGroupName),
					"vpc_id":                                  aws.ToString(cluster.VpcId),
					"iam_roles":                               p.extractIamRoles(cluster.IamRoles),
					"db_name":                                 aws.ToString(cluster.DBName),
					"master_username":                         aws.ToString(cluster.MasterUsername),
					"cluster_version":                         aws.ToString(cluster.ClusterVersion),
					"cluster_type":                            aws.ToString(cluster.ClusterType),
					"cluster_security_groups":                 p.extractClusterSecurityGroups(cluster.ClusterSecurityGroups),
				},
			}

			// Add endpoint information
			if cluster.Endpoint != nil {
				resource.Metadata["endpoint_address"] = aws.ToString(cluster.Endpoint.Address)
				resource.Metadata["endpoint_port"] = fmt.Sprintf("%d", aws.ToInt32(cluster.Endpoint.Port))
			}

			// Add restore status information
			if cluster.RestoreStatus != nil {
				resource.Metadata["restore_status"] = aws.ToString(cluster.RestoreStatus.Status)
				resource.Metadata["restore_from_cluster_identifier"] = aws.ToString(cluster.RestoreStatus.FromClusterIdentifier)
				resource.Metadata["restore_snapshot_identifier"] = aws.ToString(cluster.RestoreStatus.SnapshotIdentifier)
				resource.Metadata["restore_current_restore_rate_in_mega_bytes_per_second"] = fmt.Sprintf("%f", aws.ToFloat64(cluster.RestoreStatus.CurrentRestoreRateInMegaBytesPerSecond))
				resource.Metadata["restore_snapshot_size_in_mega_bytes"] = fmt.Sprintf("%d", aws.ToInt64(cluster.RestoreStatus.SnapshotSizeInMegaBytes))
				resource.Metadata["restore_progress_in_mega_bytes"] = fmt.Sprintf("%d", aws.ToInt64(cluster.RestoreStatus.ProgressInMegaBytes))
				resource.Metadata["restore_elapsed_time_in_seconds"] = fmt.Sprintf("%d", aws.ToInt64(cluster.RestoreStatus.ElapsedTimeInSeconds))
				resource.Metadata["restore_estimated_time_to_completion_in_seconds"] = fmt.Sprintf("%d", aws.ToInt64(cluster.RestoreStatus.EstimatedTimeToCompletionInSeconds))
			}

			// Add data transfer progress information
			if cluster.DataTransferProgress != nil {
				resource.Metadata["data_transfer_progress_status"] = aws.ToString(cluster.DataTransferProgress.Status)
				resource.Metadata["data_transfer_progress_current_rate_in_mega_bytes_per_second"] = fmt.Sprintf("%f", aws.ToFloat64(cluster.DataTransferProgress.CurrentRateInMegaBytesPerSecond))
				resource.Metadata["data_transfer_progress_total_data_in_mega_bytes"] = fmt.Sprintf("%d", aws.ToInt64(cluster.DataTransferProgress.TotalDataInMegaBytes))
				resource.Metadata["data_transfer_progress_data_transferred_in_mega_bytes"] = fmt.Sprintf("%d", aws.ToInt64(cluster.DataTransferProgress.DataTransferredInMegaBytes))
				resource.Metadata["data_transfer_progress_elapsed_time_in_seconds"] = fmt.Sprintf("%d", aws.ToInt64(cluster.DataTransferProgress.ElapsedTimeInSeconds))
				resource.Metadata["data_transfer_progress_estimated_time_to_completion_in_seconds"] = fmt.Sprintf("%d", aws.ToInt64(cluster.DataTransferProgress.EstimatedTimeToCompletionInSeconds))
			}

			// Add HSM status information
			if cluster.HsmStatus != nil {
				resource.Metadata["hsm_status"] = aws.ToString(cluster.HsmStatus.HsmClientCertificateIdentifier)
				resource.Metadata["hsm_configuration_identifier"] = aws.ToString(cluster.HsmStatus.HsmConfigurationIdentifier)
				resource.Metadata["hsm_status"] = aws.ToString(cluster.HsmStatus.Status)
			}

			// Add cluster snapshot copy status
			if cluster.ClusterSnapshotCopyStatus != nil {
				resource.Metadata["snapshot_copy_destination_region"] = aws.ToString(cluster.ClusterSnapshotCopyStatus.DestinationRegion)
				resource.Metadata["snapshot_copy_retention_period"] = fmt.Sprintf("%d", aws.ToInt32(cluster.ClusterSnapshotCopyStatus.RetentionPeriod))
				resource.Metadata["snapshot_copy_manual_snapshot_retention_period"] = fmt.Sprintf("%d", aws.ToInt32(cluster.ClusterSnapshotCopyStatus.ManualSnapshotRetentionPeriod))
				resource.Metadata["snapshot_copy_grant_number"] = fmt.Sprintf("%d", aws.ToInt32(cluster.ClusterSnapshotCopyStatus.SnapshotCopyGrantNumber))
			}

			// Add logging status information
			if cluster.LoggingStatus != nil {
				resource.Metadata["logging_enabled"] = fmt.Sprintf("%v", aws.ToBool(cluster.LoggingStatus.LoggingEnabled))
				resource.Metadata["logging_bucket_name"] = aws.ToString(cluster.LoggingStatus.BucketName)
				resource.Metadata["logging_s3_key_prefix"] = aws.ToString(cluster.LoggingStatus.S3KeyPrefix)
				resource.Metadata["logging_last_successful_delivery_time"] = aws.ToString(cluster.LoggingStatus.LastSuccessfulDeliveryTime)
				resource.Metadata["logging_last_failure_time"] = aws.ToString(cluster.LoggingStatus.LastFailureTime)
				resource.Metadata["logging_last_failure_message"] = aws.ToString(cluster.LoggingStatus.LastFailureMessage)
			}

			// Add pending modified values
			if cluster.PendingModifiedValues != nil {
				resource.Metadata["pending_master_user_password"] = aws.ToString(cluster.PendingModifiedValues.MasterUserPassword)
				resource.Metadata["pending_node_type"] = aws.ToString(cluster.PendingModifiedValues.NodeType)
				resource.Metadata["pending_number_of_nodes"] = fmt.Sprintf("%d", aws.ToInt32(cluster.PendingModifiedValues.NumberOfNodes))
				resource.Metadata["pending_cluster_type"] = aws.ToString(cluster.PendingModifiedValues.ClusterType)
				resource.Metadata["pending_cluster_version"] = aws.ToString(cluster.PendingModifiedValues.ClusterVersion)
				resource.Metadata["pending_automated_snapshot_retention_period"] = fmt.Sprintf("%d", aws.ToInt32(cluster.PendingModifiedValues.AutomatedSnapshotRetentionPeriod))
				resource.Metadata["pending_cluster_identifier"] = aws.ToString(cluster.PendingModifiedValues.ClusterIdentifier)
			}

			resources.Append(resource)
		}
	}

	return resources, nil
}

// extractClusterParameterGroups extracts parameter group information
func (p *redshiftProvider) extractClusterParameterGroups(groups []types.ClusterParameterGroup) string {
	var result []string
	for _, group := range groups {
		result = append(result, fmt.Sprintf("%s:%s", aws.ToString(group.ParameterGroupName), aws.ToString(group.ParameterApplyStatus)))
	}
	return strings.Join(result, ",")
}

// extractClusterSecurityGroups extracts security group information
func (p *redshiftProvider) extractClusterSecurityGroups(groups []types.ClusterSecurityGroup) string {
	var result []string
	for _, group := range groups {
		result = append(result, fmt.Sprintf("%s:%s", aws.ToString(group.ClusterSecurityGroupName), aws.ToString(group.Status)))
	}
	return strings.Join(result, ",")
}

// extractIamRoles extracts IAM role information
func (p *redshiftProvider) extractIamRoles(roles []types.ClusterIamRole) string {
	var result []string
	for _, role := range roles {
		result = append(result, fmt.Sprintf("%s:%s", aws.ToString(role.IamRoleArn), aws.ToString(role.ApplyStatus)))
	}
	return strings.Join(result, ",")
}
