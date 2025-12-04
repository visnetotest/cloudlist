package main

import (
	"log"

	"github.com/nats-io/nats.go"
)

func main() {
	// Connect to NATS
	nc, err := nats.Connect(nats.DefaultURL)
	if err != nil {
		log.Fatalf("Error connecting to NATS: %v", err)
	}
	defer nc.Close()

	log.Println("Connected to NATS")

	// Subscribe to the asset events
	_, err = nc.Subscribe("asset.events", func(m *nats.Msg) {
		log.Printf("Received event: %s", string(m.Data))
	})
	if err != nil {
		log.Fatalf("Error subscribing to NATS: %v", err)
	}

	log.Println("Subscribed to asset.events")

	// Keep the connection alive
	select {}
}
