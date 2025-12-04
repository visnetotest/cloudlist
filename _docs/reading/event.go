package pipeline

import "github.com/projectdiscovery/cloudlist/pkg/schema"

// Event represents a single asset discovery event.
// It contains the asset data and the provider it came from.
type Event struct {
	Provider string           `json:"provider"`
	Asset    *schema.Resource `json:"asset"`
}
