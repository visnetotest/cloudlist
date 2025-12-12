package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/projectdiscovery/cloudlist/internal/runner"
	"github.com/projectdiscovery/cloudlist/pkg/database"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
	"github.com/projectdiscovery/gologger"
)

// Server represents the web server
type Server struct {
	db     *database.SQLiteDB
	router *gin.Engine
}

// NewServer creates a new web server
func NewServer(dbPath string) (*Server, error) {
	// Initialize SQLite database
	db, err := database.NewSQLite(dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize database: %w", err)
	}

	// Create Gin router
	router := gin.Default()

	server := &Server{
		db:     db,
		router: router,
	}

	server.setupRoutes()
	return server, nil
}

// setupRoutes configures all API routes
func (s *Server) setupRoutes() {
	// Serve static files
	s.router.Static("/static", "./static")
	s.router.StaticFile("/", "./static/index.html")

	// API routes
	api := s.router.Group("/api/v1")
	{
		// Asset routes
		api.GET("/assets", s.getAssets)
		api.GET("/assets/count", s.getAssetCount)
		api.GET("/assets/providers", s.getProviders)
		api.GET("/assets/services", s.getServices)
		api.GET("/assets/provider/:provider", s.getAssetsByProvider)
		api.GET("/assets/service/:service", s.getAssetsByService)
		api.POST("/assets", s.addAsset)
		api.DELETE("/assets/provider/:provider/resource/:resourceId", s.deleteAsset)

		// Discovery routes
		api.POST("/discover", s.discoverAssets)
	}

	// Health check
	s.router.GET("/health", s.healthCheck)
}

// getAssets returns all assets
func (s *Server) getAssets(c *gin.Context) {
	assets, err := s.db.GetAssets()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"data": assets, "count": len(assets)})
}

// getAssetCount returns the total number of assets
func (s *Server) getAssetCount(c *gin.Context) {
	count, err := s.db.GetAssetCount()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"count": count})
}

// getProviders returns all unique providers
func (s *Server) getProviders(c *gin.Context) {
	providers, err := s.db.GetProviders()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"data": providers})
}

// getServices returns all unique services
func (s *Server) getServices(c *gin.Context) {
	services, err := s.db.GetServices()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"data": services})
}

// getAssetsByProvider returns assets filtered by provider
func (s *Server) getAssetsByProvider(c *gin.Context) {
	provider := c.Param("provider")
	assets, err := s.db.GetAssetsByProvider(provider)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"data": assets, "count": len(assets)})
}

// getAssetsByService returns assets filtered by service
func (s *Server) getAssetsByService(c *gin.Context) {
	service := c.Param("service")
	assets, err := s.db.GetAssetsByService(service)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"data": assets, "count": len(assets)})
}

// addAsset adds a new asset
func (s *Server) addAsset(c *gin.Context) {
	var asset schema.Resource
	if err := c.ShouldBindJSON(&asset); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := s.db.AddAsset(&asset); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "Asset added successfully", "data": asset})
}

// deleteAsset deletes an asset
func (s *Server) deleteAsset(c *gin.Context) {
	provider := c.Param("provider")
	resourceID := c.Param("resourceId")

	if err := s.db.DeleteAsset(provider, resourceID); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Asset deleted successfully"})
}

// discoverAssets runs asset discovery using providers
func (s *Server) discoverAssets(c *gin.Context) {
	var request DiscoveryRequest
	if err := c.ShouldBindJSON(&request); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Create runner options
	options := &runner.Options{
		ProviderConfig: request.ConfigFile,
	}

	// Filter providers if specified
	if len(request.Providers) > 0 {
		options.Providers = request.Providers
	}

	// Filter services if specified
	if len(request.Services) > 0 {
		options.Services = request.Services
	}

	// Create runner and enumerate
	run, err := runner.New(options)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// Run discovery
	resources := run.Enumerate()

	// Store discovered assets in database
	for _, resource := range resources {
		if err := s.db.AddAsset(resource); err != nil {
			gologger.Warning().Msgf("Failed to store asset %s: %v", resource.ID, err)
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "Discovery completed successfully",
		"data":    resources,
		"count":   len(resources),
	})
}

// healthCheck returns server health status
func (s *Server) healthCheck(c *gin.Context) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := s.db.Ping(ctx); err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "unhealthy",
			"error":  err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"status": "healthy",
		"time":   time.Now().UTC(),
	})
}

// Start starts the web server
func (s *Server) Start(port string) error {
	gologger.Info().Msgf("Starting web server on port %s", port)
	return s.router.Run(":" + port)
}

// Close closes the server and database connections
func (s *Server) Close() error {
	return s.db.Close()
}

// DiscoveryRequest represents a discovery request
type DiscoveryRequest struct {
	ConfigFile string   `json:"config_file" binding:"required"`
	Providers  []string `json:"providers,omitempty"`
	Services   []string `json:"services,omitempty"`
}

func main() {
	// Get configuration
	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "./cloudlist.db"
	}

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	// Create server
	server, err := NewServer(dbPath)
	if err != nil {
		log.Fatalf("Failed to create server: %v", err)
	}
	defer server.Close()

	// Ensure static directory exists
	if err := os.MkdirAll("./static", 0755); err != nil {
		log.Fatalf("Failed to create static directory: %v", err)
	}

	// Start server
	if err := server.Start(port); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}
