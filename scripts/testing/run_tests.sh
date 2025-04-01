#!/bin/bash
set -e

# Run all tests for the Muxly project

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

# Print the banner
echo -e "${BLUE}"
echo "=========================="
echo "   Muxly Test Runner"
echo "=========================="
echo -e "${NC}"

# Run cargo tests
info "Running unit tests..."
cargo test || error "Unit tests failed"
success "Unit tests passed"

# Run integration tests
info "Running integration tests..."
cargo test --test '*' || error "Integration tests failed"
success "Integration tests passed"

# Run linter
info "Running linter..."
cargo clippy -- -D warnings || error "Linter found issues"
success "Linter passed"

# Run formatter check
info "Checking code formatting..."
cargo fmt --check || error "Code formatting issues found"
success "Code formatting is correct"

# Run documentation tests
info "Running documentation tests..."
cargo test --doc || error "Documentation tests failed"
success "Documentation tests passed"

echo
success "All tests passed!" 