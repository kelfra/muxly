# Muxly User Guide

Welcome to the Muxly User Guide. This documentation will help you make the most of Muxly's features for collecting, unifying, and routing data.

## Getting Started

Muxly is a lightweight, cross-platform service that enables you to collect, unify, and route product metrics and data from disparate sources. It connects internal APIs with third-party services, transforming all data into a consistent JSON format before routing it to desired destinations.

## Core Concepts

- [**Configuration Guide**](configuration.md) - How to configure Muxly
- [**Authentication Guide**](authentication.md) - Setting up and using authentication
- [**Connectors Guide**](connectors.md) - Working with data source connectors
- [**Scheduler Guide**](scheduler.md) - Automating data tasks
- [**Router Guide**](router.md) - Defining data routes
- [**Destinations Guide**](destinations.md) - Configuring data destinations
- [**Router Rules**](router-rules.md) - Setting up conditional routing rules

## API Documentation

- [**API Reference**](api.md) - Comprehensive documentation of REST API endpoints
- **Swagger UI** - Interactive API documentation (available at `/api/docs` when Muxly is running)

## Quick Setup

### Docker Installation

The fastest way to get started with Muxly is using Docker:

```bash
# Pull the Docker image
docker pull muxly/muxly:latest

# Run with a mounted config directory
docker run -p 3000:3000 -v $(pwd)/config:/var/lib/muxly/config muxly/muxly:latest
```

Or using Docker Compose:

```bash
# Create a docker-compose.yml file
cat > docker-compose.yml << EOF
version: '3.8'

services:
  muxly:
    image: muxly/muxly:latest
    ports:
      - "3000:3000"
    volumes:
      - ./config:/var/lib/muxly/config
      - ./data:/var/lib/muxly/data
    environment:
      - RUST_LOG=info
EOF

# Start the container
docker-compose up -d
```

## Need Help?

If you need additional help:

- Check the [FAQ](../troubleshooting/faq.md)
- Read the [Troubleshooting Guide](../troubleshooting/README.md)
- View [Examples](../examples/README.md) of common configurations

## Tutorials

- [Setting up your first connector](./tutorials/first-connector.md)
- [Creating a data pipeline](./tutorials/data-pipeline.md)
- [Scheduling recurring tasks](./tutorials/scheduling.md)

## Reference

- [API Reference](./api-reference.md)
- [Configuration Reference](./configuration-reference.md)
- [CLI Reference](./cli-reference.md)

## Troubleshooting

- [Common Issues](./troubleshooting.md)
- [FAQ](./faq.md)

## Additional Resources

- [Examples](../examples) - Example configurations and use cases
- [Development Documentation](../development) - Documentation for developers 