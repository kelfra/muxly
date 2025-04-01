# Builder stage
FROM rust:1.72-slim as builder

WORKDIR /usr/src/muxly

# Create a dummy project to cache dependencies
RUN USER=root cargo new --bin muxly
WORKDIR /usr/src/muxly/muxly
COPY Cargo.toml ./
RUN cargo build --release && \
    rm -rf src/

# Copy the actual source code
COPY . .

# Build the actual project
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create muxly user and directories
RUN useradd -ms /bin/bash muxly && \
    mkdir -p /var/lib/muxly/data /var/lib/muxly/config /var/lib/muxly/templates && \
    chown -R muxly:muxly /var/lib/muxly

WORKDIR /var/lib/muxly

# Copy the binary from the builder stage
COPY --from=builder /usr/src/muxly/muxly/target/release/muxly /usr/local/bin/

# Copy default templates and configurations
COPY --from=builder /usr/src/muxly/muxly/config /var/lib/muxly/config/
COPY --from=builder /usr/src/muxly/muxly/templates /var/lib/muxly/templates/

# Set permissions
RUN chmod +x /usr/local/bin/muxly && \
    chown -R muxly:muxly /var/lib/muxly

# Switch to non-root user
USER muxly

# Expose API port
EXPOSE 3000

# Set up volumes
VOLUME ["/var/lib/muxly/data", "/var/lib/muxly/config"]

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["/usr/local/bin/muxly"] 