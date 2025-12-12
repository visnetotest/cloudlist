package database

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"

	"github.com/jmoiron/sqlx"
	_ "github.com/mattn/go-sqlite3"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// SQLiteDB represents SQLite asset database
type SQLiteDB struct {
	db *sqlx.DB
}

// NewSQLite creates a new SQLite database
func NewSQLite(dbPath string) (*SQLiteDB, error) {
	db, err := sqlx.Connect("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	// Enable foreign key support
	db.Exec("PRAGMA foreign_keys = ON")

	sqliteDB := &SQLiteDB{db: db}

	// Initialize database schema
	if err := sqliteDB.initSchema(); err != nil {
		return nil, fmt.Errorf("failed to initialize database schema: %w", err)
	}

	return sqliteDB, nil
}

// initSchema creates the necessary tables
func (s *SQLiteDB) initSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS assets (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		public BOOLEAN NOT NULL DEFAULT FALSE,
		provider TEXT NOT NULL,
		service TEXT,
		resource_id TEXT,
		public_ipv4 TEXT,
		public_ipv6 TEXT,
		private_ipv4 TEXT,
		private_ipv6 TEXT,
		dns_name TEXT,
		metadata TEXT,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);

	CREATE INDEX IF NOT EXISTS idx_assets_provider ON assets(provider);
	CREATE INDEX IF NOT EXISTS idx_assets_service ON assets(service);
	CREATE INDEX IF NOT EXISTS idx_assets_public_ipv4 ON assets(public_ipv4);
	CREATE INDEX IF NOT EXISTS idx_assets_public_ipv6 ON assets(public_ipv6);
	CREATE INDEX IF NOT EXISTS idx_assets_dns_name ON assets(dns_name);
	`

	_, err := s.db.Exec(schema)
	return err
}

// AddAsset adds an asset to the database
func (s *SQLiteDB) AddAsset(asset *schema.Resource) error {
	metadataJSON, err := json.Marshal(asset.Metadata)
	if err != nil {
		return fmt.Errorf("failed to marshal metadata: %w", err)
	}

	query := `
		INSERT INTO assets (public, provider, service, resource_id, public_ipv4, public_ipv6, private_ipv4, private_ipv6, dns_name, metadata)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`

	_, err = s.db.Exec(query,
		asset.Public,
		asset.Provider,
		asset.Service,
		asset.ID,
		asset.PublicIPv4,
		asset.PublicIPv6,
		asset.PrivateIpv4,
		asset.PrivateIpv6,
		asset.DNSName,
		string(metadataJSON),
	)

	return err
}

// GetAssets retrieves all assets from the database
func (s *SQLiteDB) GetAssets() ([]*schema.Resource, error) {
	query := `
		SELECT public, provider, service, resource_id, public_ipv4, public_ipv6, private_ipv4, private_ipv6, dns_name, metadata
		FROM assets
		ORDER BY created_at DESC
	`

	rows, err := s.db.Query(query)
	if err != nil {
		return nil, fmt.Errorf("failed to query assets: %w", err)
	}
	defer rows.Close()

	var assets []*schema.Resource
	for rows.Next() {
		var asset schema.Resource
		var metadataJSON sql.NullString

		err := rows.Scan(
			&asset.Public,
			&asset.Provider,
			&asset.Service,
			&asset.ID,
			&asset.PublicIPv4,
			&asset.PublicIPv6,
			&asset.PrivateIpv4,
			&asset.PrivateIpv6,
			&asset.DNSName,
			&metadataJSON,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan asset: %w", err)
		}

		if metadataJSON.Valid {
			if err := json.Unmarshal([]byte(metadataJSON.String), &asset.Metadata); err != nil {
				return nil, fmt.Errorf("failed to unmarshal metadata: %w", err)
			}
		}

		assets = append(assets, &asset)
	}

	return assets, rows.Err()
}

// GetAssetsByProvider retrieves assets by provider
func (s *SQLiteDB) GetAssetsByProvider(provider string) ([]*schema.Resource, error) {
	query := `
		SELECT public, provider, service, resource_id, public_ipv4, public_ipv6, private_ipv4, private_ipv6, dns_name, metadata
		FROM assets
		WHERE provider = ?
		ORDER BY created_at DESC
	`

	rows, err := s.db.Query(query, provider)
	if err != nil {
		return nil, fmt.Errorf("failed to query assets by provider: %w", err)
	}
	defer rows.Close()

	var assets []*schema.Resource
	for rows.Next() {
		var asset schema.Resource
		var metadataJSON sql.NullString

		err := rows.Scan(
			&asset.Public,
			&asset.Provider,
			&asset.Service,
			&asset.ID,
			&asset.PublicIPv4,
			&asset.PublicIPv6,
			&asset.PrivateIpv4,
			&asset.PrivateIpv6,
			&asset.DNSName,
			&metadataJSON,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan asset: %w", err)
		}

		if metadataJSON.Valid {
			if err := json.Unmarshal([]byte(metadataJSON.String), &asset.Metadata); err != nil {
				return nil, fmt.Errorf("failed to unmarshal metadata: %w", err)
			}
		}

		assets = append(assets, &asset)
	}

	return assets, rows.Err()
}

// GetAssetsByService retrieves assets by service
func (s *SQLiteDB) GetAssetsByService(service string) ([]*schema.Resource, error) {
	query := `
		SELECT public, provider, service, resource_id, public_ipv4, public_ipv6, private_ipv4, private_ipv6, dns_name, metadata
		FROM assets
		WHERE service = ?
		ORDER BY created_at DESC
	`

	rows, err := s.db.Query(query, service)
	if err != nil {
		return nil, fmt.Errorf("failed to query assets by service: %w", err)
	}
	defer rows.Close()

	var assets []*schema.Resource
	for rows.Next() {
		var asset schema.Resource
		var metadataJSON sql.NullString

		err := rows.Scan(
			&asset.Public,
			&asset.Provider,
			&asset.Service,
			&asset.ID,
			&asset.PublicIPv4,
			&asset.PublicIPv6,
			&asset.PrivateIpv4,
			&asset.PrivateIpv6,
			&asset.DNSName,
			&metadataJSON,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan asset: %w", err)
		}

		if metadataJSON.Valid {
			if err := json.Unmarshal([]byte(metadataJSON.String), &asset.Metadata); err != nil {
				return nil, fmt.Errorf("failed to unmarshal metadata: %w", err)
			}
		}

		assets = append(assets, &asset)
	}

	return assets, rows.Err()
}

// DeleteAsset deletes an asset by provider and resource ID
func (s *SQLiteDB) DeleteAsset(provider, resourceID string) error {
	query := `DELETE FROM assets WHERE provider = ? AND resource_id = ?`

	result, err := s.db.Exec(query, provider, resourceID)
	if err != nil {
		return fmt.Errorf("failed to delete asset: %w", err)
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}

	if rowsAffected == 0 {
		return fmt.Errorf("asset not found")
	}

	return nil
}

// GetAssetCount returns the total number of assets
func (s *SQLiteDB) GetAssetCount() (int, error) {
	var count int
	err := s.db.Get(&count, "SELECT COUNT(*) FROM assets")
	return count, err
}

// GetProviders returns all unique providers
func (s *SQLiteDB) GetProviders() ([]string, error) {
	var providers []string
	err := s.db.Select(&providers, "SELECT DISTINCT provider FROM assets ORDER BY provider")
	return providers, err
}

// GetServices returns all unique services
func (s *SQLiteDB) GetServices() ([]string, error) {
	var services []string
	err := s.db.Select(&services, "SELECT DISTINCT service FROM assets WHERE service IS NOT NULL ORDER BY service")
	return services, err
}

// Close closes the database connection
func (s *SQLiteDB) Close() error {
	return s.db.Close()
}

// Ping checks if the database is alive
func (s *SQLiteDB) Ping(ctx context.Context) error {
	return s.db.PingContext(ctx)
}
