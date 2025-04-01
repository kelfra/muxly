#!/bin/bash
set -e

# Muxly Release Preparation Script
# This script prepares Muxly for a new release

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
echo "============================"
echo "   Muxly Release Preparation"
echo "============================"
echo -e "${NC}"

# Check if version argument is provided
if [ -z "$1" ]; then
    error "Please provide a version number as the first argument (e.g. 0.1.0)"
fi

VERSION=$1
info "Preparing release for version $VERSION"

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    warning "Working directory is not clean. Uncommitted changes may be included in the release."
    echo "Do you want to continue? (y/N)"
    read -r CONTINUE
    if [[ ! "$CONTINUE" =~ ^[Yy]$ ]]; then
        error "Release preparation aborted"
    fi
fi

# Run tests to ensure code quality
info "Running tests..."
if ! ./scripts/testing/run_tests.sh; then
    error "Tests failed. Please fix the issues before releasing."
fi
success "Tests passed"

# Update version in Cargo.toml
info "Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml && rm Cargo.toml.bak
success "Updated version in Cargo.toml"

# Update CHANGELOG.md if it exists
if [ -f "CHANGELOG.md" ]; then
    info "Updating CHANGELOG.md..."
    DATE=$(date +%Y-%m-%d)
    
    # Create a new entry in CHANGELOG.md
    NEW_ENTRY="## [$VERSION] - $DATE\n\n### Added\n- \n\n### Changed\n- \n\n### Fixed\n- \n"
    
    # Check if there's an Unreleased section
    if grep -q "## \[Unreleased\]" CHANGELOG.md; then
        # Replace the Unreleased section with the new version and keep any existing content
        sed -i.bak "/## \[Unreleased\]/,/## \[/s/## \[Unreleased\]/## [$VERSION] - $DATE/" CHANGELOG.md && rm CHANGELOG.md.bak
    else
        # Add a new version entry at the top of the file (after header)
        sed -i.bak "0,/# Changelog/s/# Changelog/# Changelog\n\n$NEW_ENTRY/" CHANGELOG.md && rm CHANGELOG.md.bak
    fi
    success "Updated CHANGELOG.md"
else
    info "Creating CHANGELOG.md..."
    DATE=$(date +%Y-%m-%d)
    cat > CHANGELOG.md << EOF
# Changelog

All notable changes to Muxly will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [$VERSION] - $DATE

### Added
- Initial release

### Changed
- 

### Fixed
- 
EOF
    success "Created CHANGELOG.md"
fi

# Build the release
info "Building release package..."
cargo build --release

if [ $? -ne 0 ]; then
    error "Release build failed"
fi
success "Release build completed"

# Create release directory if it doesn't exist
RELEASE_DIR="${PROJECT_ROOT}/releases/${VERSION}"
mkdir -p "$RELEASE_DIR"

# Copy binary and configuration files
info "Preparing release package..."
cp "${PROJECT_ROOT}/target/release/muxly" "$RELEASE_DIR/"
cp -r "${PROJECT_ROOT}/templates" "$RELEASE_DIR/"
cp -r "${PROJECT_ROOT}/config" "$RELEASE_DIR/"
cp "${PROJECT_ROOT}/README.md" "$RELEASE_DIR/"
cp "${PROJECT_ROOT}/CHANGELOG.md" "$RELEASE_DIR/"

# Create a default .env file for the release
cat > "${RELEASE_DIR}/.env" << EOF
# Muxly Environment Variables - v${VERSION}

# Server settings
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LOG_LEVEL=info

# Feature flags
ENABLE_PROMETHEUS=true
ENABLE_WEBHOOK=true
EOF

# Create release archive
ARCHIVE_NAME="muxly-${VERSION}-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz"
info "Creating release archive: $ARCHIVE_NAME"
tar -czf "${PROJECT_ROOT}/releases/${ARCHIVE_NAME}" -C "${PROJECT_ROOT}/releases" "${VERSION}"
success "Created release archive at releases/${ARCHIVE_NAME}"

# Create Docker image
if command -v docker &> /dev/null; then
    info "Building Docker image..."
    DOCKER_TAG="muxly:${VERSION}"
    docker build -t "$DOCKER_TAG" .
    
    if [ $? -eq 0 ]; then
        success "Docker image built: $DOCKER_TAG"
        
        # Save Docker image to file
        DOCKER_IMAGE_FILE="muxly-${VERSION}-docker.tar"
        info "Saving Docker image to file: $DOCKER_IMAGE_FILE"
        docker save -o "${PROJECT_ROOT}/releases/${DOCKER_IMAGE_FILE}" "$DOCKER_TAG"
        success "Docker image saved to releases/${DOCKER_IMAGE_FILE}"
    else
        warning "Docker image build failed"
    fi
else
    warning "Docker not found. Skipping Docker image creation."
fi

# Prepare git for release
info "Preparing Git release..."
git add Cargo.toml CHANGELOG.md
git commit -m "Release version $VERSION"
git tag -a "v$VERSION" -m "Version $VERSION"

echo
success "Release preparation complete!"
echo 
echo "Next steps:"
echo "1. Review the changes and CHANGELOG.md"
echo "2. Push the changes: git push && git push origin v$VERSION"
echo "3. Create a GitHub release with the archive from: releases/${ARCHIVE_NAME}"
echo
echo "To run the release:"
echo "  - Extract the archive: tar -xzf ${ARCHIVE_NAME}"
echo "  - Run the binary: ./muxly"
echo
echo "To use the Docker image:"
echo "  - Load the image: docker load -i ${DOCKER_IMAGE_FILE}"
echo "  - Run the container: docker run -p 8080:8080 $DOCKER_TAG" 