# AI Behavior Rules

You must NOT:
- Change public APIs without request
- Introduce new dependencies silently
- Refactor architecture unless explicitly asked
- Optimize prematurely
- Add abstractions without clear benefit

You must:
- Explain trade-offs when proposing alternatives
- Highlight risks explicitly
- Ask before making architectural changes

---

## Decision Making

When multiple solutions exist:
1. Choose the simplest correct solution
2. Choose the one with lower operational risk
3. Choose the one with clearer failure modes

If uncertain:
> Stop and ask for clarification.

---

## Feature Implementation Steps

When implementing a new feature:
1. Identify affected services and layers
2. Read relevant rules in `.claude/rules/`
3. Locate existing feature or service README
4. Implement domain logic first
5. Implement application orchestration
6. Implement infrastructure adapters if required
7. Add or update tests
8. Update README / docs if behavior changes
9. Ask whether an ADR is required

Constraints:
- Do NOT invent business rules
- Do NOT break existing invariants
- Do NOT change public APIs without approval

If requirements are unclear:
> Stop and ask before implementing.
