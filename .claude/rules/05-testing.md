---
paths:
  - "**/*.rs"
  - "**/tests/**/*"
---

# Testing Rules (Mandatory)

## General
- Tests are part of the production codebase
- Tests must be deterministic and reproducible
- No flaky or time-based tests
- No network access in unit tests

Do NOT skip tests unless explicitly approved.

---

## Unit Tests
- Required for domain and application layers
- Must test business invariants and edge cases
- Prefer table-driven tests
- Avoid mocking domain logic

Unit tests must:
- Run fast
- Allocate minimal resources
- Avoid async unless required by the interface

---

## Integration Tests
- Required for:
  - Kafka producers and consumers
  - gRPC services
  - Database access
- May use containers or test infrastructure
- Must be isolated and repeatable

Integration tests must NOT:
- Depend on production infrastructure
- Modify shared external state

---

## Error & Failure Testing
- Explicitly test failure paths
- Test idempotency for event consumers
- Test retries and cancellation behavior
- Test graceful shutdown where applicable

Untested failure paths are considered bugs.
