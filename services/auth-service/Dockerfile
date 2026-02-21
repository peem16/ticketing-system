# ============================================================================
# Stage 1: cargo-chef — compute a dependency-only build plan
# ============================================================================
FROM rust:1.92-bookworm AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# ============================================================================
# Stage 2: Prepare recipe (dependency lockfile for caching)
# ============================================================================
FROM chef AS planner

# Copy everything needed for dependency resolution
COPY proto ./proto
COPY services/auth-service/Cargo.toml Cargo.toml
COPY services/auth-service/build.rs build.rs
COPY services/auth-service/src ./src
COPY services/auth-service/migrations ./migrations

RUN cargo chef prepare --recipe-path recipe.json

# ============================================================================
# Stage 3: Build dependencies (cached unless Cargo.toml/lock changes)
# ============================================================================
FROM chef AS builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy proto files
COPY proto ./proto

# Copy build script (needed for tonic-build during dep compilation)
COPY services/auth-service/build.rs build.rs

# Cook dependencies from the recipe (this layer is heavily cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy actual source code + migrations
COPY services/auth-service/src ./src
COPY services/auth-service/migrations ./migrations
COPY services/auth-service/diesel.toml ./diesel.toml

# Build the actual application
RUN cargo build --release

# ============================================================================
# Stage 4: Minimal runtime image
# ============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install only the minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/auth-service /app/auth-service

# Copy entrypoint script and fix line endings
COPY services/auth-service/docker-entrypoint.sh /app/docker-entrypoint.sh
RUN sed -i 's/\r$//' /app/docker-entrypoint.sh && chmod +x /app/docker-entrypoint.sh

# Expose HTTP and gRPC ports
EXPOSE 8080 50051

# Set entrypoint
ENTRYPOINT ["/app/docker-entrypoint.sh"]
