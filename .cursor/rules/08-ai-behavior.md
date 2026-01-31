# AI Behavior Rules

AI must NOT:
- Change public APIs without request
- Introduce new dependencies silently
- Refactor architecture unless explicitly asked
- Optimize prematurely
- Add abstractions without clear benefit

AI must:
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
