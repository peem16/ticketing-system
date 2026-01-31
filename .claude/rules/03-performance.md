---
paths:
  - "**/*.rs"
---

# Performance & Resource Management

- Avoid unnecessary allocations
- Avoid cloning unless justified
- Prefer borrowing over ownership
- Avoid blocking operations in async contexts
- Be explicit about async vs sync boundaries
- Avoid spawning unbounded tasks
- Avoid unnecessary loops or recursion in hot paths
- Prefer zero-copy serialization/deserialization where feasible

Do not introduce performance regressions without explanation.

---

## Concurrency & Async

- All async code must be cancellation-safe
- Respect backpressure explicitly
- Avoid detached background tasks
- No hidden global executors
- Graceful shutdown must be supported
