package database

import (
	"encoding/json"
	"os"
	"sync"

	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// Database represents the asset database.
// For the MVP, it's a simple JSON file.
type Database struct {
	mu   sync.Mutex
	path string
}

// New creates a new database.
func New(path string) (*Database, error) {
	return &Database{path: path}, nil
}

// AddAsset adds an asset to the database.
func (db *Database) AddAsset(asset *schema.Resource) error {
	db.mu.Lock()
	defer db.mu.Unlock()

	assets, err := db.readAssets()
	if err != nil {
		return err
	}

	assets = append(assets, asset)

	return db.writeAssets(assets)
}

func (db *Database) readAssets() ([]*schema.Resource, error) {
	data, err := os.ReadFile(db.path)
	if err != nil {
		if os.IsNotExist(err) {
			return []*schema.Resource{}, nil
		}
		return nil, err
	}

	var assets []*schema.Resource
	if err := json.Unmarshal(data, &assets); err != nil {
		return nil, err
	}

	return assets, nil
}

func (db *Database) writeAssets(assets []*schema.Resource) error {
	data, err := json.MarshalIndent(assets, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(db.path, data, 0644)
}
