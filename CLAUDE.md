# CLAUDE.md — Project Guide for AI Assistants

## Project Overview

Microservices-based ticketing system written in **Rust**. Currently contains the **auth-service** and an **api-gateway**; additional services (ticket, notification) are planned under `services/`.

## Architecture

- **Clean Architecture**: domain → application → infrastructure → interface
- **Event-Driven Microservices** with per-service data ownership
- Domain layer has zero external dependencies; all I/O goes through trait abstractions

### Tech Stack

| Layer | Tech |
|---|---|
| Language | Rust (edition 2021) |
| Web framework | Axum 0.7 (services), Axum 0.8 (api-gateway) |
| API Gateway | **GraphQL** via `async-graphql` 7 — _differs from backend services which use REST/gRPC_ |
| Inter-service | gRPC via tonic 0.12 / prost 0.13 |
| Async runtime | Tokio |
| Database | PostgreSQL 16 via Diesel 2.0 + R2D2 pool |
| Auth | Argon2id password hashing, JWT (jsonwebtoken 9) |
| Serialization | Serde / serde_json |
| Logging | tracing + tracing-subscriber |
| Containers | Docker multi-stage builds, Docker Compose |

## Project Structure

```
proto/                     # Shared protobuf definitions
  auth.proto               # Auth gRPC service (Register, Login, GetMe, ValidateToken)

services/api-gateway/      # GraphQL API gateway (client-facing)
  src/
    main.rs                # Entry point, builds GraphQL schema
    config.rs              # Environment variable loading
    grpc_client.rs         # Generated gRPC stubs for auth-service
    schema.rs              # GraphQL Query + Mutation resolvers
    router.rs              # /graphql endpoint + GraphiQL playground
  build.rs                 # Proto compilation (tonic-build)

services/auth-service/     # Auth microservice (HTTP + gRPC)
  src/
    main.rs                # Entry point (serves HTTP :8080 + gRPC :50051)
    lib.rs                 # Library root, re-exports AppState
    domain/                # Entities, value objects, trait definitions (no external deps)
    application/commands/  # Use cases (RegisterUser, LoginUser)
    infrastructure/        # Diesel repos, Argon2 hasher, JWT service, config
    interface/http/        # Axum REST handlers + router
    interface/grpc/        # Tonic gRPC service implementation
  tests/                   # Integration tests
  migrations/              # Diesel SQL migrations
```

## Build & Run

```bash
# Docker (preferred)
make up-build          # Build and start all services
make down              # Stop services
make down-clean        # Stop and remove volumes
make logs              # All service logs
make auth-logs         # Auth service logs only
make gateway-logs      # API gateway logs only

# Local development (requires PostgreSQL + protoc)
cd services/auth-service
cargo build
cargo run

cd services/api-gateway
cargo build
cargo run

# Database
make migrate           # Run Diesel migrations
make db-shell          # Open psql console
```

## Testing

```bash
cd services/auth-service
cargo test                                    # Unit tests
cargo test -- --ignored                       # DB-dependent integration tests
cargo test -- --nocapture --test-threads=1    # Verbose single-threaded
```

- Unit tests live inline as `#[cfg(test)]` modules within source files.
- Integration tests are in `tests/integration_test.rs` (require `DATABASE_URL`).
- Mocks: `MockUserRepository`, `MockPasswordHasher`, `MockTokenService` defined in test modules.

## Environment Variables

### auth-service

| Variable | Required | Default | Notes |
|---|---|---|---|
| `DATABASE_URL` | Yes | — | Postgres connection string |
| `AUTH_JWT_SECRET` | Yes | — | JWT signing key |
| `AUTH_JWT_EXP_SECS` | No | `3600` | Token expiry in seconds |
| `SERVER_HOST` | No | `127.0.0.1` | Bind address |
| `SERVER_PORT` | No | `8080` | HTTP bind port |
| `GRPC_PORT` | No | `50051` | gRPC bind port |
| `RUST_LOG` | No | — | Tracing filter (e.g. `auth_service=debug,info`) |

### api-gateway

| Variable | Required | Default | Notes |
|---|---|---|---|
| `SERVER_HOST` | No | `127.0.0.1` | Bind address |
| `SERVER_PORT` | No | `3000` | HTTP bind port |
| `AUTH_SERVICE_GRPC_URL` | No | `http://127.0.0.1:50051` | auth-service gRPC endpoint |
| `RUST_LOG` | No | — | Tracing filter (e.g. `api_gateway=debug,info`) |

## API

### api-gateway — GraphQL (port 3000)

The API gateway uses **GraphQL** (not REST) as its client-facing protocol. This is a deliberate choice to give frontend clients flexible querying while backend services communicate via gRPC.

- `POST /graphql` — GraphQL endpoint
- `GET  /graphql` — GraphiQL interactive playground
- `GET  /health`  — Health check (REST, for infrastructure probes)

#### Schema

```graphql
type Query {
  me: User!            # Requires Authorization: Bearer <token>
  health: String!
}

type Mutation {
  register(input: RegisterInput!): RegisterPayload!
  login(input: LoginInput!): LoginPayload!
}
```

### auth-service — REST (port 8080) + gRPC (port 50051)

- `POST /auth/register` — Register a new user (201)
- `POST /auth/login` — Authenticate, returns JWT (200)
- `GET  /auth/me` — Current user info (requires Bearer token)
- `GET  /health` — Health check (200)

## Coding Conventions

- **Files/variables/functions**: `snake_case`
- **Types/traits/structs**: `PascalCase`
- **Trait implementations**: descriptive concrete names (`DieselUserRepository`, `Argon2PasswordHasher`)
- **Value objects**: `UserId`, `Email`, `HashedPassword` with validation in the domain layer
- **Error handling**: `Result` types throughout; domain errors map to HTTP status codes
- **Design patterns**: Repository, Use Case (command), Dependency Injection via trait objects
- **Formatting/linting**: `cargo fmt` and `cargo clippy` (default configs)
- **Doc comments**: `///` on all public items

## Key Design Decisions

- Domain layer must never depend on infrastructure crates.
- Each microservice owns its own database — no shared DB access.
- Password hashing uses Argon2id with random salts.
- JWT validation checks expiration and account active status.
- HTTP error responses do not leak internal details (e.g. `InvalidCredentials` → 401).
- **API gateway uses GraphQL** (`async-graphql`) while backend services use REST + gRPC. This is intentional — GraphQL provides flexible client-facing queries, gRPC provides typed inter-service communication.
- Shared `.proto` definitions live in `proto/` at the project root; each service compiles them via `tonic-build` in `build.rs`.
