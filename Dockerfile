# Use the official Golang image to create a build artifact.
FROM golang:1.24-alpine as builder

WORKDIR /app

# Copy go.mod and go.sum files to leverage Docker cache
COPY go.mod go.sum ./

# Download dependencies
RUN go mod download

# Copy the rest of the application source code
COPY . .

# Build the cloudlist binary
RUN go build -v -o cloudlist cmd/cloudlist/main.go

# ---

# The final image is lightweight
FROM alpine:latest

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/cloudlist .
COPY --from=builder /app/demo/ ./demo/

# Grant execution permissions
RUN chmod +x cloudlist

# The default command will be to run the application,
# but it will be overridden in the docker-compose file.
CMD ["./cloudlist"]
