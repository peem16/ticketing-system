# Implement Feature (Standard)

## Objective
Implement a new feature following existing architecture, rules, and documentation standards.

## Steps
1. Identify affected services and layers
2. Read relevant `.cursor/rules/*.mdc`
3. Locate existing feature or service README
4. Implement domain logic first
5. Implement application orchestration
6. Implement infrastructure adapters if required
7. Add or update tests
8. Update README / docs if behavior changes
9. Ask whether an ADR is required

## Constraints
- Do NOT invent business rules
- Do NOT break existing invariants
- Do NOT change public APIs without approval

If requirements are unclear:
> Stop and ask before implementing.