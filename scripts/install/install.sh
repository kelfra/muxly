#!/bin/bash
set -e

# Muxly Installation Script
# This script automates the installation of Muxly

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Print a message with a colored prefix
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Print the banner
print_banner() {
    echo -e "${BLUE}"
    echo "=========================="
    echo "   Muxly Installer"
    echo "=========================="
    echo -e "${NC}"
    echo "This script will install Muxly on your system."
    echo
}

# Check if Docker is installed
check_docker() {
    info "Checking if Docker is installed..."
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed. Please install Docker before continuing."
    else
        success "Docker is installed."
    fi

    info "Checking if Docker Compose is installed..."
    if ! command -v docker-compose &> /dev/null; then
        if ! docker compose version &> /dev/null; then
            error "Docker Compose is not installed. Please install Docker Compose before continuing."
        else
            success "Docker Compose (plugin) is installed."
        fi
    else
        success "Docker Compose is installed."
    fi
}

# Create data directories
create_directories() {
    info "Creating data directories..."
    mkdir -p data/output config templates
    success "Directories created."
}

# Check if config file exists, create if it doesn't
check_config() {
    info "Checking configuration..."
    if [ ! -f "config/muxly.toml" ]; then
        info "Configuration file not found. Creating a default one..."
        if [ ! -f "config/muxly.toml.example" ]; then
            error "Example configuration file not found. Please download the Muxly package again."
        fi
        cp config/muxly.toml.example config/muxly.toml
        success "Default configuration created at config/muxly.toml"
        info "Please review and update the configuration file as needed."
    else
        success "Configuration file exists."
    fi
}

# Set permissions
set_permissions() {
    info "Setting permissions..."
    chmod -R 755 data config templates
    success "Permissions set."
}

# Pull or build Docker images
build_images() {
    info "Building Docker images..."
    if [ -f "docker-compose.yml" ]; then
        docker-compose build
        success "Docker images built successfully."
    else
        error "docker-compose.yml not found. Please download the Muxly package again."
    fi
}

# Start Muxly
start_muxly() {
    info "Starting Muxly..."
    docker-compose up -d
    success "Muxly is now running!"
    info "You can access the API at http://localhost:3000"
    info "To view logs, run: docker-compose logs -f"
    info "To stop Muxly, run: docker-compose down"
}

# Main installation flow
main() {
    print_banner
    check_docker
    create_directories
    check_config
    set_permissions
    build_images
    start_muxly
    
    echo
    success "Installation complete!"
    echo
    info "To get started with Muxly, follow these steps:"
    echo "1. Configure your connectors in config/muxly.toml"
    echo "2. Send a request to http://localhost:3000/health to verify the service is running"
    echo "3. Use the API to manage your connectors and data flows"
    echo
    info "For more information, visit: https://github.com/kelfra/muxly"
    echo
}

# Run the main function
main 