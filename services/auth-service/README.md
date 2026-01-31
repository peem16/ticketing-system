# Auth Service

JWT-based authentication microservice built with Rust, following Clean Architecture principles.

## Features

- User registration with email/password
- User login with JWT token generation
- Token validation and user info retrieval
- Argon2 password hashing
- PostgreSQL storage via Diesel ORM

## Architecture

```
src/
├── domain/           # Business entities and interfaces (no dependencies)
│   ├── user.rs       # User entity and value objects
│   ├── auth.rs       # Repository and service traits
│   └── error.rs      # Domain errors
├── application/      # Use cases / command handlers
│   └── commands/
│       ├── register_user.rs
│       └── login_user.rs
├── infrastructure/   # External integrations
│   ├── db/           # Diesel + PostgreSQL
│   └── security/     # JWT + Argon2
└── interface/        # HTTP/gRPC adapters
    └── http/
        ├── handlers.rs
        └── router.rs
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | Register new user |
| POST | `/auth/login` | Authenticate and get JWT |
| GET | `/auth/me` | Get current user info (requires JWT) |
| GET | `/health` | Health check |

## Configuration

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | (required) |
| `AUTH_JWT_SECRET` | Secret key for JWT signing | (required) |
| `AUTH_JWT_EXP_SECS` | Token expiration in seconds | 3600 |
| `SERVER_HOST` | Server bind address | 127.0.0.1 |
| `SERVER_PORT` | Server port | 8080 |

## Development

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Diesel CLI (`cargo install diesel_cli --no-default-features --features postgres`)

### Setup

1. Create database:
   ```bash
   createdb auth_service
   ```

2. Run migrations:
   ```bash
   diesel migration run
   ```

3. Set environment variables:
   ```bash
   export DATABASE_URL=postgres://localhost/auth_service
   export AUTH_JWT_SECRET=your-secret-key
   ```

4. Run the service:
   ```bash
   cargo run
   ```

### Testing

```bash
# Unit tests
cargo test

# Integration tests (requires database)
cargo test -- --ignored
```

## Security Considerations

- Passwords are hashed using Argon2id
- JWT tokens have configurable expiration
- Email addresses are normalized and validated
- All errors are mapped to appropriate HTTP status codes without leaking internals
