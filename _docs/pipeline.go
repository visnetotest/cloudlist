package pipeline

import (
	"encoding/json"
	"log"

	"github.com/nats-io/nats.go"
)

// Pipeline represents the data ingestion pipeline.
// It uses NATS to publish asset discovery events.
type Pipeline struct {
	nc *nats.Conn
}

// New creates a new pipeline and connects to NATS.
func New(natsURL string) (*Pipeline, error) {
	nc, err := nats.Connect(natsURL)
	if err != nil {
		return nil, err
	}
	return &Pipeline{nc: nc}, nil
}

// AddEvent publishes an asset discovery event to the NATS bus.
func (p *Pipeline) AddEvent(event *Event) error {
	// Marshal the event to JSON
	data, err := json.Marshal(event)
	if err != nil {
		log.Printf("Could not marshal event to JSON: %v", err)
		return err
	}

	// Publish the event to the "asset.events" subject
	if err := p.nc.Publish("asset.events", data); err != nil {
		log.Printf("Could not publish event to NATS: %v", err)
		return err
	}
	log.Printf("Published event for asset: %s", event.Asset.DNSName)
	return nil
}

// Close closes the NATS connection.
func (p *Pipeline) Close() {
	if p.nc != nil {
		p.nc.Close()
	}
}
