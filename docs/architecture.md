# Architecture Decisions

## Why Event-Driven + Microservices
- Scale independently
- Reduce synchronous coupling

## Why Kafka
- High throughput
- Replayability
- Strong ordering guarantees per partition

## Why gRPC for internal calls
- Strong typing
- Backward compatibility
- Lower overhead vs REST

## Why GraphQL at the API Gateway
- Clients can request exactly the fields they need
- Single endpoint for all operations — simplifies client integration
- Introspectable schema (GraphiQL playground available at `/graphql`)
- Backend services still communicate via gRPC for performance and type safety
- `async-graphql` crate keeps the gateway in Rust, consistent with the rest of the stack