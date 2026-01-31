# Build stage
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Install protobuf compiler and diesel CLI
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
RUN cargo install diesel_cli --no-default-features --features postgres

# Copy proto files
COPY proto ./proto

# Copy manifests and build script
COPY services/auth-service/Cargo.toml Cargo.toml
COPY services/auth-service/build.rs build.rs

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY services/auth-service/src ./src
COPY services/auth-service/migrations ./migrations
COPY services/auth-service/diesel.toml ./diesel.toml

# Touch main.rs to invalidate the binary cache
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/auth-service /app/auth-service

# Copy diesel CLI and migrations for runtime migration support
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/diesel.toml /app/diesel.toml

# Copy entrypoint script and fix line endings (Windows CRLF -> Unix LF)
COPY services/auth-service/docker-entrypoint.sh /app/docker-entrypoint.sh
RUN sed -i 's/\r$//' /app/docker-entrypoint.sh && chmod +x /app/docker-entrypoint.sh

# Expose HTTP and gRPC ports
EXPOSE 8080 50051

# Set entrypoint
ENTRYPOINT ["/app/docker-entrypoint.sh"]
