#!/bin/bash
set -e

# Muxly Development Environment Setup Script
# This script sets up a development environment for the Muxly project

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

# Change to the project root directory
cd "$(dirname "$0")/../.."
PROJECT_ROOT=$(pwd)

# Print the banner
echo -e "${BLUE}"
echo "==============================="
echo "   Muxly Dev Environment Setup"
echo "==============================="
echo -e "${NC}"

# Check dependencies first
info "Checking dependencies..."
if ! "${PROJECT_ROOT}/scripts/utils/check_dependencies.sh"; then
    error "Dependencies check failed. Please install the required dependencies."
fi

# Setup Rust environment
info "Setting up Rust development environment..."

# Add Rust components if needed
if command -v rustup &> /dev/null; then
    info "Installing/updating Rust components..."
    rustup component add rustfmt clippy rust-src
    success "Rust components installed"
fi

# Install development tools
info "Installing additional development tools..."
cargo install cargo-watch cargo-expand cargo-edit || warning "Some development tools could not be installed"

# Create .env file if it doesn't exist
if [ ! -f "${PROJECT_ROOT}/.env" ]; then
    info "Creating default .env file..."
    cat > "${PROJECT_ROOT}/.env" << EOF
# Muxly Development Environment Variables

# Server settings
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
LOG_LEVEL=debug

# Database settings
# Uncomment and update these as needed
# DB_HOST=localhost
# DB_PORT=5432
# DB_USER=postgres
# DB_PASSWORD=password
# DB_NAME=muxly

# Feature flags
ENABLE_PROMETHEUS=true
ENABLE_WEBHOOK=true
EOF
    success "Created default .env file"
fi

# Create sample config files
if [ ! -d "${PROJECT_ROOT}/config" ]; then
    info "Creating sample configuration files..."
    mkdir -p "${PROJECT_ROOT}/config"
    
    # Create a basic config.json if it doesn't exist
    if [ ! -f "${PROJECT_ROOT}/config/config.json" ]; then
        cat > "${PROJECT_ROOT}/config/config.json" << EOF
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080,
    "workers": 4
  },
  "logging": {
    "level": "debug",
    "file": "logs/muxly.log"
  },
  "destinations": [
    {
      "type": "file",
      "name": "local_file",
      "config": {
        "path": "data/metrics.txt",
        "format": "text",
        "rotate": {
          "max_size_mb": 10,
          "keep_files": 5
        }
      }
    }
  ]
}
EOF
        success "Created sample config.json"
    fi
fi

# Create logs and data directories
mkdir -p "${PROJECT_ROOT}/logs" "${PROJECT_ROOT}/data"
success "Created log and data directories"

# Setup Docker environment if Docker is available
if command -v docker &> /dev/null; then
    info "Setting up Docker development environment..."
    
    # Create Docker development script if it doesn't exist
    if [ ! -f "${PROJECT_ROOT}/scripts/docker/dev.sh" ]; then
        mkdir -p "${PROJECT_ROOT}/scripts/docker"
        cat > "${PROJECT_ROOT}/scripts/docker/dev.sh" << EOF
#!/bin/bash
set -e

cd "\$(dirname "\$0")/../.."

echo "Starting development Docker containers..."
docker-compose -f scripts/docker/docker-compose.prometheus.yml up -d

echo "Development environment is now running."
echo "- Muxly: http://localhost:8080"
echo "- Prometheus: http://localhost:9090"
echo "- Grafana: http://localhost:3000"
echo
echo "To stop the environment, run: docker-compose -f scripts/docker/docker-compose.prometheus.yml down"
EOF
        chmod +x "${PROJECT_ROOT}/scripts/docker/dev.sh"
        success "Created Docker development script"
    fi
fi

# Generate Visual Studio Code configuration if needed
if command -v code &> /dev/null; then
    if [ ! -d "${PROJECT_ROOT}/.vscode" ]; then
        info "Setting up VSCode configuration..."
        mkdir -p "${PROJECT_ROOT}/.vscode"
        
        # Create settings.json
        cat > "${PROJECT_ROOT}/.vscode/settings.json" << EOF
{
    "editor.formatOnSave": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.inlayHints.typeHints.enable": true,
    "rust-analyzer.inlayHints.parameterHints.enable": true
}
EOF
        
        # Create launch.json for debugging
        cat > "${PROJECT_ROOT}/.vscode/launch.json" << EOF
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable",
            "cargo": {
                "args": [
                    "build",
                    "--bin=muxly",
                    "--package=muxly"
                ]
            },
            "args": [],
            "cwd": "\${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug unit tests",
            "cargo": {
                "args": [
                    "test",
                    "--no-run",
                    "--bin=muxly",
                    "--package=muxly"
                ]
            },
            "args": [],
            "cwd": "\${workspaceFolder}"
        }
    ]
}
EOF
        success "VSCode configuration created"
    fi
fi

echo
success "Development environment setup complete!"
echo
echo "You can now begin development on Muxly."
echo "- To run the project: cargo run"
echo "- To run tests: ./scripts/testing/run_tests.sh"
echo "- To run benchmarks: ./scripts/testing/benchmark.sh"
echo
echo "If you're using Docker, you can start the development environment with:"
echo "  ./scripts/docker/dev.sh" 