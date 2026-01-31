# Build stage
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy proto files
COPY proto ./proto

# Copy manifests and build script
COPY services/api-gateway/Cargo.toml Cargo.toml
COPY services/api-gateway/build.rs build.rs

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY services/api-gateway/src ./src

# Touch main.rs to invalidate the binary cache
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/api-gateway /app/api-gateway

# Expose HTTP port
EXPOSE 3000

# Start the application
CMD ["/app/api-gateway"]
