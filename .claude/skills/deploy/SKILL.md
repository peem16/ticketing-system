---
name: deploy
description: Build and deploy all services using Docker Compose
disable-model-invocation: true
---

# Deploy Services

Follow these steps in order:

1. Run linting on all services:
   ```bash
   cd services/auth-service && cargo clippy --all-targets -- -D warnings
   cd services/api-gateway && cargo clippy --all-targets -- -D warnings
   ```

2. Run tests on auth-service:
   ```bash
   cd services/auth-service && cargo test
   ```

3. Build and start all services:
   ```bash
   make up-build
   ```

4. Verify services are running:
   ```bash
   docker-compose ps
   ```

5. Check health endpoints:
   ```bash
   curl -s http://localhost:3000/health
   curl -s http://localhost:8080/health
   ```

6. Show service logs for any startup errors:
   ```bash
   make logs
   ```

Report the status of each step. If any step fails, stop and report the error.
