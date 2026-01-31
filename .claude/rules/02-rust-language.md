---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---

# Language Rules (Rust)

- Follow official Rust style and idioms
- Avoid `unsafe` unless explicitly justified
- No hidden panics in production code paths
- Use `Result` with typed errors
- Avoid `unwrap()` and `expect()` outside tests
- Be explicit about lifetimes only when required
