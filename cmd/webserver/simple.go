package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/projectdiscovery/cloudlist/pkg/database"
	"github.com/projectdiscovery/cloudlist/pkg/schema"
)

// Server represents a simple web server for testing
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
		api.POST("/assets", s.addAsset)
		api.DELETE("/assets/provider/:provider/resource/:resourceId", s.deleteAsset)

		// Health check
		api.GET("/health", s.healthCheck)
	}
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

// getAssetCount returns total number of assets
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
	log.Printf("Starting web server on port %s", port)
	return s.router.Run(":" + port)
}

// Close closes the server and database connections
func (s *Server) Close() error {
	return s.db.Close()
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
