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