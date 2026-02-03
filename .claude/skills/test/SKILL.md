---
name: test
description: Run all tests across the ticketing system
disable-model-invocation: true
---

# Run Tests

Follow these steps in order:

1. Run auth-service unit tests:
   ```bash
   cd services/auth-service && cargo test -- --nocapture
   ```

2. Run auth-service integration tests (requires DATABASE_URL):
   ```bash
   cd services/auth-service && cargo test -- --ignored --nocapture --test-threads=1
   ```

3. Run api-gateway compilation check:
   ```bash
   cd services/api-gateway && cargo check
   ```

4. Run clippy on all services:
   ```bash
   cd services/auth-service && cargo clippy --all-targets -- -D warnings
   cd services/api-gateway && cargo clippy --all-targets -- -D warnings
   ```

5. Check formatting:
   ```bash
   cd services/auth-service && cargo fmt -- --check
   cd services/api-gateway && cargo fmt -- --check
   ```

Report results for each step. If unit tests fail, show the failure details.
If integration tests fail because DATABASE_URL is not set, note that and continue.
