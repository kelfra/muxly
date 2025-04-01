#!/bin/bash
set -e

# Configuration
DOCKER_REPO="muxly"
IMAGE_NAME="muxly"
VERSION=$(grep '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/' | tr -d '[:space:]')

# Print details
echo "Building Docker image: $DOCKER_REPO/$IMAGE_NAME:$VERSION"
echo "Working directory: $(pwd)"

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Make sure you're running this script from the project root."
    exit 1
fi

# Create a temporary Dockerfile
cat > Dockerfile << EOF
FROM rust:1.76-slim-bullseye as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY . .

# Build the application in release mode
RUN cargo build --release

# Create a new stage with a minimal image
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/muxly /app/muxly

# Expose API port
EXPOSE 3000

# Create config and data directories
RUN mkdir -p /var/lib/muxly/config /var/lib/muxly/data

# Create volume mount points
VOLUME ["/var/lib/muxly/config", "/var/lib/muxly/data"]

# Set environment variables
ENV RUST_LOG=info
ENV MUXLY_CONFIG_PATH=/var/lib/muxly/config

# Run the service
CMD ["/app/muxly"]
EOF

# Build the Docker image
docker build -t $DOCKER_REPO/$IMAGE_NAME:$VERSION .
docker tag $DOCKER_REPO/$IMAGE_NAME:$VERSION $DOCKER_REPO/$IMAGE_NAME:latest

echo "Build completed successfully."
echo "You can now run the image with:"
echo "  docker run -p 3000:3000 -v \$(pwd)/config:/var/lib/muxly/config $DOCKER_REPO/$IMAGE_NAME:$VERSION"

# Clean up temporary Dockerfile
rm Dockerfile

# Optionally push to Docker registry
read -p "Do you want to push the image to Docker Hub? [y/N] " PUSH
if [[ "$PUSH" =~ ^[Yy]$ ]]; then
    echo "Pushing to Docker Hub..."
    docker push $DOCKER_REPO/$IMAGE_NAME:$VERSION
    docker push $DOCKER_REPO/$IMAGE_NAME:latest
    echo "Push completed."
fi 