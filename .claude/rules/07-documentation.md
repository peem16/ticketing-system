---
paths:
  - "**/*.rs"
  - "**/*.md"
  - "**/*.proto"
---

# Documentation Rules

## General
- Documentation is part of the system design
- Docs must reflect reality, not intent
- Outdated documentation is considered harmful

---

## Required Documentation
The following must be documented:
- Public APIs (REST / gRPC / GraphQL)
- Kafka events (schema, versioning, semantics)
- Non-obvious architectural decisions
- Performance-critical assumptions
- Failure modes and recovery strategies

---

## Code-Level Documentation
- Use Rust doc comments (`///`) for:
  - Public structs
  - Public traits
  - Public functions
- Document:
  - Invariants
  - Ownership expectations
  - Thread-safety assumptions
  - Error semantics

Avoid restating obvious code behavior.

---

## Architectural Documentation
- Major architectural decisions must be written down
- Prefer short, focused documents over large design specs
- Document *why* a decision was made, not just *what*

---

## Observability (Mandatory)

- Structured logging only
- Logs must include correlation / trace identifiers
- Errors must be classified (recoverable vs fatal)
- No silent failures
