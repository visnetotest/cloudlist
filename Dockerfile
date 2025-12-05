# Use the official Golang image to build the application
FROM golang:1.24-alpine AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy the go.mod and go.sum files to download dependencies
COPY go.mod go.sum ./

# Download all dependencies. Dependencies will be cached if the go.mod and go.sum files are not changed
RUN go mod download

# Copy the source code into the container
COPY . .

# Build the application for a static, linux-native binary
RUN CGO_ENABLED=0 GOOS=linux go build -v -o /app/cloudlist cmd/cloudlist/main.go

# --- Final Stage ---
# Use a minimal, non-root image for the final container
FROM alpine:latest

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/cloudlist .

# The 'command' in docker-compose.yml will override this, but it's good practice
# to define a default entrypoint.
ENTRYPOINT ["/app/cloudlist"]
