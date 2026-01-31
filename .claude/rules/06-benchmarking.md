---
paths:
  - "**/benches/**/*"
  - "**/Cargo.toml"
---

# Benchmarking Rules (Criterion)

## General
- Benchmarks are optional but encouraged
- Use **Criterion** as the standard benchmarking framework
- Benchmarks must be stable and repeatable

Do NOT benchmark trivial or I/O-bound code.

---

## Scope of Benchmarks
Benchmarks must focus on:
- CPU-heavy functions
- Allocation-heavy code paths
- Hot paths identified by profiling
- Performance-critical domain logic

Benchmarks must be written at the **function level**.

---

## What NOT to Benchmark
- Entire services
- Network calls
- Kafka producers/consumers end-to-end
- gRPC handlers as a whole
- Simple glue code

Benchmarking non-hot paths is discouraged.

---

## Benchmark Design
- Isolate the function under test
- Minimize setup cost inside the benchmark loop
- Avoid shared mutable state
- Avoid logging inside benchmarks

Benchmarks should answer:
> "Is this function fast enough and predictable under load?"

---

## Performance Changes
- Any optimization must be justified by:
  - Benchmark results, or
  - Clear reasoning tied to a hot path
- Avoid micro-optimizations without evidence
