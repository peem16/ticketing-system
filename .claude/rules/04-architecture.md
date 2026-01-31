---
paths:
  - "services/**/*"
  - "docker-compose*.yml"
---

# Clean Architecture (Strict)

## Domain Layer
- No dependency on infrastructure
- No framework types
- No async unless unavoidable
- Contains business invariants only

## Application Layer
- Coordinates use cases
- Contains no transport logic

## Infrastructure Layer
- Kafka, gRPC, databases, external APIs only
- No business logic

## Interface Layer
- REST / gRPC / GraphQL adapters only

Violating the dependency rule is NOT allowed.

---

## Microservices & Boundaries

- Each service owns its data
- No shared database access
- Cross-service communication via gRPC or events only
- No implicit coupling via shared structs or packages

---

## Event-Driven Rules (Kafka)

- Events are immutable facts
- Explicit event versioning is required
- Consumers must be idempotent
- Handle duplicate and out-of-order events
- Events are NOT commands
- No synchronous dependency on event consumption

---

## API Rules

### REST
- Secondary integration mechanism
- Follow HTTP semantics strictly

### gRPC
- Preferred for internal communication
- APIs must be backward-compatible
- Avoid chatty interfaces

### GraphQL
- Apollo Gateway only
- Gateway must not contain business logic
- Resolvers orchestrate calls, not domain computation

---

## Monorepo Rules

- Shared libraries must be explicit and minimal
- No implicit cross-service imports
- No circular dependencies between services
