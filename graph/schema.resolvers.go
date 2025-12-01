package graph

import (
    "context"
    "fmt"

    "github.com/projectdiscovery/cloudlist/graph/model"
    "github.com/projectdiscovery/cloudlist/pkg/inventory"
    "github.com/projectdiscovery/cloudlist/pkg/schema"
)

func (r *queryResolver) Assets(ctx context.Context) ([]*model.Asset, error) {
    inventory, err := inventory.New(schema.Options{})
    if err != nil {
        return nil, fmt.Errorf("failed to create inventory: %w", err)
    }

    var assets []*model.Asset
    for _, provider := range inventory.Providers {
        resources, err := provider.Resources(ctx)
        if err != nil {
            return nil, fmt.Errorf("failed to get resources from provider %s: %w", provider.Name(), err)
        }

        for _, resource := range resources.Items {
            assets = append(assets, &model.Asset{
                Public:      resource.Public,
                Provider:    resource.Provider,
                Service:     resource.Service,
                ID:          resource.ID,
                PublicIpv4:  &resource.PublicIPv4,
                PublicIpv6:  &resource.PublicIPv6,
                PrivateIpv4: &resource.PrivateIpv4,
                PrivateIpv6: &resource.PrivateIpv6,
                DNSName:     &resource.DNSName,
                Metadata:    resource.Metadata,
            })
        }
    }

    return assets, nil
}
