---
name: review
description: Code review current branch changes against main
---

# Code Review

Review all changes on the current branch compared to the base branch.

## Steps

1. Identify the base branch and get the diff:
   ```bash
   git log --oneline main..HEAD
   git diff main...HEAD
   ```

2. For each changed file, review for:
   - **Correctness**: Logic errors, off-by-one, null handling
   - **Architecture**: Clean Architecture violations (domain depending on infra)
   - **Rust idioms**: Unnecessary `unwrap()`, `clone()`, missing error handling
   - **Security**: SQL injection, credential leaks, unsafe token handling
   - **Performance**: Unnecessary allocations, blocking in async, unbounded tasks
   - **Testing**: Are new code paths tested? Missing edge cases?

3. Check against project rules in `.claude/rules/`:
   - `02-rust-language.md`: No unsafe, no hidden panics, use Result
   - `03-performance.md`: No unnecessary alloc, cancellation-safe async
   - `04-architecture.md`: Dependency rule, service boundaries
   - `05-testing.md`: Tests required for domain/application layers

4. Provide a structured review:
   ```
   ## Summary
   [1-2 sentence overview]

   ## Issues Found
   - [CRITICAL] file:line - description
   - [WARNING] file:line - description
   - [SUGGESTION] file:line - description

   ## Positive Notes
   - What was done well

   ## Verdict
   APPROVE / REQUEST_CHANGES / NEEDS_DISCUSSION
   ```
