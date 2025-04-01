#!/bin/bash

# Muxly Dependency Checker
# This script verifies that all necessary dependencies for Muxly are installed

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
    echo -e "${GREEN}[PASS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[MISSING]${NC} $1"
    MISSING_DEPS=1
}

# Print the banner
echo -e "${BLUE}"
echo "============================"
echo "   Muxly Dependency Check"
echo "============================"
echo -e "${NC}"

echo "Checking required dependencies for Muxly..."
echo

# Initialize missing dependencies flag
MISSING_DEPS=0

# Check for Rust/Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    success "Cargo: $CARGO_VERSION"
else
    error "Cargo not found. Please install Rust from https://rustup.rs/"
fi

# Check for Docker
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    success "Docker: $DOCKER_VERSION"
else
    error "Docker not found. Required for containerized deployment."
    echo "    Install from https://docs.docker.com/get-docker/"
fi

# Check for Docker Compose
if command -v docker-compose &> /dev/null; then
    COMPOSE_VERSION=$(docker-compose --version)
    success "Docker Compose: $COMPOSE_VERSION"
elif command -v docker &> /dev/null && docker compose version &> /dev/null; then
    COMPOSE_VERSION=$(docker compose version)
    success "Docker Compose (Plugin): $COMPOSE_VERSION"
else
    error "Docker Compose not found. Required for multi-container deployments."
    echo "    Install from https://docs.docker.com/compose/install/"
fi

# Check for Git
if command -v git &> /dev/null; then
    GIT_VERSION=$(git --version)
    success "Git: $GIT_VERSION"
else
    error "Git not found. Required for version control."
    echo "    Install from https://git-scm.com/downloads"
fi

# Check Rust toolchain components
if command -v rustup &> /dev/null; then
    # Check for rustfmt
    if rustup component list | grep -q "rustfmt.*installed"; then
        success "rustfmt: Installed"
    else
        warning "rustfmt not installed. Required for code formatting."
        echo "    Install with: rustup component add rustfmt"
    fi

    # Check for clippy
    if rustup component list | grep -q "clippy.*installed"; then
        success "clippy: Installed"
    else
        warning "clippy not installed. Required for linting."
        echo "    Install with: rustup component add clippy"
    fi
else
    warning "rustup not found. Cannot check Rust components."
fi

# Optional dependencies
echo
echo "Checking optional dependencies..."

# Check for Prometheus CLI tools
if command -v promtool &> /dev/null; then
    PROM_VERSION=$(promtool --version 2>&1 | head -n 1)
    success "Prometheus tools: $PROM_VERSION"
else
    warning "Prometheus tools not found. Optional for metrics validation."
fi

# Result summary
echo
if [ $MISSING_DEPS -eq 0 ]; then
    echo -e "${GREEN}All required dependencies are installed!${NC}"
    exit 0
else
    echo -e "${RED}Some required dependencies are missing.${NC}"
    echo "Please install the missing dependencies before proceeding."
    exit 1
fi 