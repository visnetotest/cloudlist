package pipeline

import (
	"encoding/json"
	"os"
	"sync"

	"github.com/projectdiscovery/cloudlist/pkg/database"
)

// Pipeline represents the data ingestion pipeline.
// For the MVP, it uses a file-based queue.
type Pipeline struct {
	mu       sync.Mutex
	path     string
	database *database.Database
}

// New creates a new pipeline.
func New(path string, db *database.Database) (*Pipeline, error) {
	return &Pipeline{path: path, database: db}, nil
}

// AddEvent adds an event to the pipeline's queue.
func (p *Pipeline) AddEvent(event *Event) error {
	p.mu.Lock()
	defer p.mu.Unlock()

	events, err := p.readEvents()
	if err != nil {
		return err
	}

	events = append(events, event)

	return p.writeEvents(events)
}

// ProcessEvents processes all events in the queue.
func (p *Pipeline) ProcessEvents() error {
	p.mu.Lock()
	defer p.mu.Unlock()

	events, err := p.readEvents()
	if err != nil {
		return err
	}

	for _, event := range events {
		if err := p.database.AddAsset(event.Asset); err != nil {
			return err
		}
	}

	// Clear the queue after processing
	return p.writeEvents([]*Event{})
}

func (p *Pipeline) readEvents() ([]*Event, error) {
	data, err := os.ReadFile(p.path)
	if err != nil {
		if os.IsNotExist(err) {
			return []*Event{}, nil
		}
		return nil, err
	}

	var events []*Event
	if err := json.Unmarshal(data, &events); err != nil {
		return nil, err
	}

	return events, nil
}

func (p *Pipeline) writeEvents(events []*Event) error {
	data, err := json.MarshalIndent(events, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(p.path, data, 0644)
}
