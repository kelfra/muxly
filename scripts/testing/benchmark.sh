#!/bin/bash
set -e

# Benchmark script for the Muxly project

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
echo "============================"
echo "   Muxly Benchmark Runner"
echo "============================"
echo -e "${NC}"

# Check if criterion is installed
if ! cargo bench --help | grep -q criterion; then
    warning "Criterion doesn't seem to be configured. Adding it to dependencies..."
    if ! grep -q "\[dev-dependencies\]" Cargo.toml; then
        echo -e "\n[dev-dependencies]" >> Cargo.toml
    fi
    if ! grep -q "criterion" Cargo.toml; then
        echo 'criterion = "0.5"' >> Cargo.toml
    fi
    
    # Check if bench section exists in Cargo.toml
    if ! grep -q "\[\[bench\]\]" Cargo.toml; then
        warning "Adding benchmark configuration to Cargo.toml"
        cat >> Cargo.toml << EOF

[[bench]]
name = "router_benchmark"
harness = false

[[bench]]
name = "destination_benchmark"
harness = false
EOF
    fi
fi

# Create benchmark directory if it doesn't exist
if [ ! -d "benches" ]; then
    info "Creating benchmark directory..."
    mkdir -p benches
    
    # Create basic router benchmark if it doesn't exist
    if [ ! -f "benches/router_benchmark.rs" ]; then
        info "Creating sample router benchmark..."
        cat > benches/router_benchmark.rs << EOF
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use muxly::router::{Router, Destination};

// Sample dummy destination for benchmarking
struct DummyDestination;

impl Destination for DummyDestination {
    fn send(&self, _data: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Do nothing, just for benchmarking
        Ok(())
    }

    fn send_batch(&self, _data: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        // Do nothing, just for benchmarking
        Ok(())
    }
}

fn router_benchmark(c: &mut Criterion) {
    let mut router = Router::new();
    let destination = DummyDestination;
    router.add_destination("dummy", Box::new(destination));

    c.bench_function("route single message", |b| {
        b.iter(|| {
            router.route(black_box("test data")).unwrap();
        })
    });

    let batch_data = vec!["data1".to_string(), "data2".to_string(), "data3".to_string()];
    c.bench_function("route batch messages", |b| {
        b.iter(|| {
            router.route_batch(black_box(&batch_data)).unwrap();
        })
    });
}

criterion_group!(benches, router_benchmark);
criterion_main!(benches);
EOF
    fi

    # Create basic destination benchmark if it doesn't exist
    if [ ! -f "benches/destination_benchmark.rs" ]; then
        info "Creating sample destination benchmark..."
        cat > benches/destination_benchmark.rs << EOF
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use muxly::router::destinations::file::FileDestination;
use muxly::router::Destination;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

fn destination_benchmark(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("benchmark_output.txt");
    
    // Create file destination
    let file_dest = FileDestination::new(file_path.to_str().unwrap()).unwrap();
    
    // Benchmark single message
    c.bench_function("file destination single message", |b| {
        b.iter(|| {
            file_dest.send(black_box("test metric data")).unwrap();
        })
    });
    
    // Benchmark batch messages
    let batch_data = vec![
        "metric1{label=\"value\"} 10".to_string(),
        "metric2{label=\"value\"} 20".to_string(),
        "metric3{label=\"value\"} 30".to_string(),
    ];
    
    c.bench_function("file destination batch messages", |b| {
        b.iter(|| {
            file_dest.send_batch(black_box(&batch_data)).unwrap();
        })
    });
}

criterion_group!(benches, destination_benchmark);
criterion_main!(benches);
EOF
    fi
fi

# Run benchmarks
info "Running benchmarks..."
cargo bench || error "Benchmarks failed"

success "Benchmarks completed successfully!"
echo "Results are available in the target/criterion directory" 