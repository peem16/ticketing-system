# Ticketing System — Monorepo

A **production-grade Ticketing System** built with **Rust**, designed using
**Clean Architecture**, **Microservices**, and **Event-Driven Architecture**.

This repository is optimized for:
- High performance
- Resource efficiency
- Clear architectural boundaries
- Long-term maintainability
- AI-assisted development with Cursor

---

## System Overview

### Architecture
- **Microservices** with strict service boundaries
- **Event-Driven** for asynchronous workflows
- **gRPC** for internal synchronous communication
- **REST** only where appropriate
- **GraphQL (Apollo Gateway)** as the external API Gateway
- **Kafka** as the event backbone

### Core Principles
- Each service owns its data
- No shared databases
- No hidden coupling
- Events represent immutable facts
- APIs are backward-compatible

---

## Tech Stack

| Area | Technology |
|----|----|
| Language | Rust |
| Internal API | gRPC |
| External API | Apollo GraphQL Gateway |
| Events | Kafka |
| Architecture | Clean Architecture + Microservices |
| Repo Style | Monorepo |
| Benchmarks | Criterion |
| Tooling | Cargo, Clippy, Rustfmt |

---
