# CLAUDE.md — Project Guide for AI Assistants

## Project Overview

Microservices-based ticketing system written in **Rust**. Currently contains the **auth-service**; additional services (ticket, notification) are planned under `services/`.

## Architecture

- **Clean Architecture**: domain → application → infrastructure → interface
- **Event-Driven Microservices** with per-service data ownership
- Domain layer has zero external dependencies; all I/O goes through trait abstractions

### Tech Stack

| Layer | Tech |
|---|---|
| Language | Rust (edition 2021) |
| Web framework | Axum 0.7 |
| Async runtime | Tokio |
| Database | PostgreSQL 16 via Diesel 2.0 + R2D2 pool |
| Auth | Argon2id password hashing, JWT (jsonwebtoken 9) |
| Serialization | Serde / serde_json |
| Logging | tracing + tracing-subscriber |
| Containers | Docker multi-stage builds, Docker Compose |

## Project Structure

```
services/auth-service/
  src/
    main.rs              # Entry point
    lib.rs               # Library root, re-exports AppState
    domain/              # Entities, value objects, trait definitions (no external deps)
    application/commands/ # Use cases (RegisterUser, LoginUser)
    infrastructure/      # Diesel repos, Argon2 hasher, JWT service, config
    interface/http/      # Axum handlers + router
  tests/                 # Integration tests
  migrations/            # Diesel SQL migrations
```

## Build & Run

```bash
# Docker (preferred)
make up-build          # Build and start all services
make down              # Stop services
make down-clean        # Stop and remove volumes
make logs              # All service logs
make auth-logs         # Auth service logs only

# Local development (requires PostgreSQL)
cd services/auth-service
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

| Variable | Required | Default | Notes |
|---|---|---|---|
| `DATABASE_URL` | Yes | — | Postgres connection string |
| `AUTH_JWT_SECRET` | Yes | — | JWT signing key |
| `AUTH_JWT_EXP_SECS` | No | `3600` | Token expiry in seconds |
| `SERVER_HOST` | No | `127.0.0.1` | Bind address |
| `SERVER_PORT` | No | `8080` | Bind port |
| `RUST_LOG` | No | — | Tracing filter (e.g. `auth_service=debug,info`) |

## API Endpoints (auth-service)

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
